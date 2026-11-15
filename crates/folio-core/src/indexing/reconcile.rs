//! Reconciling a directory: compare what the disk has in it with what the
//! index has for it, and apply the difference (spec 05).

use std::collections::HashSet;

use rusqlite::OptionalExtension;

use super::disk::file_paths;
use crate::Result;
use crate::documents::file_facts::FileFacts;
use crate::documents::index::{document_format, mark_missing};
use crate::paths::RelativePath;
use crate::workspace::Workspace;

/// What a reconciliation changed in the index.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ReconcileSummary {
    pub added: u64,
    pub updated: u64,
    pub reconnected: u64,
    pub restored: u64,
    pub missing: u64,
}

impl std::ops::AddAssign for ReconcileSummary {
    fn add_assign(&mut self, other: Self) {
        self.added += other.added;
        self.updated += other.updated;
        self.reconnected += other.reconnected;
        self.restored += other.restored;
        self.missing += other.missing;
    }
}

/// How a document file on disk relates to the index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnDisk {
    New,
    Unchanged,
    Changed,
    /// Missing in the index and back at the same path (e.g. restored from
    /// the trash): it keeps its id.
    Restored,
}

/// What the index has for a path.
struct Recorded {
    size: u64,
    modified_at: i64,
    missing: bool,
}

impl Workspace {
    /// Brings the index in step with the documents directly in `dir` (the
    /// root for `None`). Running it twice changes nothing the second time.
    pub fn reconcile_dir(&self, dir: Option<&RelativePath>) -> Result<ReconcileSummary> {
        Ok(self.reconcile(dir)?.0)
    }

    /// Reconciles `dir` and also returns how many documents it has on disk.
    pub(super) fn reconcile(&self, dir: Option<&RelativePath>) -> Result<(ReconcileSummary, u64)> {
        let mut summary = ReconcileSummary::default();
        let on_disk: HashSet<RelativePath> = file_paths(self.root(), dir)?.into_iter().collect();
        // The whole difference of the directory is written in one transaction.
        let batch = self.begin_batch()?;
        for path in &on_disk {
            self.reconcile_file(path, &mut summary)?;
        }
        summary.missing = self.mark_vanished(dir, &on_disk)?;
        batch.commit()?;
        let seen = on_disk
            .iter()
            .filter(|path| document_format(path.file_name()).is_some());
        Ok((summary, seen.count() as u64))
    }

    /// Present documents directly in `dir` that are no longer on disk become
    /// missing (spec 05, "Ausentes"). Returns how many.
    fn mark_vanished(
        &self,
        dir: Option<&RelativePath>,
        on_disk: &HashSet<RelativePath>,
    ) -> Result<u64> {
        let now = self.clock.now_millis();
        let vanished: Vec<_> = self
            .list_documents(dir)?
            .into_iter()
            .filter(|doc| !on_disk.contains(&doc.path))
            .collect();
        let mut conn = self.conn();
        for doc in &vanished {
            mark_missing(&mut conn, doc.id, now).map_err(|e| self.db_error(e))?;
        }
        Ok(vanished.len() as u64)
    }

    fn reconcile_file(&self, path: &RelativePath, summary: &mut ReconcileSummary) -> Result<()> {
        let state = self.classify(path)?;
        if state == OnDisk::New && self.try_reconnect(path)? {
            summary.reconnected += 1;
            return Ok(());
        }
        if state == OnDisk::Unchanged || self.index_file(path)?.is_none() {
            return Ok(());
        }
        match state {
            OnDisk::New => summary.added += 1,
            OnDisk::Restored => summary.restored += 1,
            _ => summary.updated += 1,
        }
        Ok(())
    }

    /// Reconnects a new file to the document it was, if one matches.
    fn try_reconnect(&self, path: &RelativePath) -> Result<bool> {
        let Some(id) = self.find_reconnection(path)? else {
            return Ok(false);
        };
        self.reconnect(id, path)?;
        Ok(true)
    }

    /// Compares the size and mtime on disk now with the ones indexed. The
    /// signature is only read again for new or changed files.
    fn classify(&self, path: &RelativePath) -> Result<OnDisk> {
        let Some(recorded) = self.recorded(path)? else {
            return Ok(OnDisk::New);
        };
        if recorded.missing {
            return Ok(OnDisk::Restored);
        }
        let facts = FileFacts::read(&path.resolve(self.root()))?;
        let same = facts.size == recorded.size && facts.modified_at == recorded.modified_at;
        Ok(if same {
            OnDisk::Unchanged
        } else {
            OnDisk::Changed
        })
    }

    fn recorded(&self, path: &RelativePath) -> Result<Option<Recorded>> {
        let sql =
            "SELECT size, modified_at, missing_since IS NOT NULL FROM documents WHERE path = ?1";
        let conn = self.conn();
        let query = || -> rusqlite::Result<Option<Recorded>> {
            conn.prepare_cached(sql)?
                .query_row([path], |row| {
                    Ok(Recorded {
                        size: row.get::<_, i64>(0)? as u64,
                        modified_at: row.get(1)?,
                        missing: row.get(2)?,
                    })
                })
                .optional()
        };
        query().map_err(|e| self.db_error(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    pub fn rel(text: &str) -> RelativePath {
        RelativePath::from_bytes(text).unwrap()
    }

    fn names(workspace: &Workspace, dir: Option<&RelativePath>) -> Vec<String> {
        let docs = workspace.list_documents(dir).unwrap();
        docs.into_iter().map(|doc| doc.name).collect()
    }

    #[test]
    fn reconcile_indexes_new_documents_in_dir() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Faculdade/Trabalho.pdf", b"%PDF-1.7\n");
        write_doc(&workspace, "Faculdade/Notas.md", b"# Notas");

        let summary = workspace.reconcile_dir(Some(&rel("Faculdade"))).unwrap();

        assert_eq!(summary.added, 2);
        assert_eq!(
            names(&workspace, Some(&rel("Faculdade"))),
            ["Notas.md", "Trabalho.pdf"]
        );
    }

    #[test]
    fn reconcile_ignores_non_documents_hidden_files_and_subdirs() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "planilha.xlsx", b"PK");
        write_doc(&workspace, ".rascunho.md", b"# oculto");
        write_doc(&workspace, "~$ntrato.docx", b"lock");
        write_doc(&workspace, "Faculdade/Dentro.md", b"# dentro");

        let summary = workspace.reconcile_dir(None).unwrap();

        assert_eq!(
            (summary.added, names(&workspace, None)),
            (0, Vec::<String>::new())
        );
    }

    #[test]
    fn reconcile_does_not_reread_unchanged_document() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Trabalho.pdf", b"%PDF-1.7\n");
        workspace.reconcile_dir(None).unwrap();
        write_doc(&workspace, "Trabalho.pdf", b"<html>x\n\n");

        workspace.reconcile_dir(None).unwrap();

        assert!(workspace.list_documents(None).unwrap()[0].valid);
    }

    #[test]
    fn reconcile_updates_changed_document_and_rechecks_signature() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Trabalho.pdf", b"%PDF-1.7\n");
        workspace.reconcile_dir(None).unwrap();
        write_doc(&workspace, "Trabalho.pdf", b"<html>truncado</html>");

        let summary = workspace.reconcile_dir(None).unwrap();

        let doc = &workspace.list_documents(None).unwrap()[0];
        assert_eq!((summary.added, summary.updated), (0, 1));
        assert_eq!((doc.size, doc.valid), (21, false));
    }

    #[test]
    fn reconcile_marks_vanished_document_missing() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Trabalho.pdf", b"%PDF-1.7\n");
        write_doc(&workspace, "Notas.md", b"# Notas");
        workspace.reconcile_dir(None).unwrap();
        std::fs::remove_file(workspace.root().join("Trabalho.pdf")).unwrap();

        let summary = workspace.reconcile_dir(None).unwrap();

        assert_eq!(
            (summary.missing, names(&workspace, None)),
            (1, vec!["Notas.md".to_string()])
        );
    }

    #[test]
    fn reconcile_twice_changes_nothing_the_second_time() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Trabalho.pdf", b"%PDF-1.7\n");
        write_doc(&workspace, "Notas.md", b"# Notas");
        workspace.reconcile_dir(None).unwrap();
        std::fs::remove_file(workspace.root().join("Notas.md")).unwrap();
        workspace.reconcile_dir(None).unwrap();

        let second = workspace.reconcile_dir(None).unwrap();

        assert_eq!(second, ReconcileSummary::default());
    }

    #[test]
    fn reconcile_restores_missing_document_found_at_same_path() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Trabalho.pdf", b"%PDF-1.7\n");
        workspace.reconcile_dir(None).unwrap();
        let away = env.dir("Lixeira").join("Trabalho.pdf");
        std::fs::rename(workspace.root().join("Trabalho.pdf"), &away).unwrap();
        workspace.reconcile_dir(None).unwrap();
        std::fs::rename(&away, workspace.root().join("Trabalho.pdf")).unwrap();

        let summary = workspace.reconcile_dir(None).unwrap();

        assert_eq!(
            (summary.restored, names(&workspace, None)),
            (1, vec!["Trabalho.pdf".to_string()])
        );
    }
}
