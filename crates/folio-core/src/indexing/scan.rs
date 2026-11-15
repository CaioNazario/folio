//! The full scan: reconciling every directory of the workspace, from the root
//! down (spec 05, "Escaneamento completo").

use std::collections::HashSet;
use std::fs;

use super::ReconcileSummary;
use super::disk::file_paths;
use crate::db::rebase::{PathTable, delete_path, table_paths};
use crate::documents::index::mark_missing;
use crate::integrity::HIDDEN_TEMP_PREFIX;
use crate::paths::{RelativePath, os_bytes};
use crate::workspace::{DirKind, Workspace};
use crate::{Error, Result};

/// How a full scan behaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanOptions {
    /// Remove leftover `.folio-tmp-*` files. Only the scan on opening does it:
    /// then no conversion is running.
    pub remove_temp_files: bool,
}

/// Running totals, reported after each directory.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ScanProgress {
    pub dirs: u64,
    pub documents: u64,
}

impl Workspace {
    /// Reconciles the whole workspace. Blocking: the app runs it on its own
    /// thread, and `progress` receives the totals after each directory.
    pub fn full_scan(
        &self,
        options: ScanOptions,
        progress: &mut dyn FnMut(ScanProgress),
    ) -> Result<ReconcileSummary> {
        let (mut summary, mut totals) = (ReconcileSummary::default(), ScanProgress::default());
        let (mut pending, mut visited) = (vec![None], HashSet::new());
        while let Some(dir) = pending.pop() {
            let (dir_summary, seen) = self.scan_dir(dir.as_ref(), options)?;
            summary += dir_summary;
            totals.dirs += 1;
            totals.documents += seen;
            progress(totals);
            pending.extend(self.folders_in(dir.as_ref())?.into_iter().map(Some));
            visited.insert(dir);
        }
        summary.missing += self.mark_unvisited_missing(&visited)?;
        self.remove_orphan_dirs()?;
        Ok(summary)
    }

    fn scan_dir(
        &self,
        dir: Option<&RelativePath>,
        options: ScanOptions,
    ) -> Result<(ReconcileSummary, u64)> {
        if options.remove_temp_files {
            self.remove_temp_files(dir)?;
        }
        self.reconcile(dir)
    }

    /// Subdirectories to walk into. Another workspace inside this one shows
    /// as a blocked folder and is never entered (spec 03).
    fn folders_in(&self, dir: Option<&RelativePath>) -> Result<Vec<RelativePath>> {
        let children = self.list_dirs(dir)?.into_iter();
        let folders = children.filter(|child| child.kind == DirKind::Folder);
        Ok(folders.map(|child| child.path).collect())
    }

    /// Documents whose directory was not walked (deleted, renamed or now
    /// hidden) are missing too: no reconciliation reached them.
    fn mark_unvisited_missing(&self, visited: &HashSet<Option<RelativePath>>) -> Result<u64> {
        let now = self.clock.now_millis();
        let unvisited: Vec<_> = self
            .documents_inside(None)?
            .into_iter()
            .filter(|doc| !visited.contains(&doc.path.parent()))
            .collect();
        let mut conn = self.conn();
        for doc in &unvisited {
            mark_missing(&mut conn, doc.id, now).map_err(|e| self.db_error(e))?;
        }
        Ok(unvisited.len() as u64)
    }

    /// Icons and favorites tied to a directory that no longer exists (renamed
    /// or deleted outside Folio) are dropped (specs 03, 05 and 06).
    fn remove_orphan_dirs(&self) -> Result<()> {
        for table in [PathTable::Icons, PathTable::Favorites] {
            self.remove_orphans_in(table)
                .map_err(|e| self.db_error(e))?;
        }
        Ok(())
    }

    fn remove_orphans_in(&self, table: PathTable) -> rusqlite::Result<()> {
        let conn = self.conn();
        let paths = table_paths(&conn, table)?;
        let orphans = paths
            .iter()
            .filter(|path| !path.resolve(self.root()).is_dir());
        for path in orphans {
            delete_path(&conn, table, path)?;
        }
        Ok(())
    }

    /// Deletes the `.folio-tmp-*` files directly in `dir`. They are Folio's
    /// own leftovers (spec 01), never user files.
    fn remove_temp_files(&self, dir: Option<&RelativePath>) -> Result<()> {
        for path in file_paths(self.root(), dir)? {
            let name = os_bytes::as_bytes(path.file_name());
            if !name.starts_with(HIDDEN_TEMP_PREFIX.as_bytes()) {
                continue;
            }
            let full = path.resolve(self.root());
            fs::remove_file(&full).map_err(|e| Error::io(&full, e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    const OPENING: ScanOptions = ScanOptions {
        remove_temp_files: true,
    };

    fn scan(workspace: &Workspace) -> ReconcileSummary {
        workspace.full_scan(OPENING, &mut |_| {}).unwrap()
    }

    fn write_all(workspace: &Workspace, paths: &[&str]) {
        for path in paths {
            write_doc(workspace, path, b"texto");
        }
    }

    fn all_paths(workspace: &Workspace) -> Vec<String> {
        let docs = workspace.documents_inside(None).unwrap();
        docs.into_iter().map(|doc| doc.path.display()).collect()
    }

    #[test]
    fn full_scan_indexes_whole_tree() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        for path in [
            "leia-me.txt",
            "Faculdade/TCC/Trabalho.pdf",
            "Trabalho/Contrato.md",
        ] {
            write_doc(&workspace, path, b"texto");
        }

        let summary = scan(&workspace);

        assert_eq!(summary.added, 3);
        let expected = [
            "Faculdade/TCC/Trabalho.pdf",
            "Trabalho/Contrato.md",
            "leia-me.txt",
        ];
        assert_eq!(all_paths(&workspace), expected);
    }

    #[test]
    fn full_scan_skips_hidden_dirs_symlinked_dirs_and_nested_workspaces() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, ".git/notas.md", b"oculto");
        write_doc(&workspace, "Copiado/.folio/folio.db", b"outro workspace");
        write_doc(&workspace, "Copiado/Dele.md", b"de outro workspace");
        let outside = env.dir("Fora");
        std::fs::write(outside.join("Fora.md"), b"fora").unwrap();
        std::os::unix::fs::symlink(&outside, workspace.root().join("Atalho")).unwrap();
        write_doc(&workspace, "Faculdade/Meu.md", b"meu");

        scan(&workspace);

        assert_eq!(all_paths(&workspace), ["Faculdade/Meu.md"]);
    }

    #[test]
    fn full_scan_removes_leftover_hidden_temp_files_when_opening() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let temp = "Faculdade/.folio-tmp-a1b2c3";
        let user = "Faculdade/.oculto-do-usuario";
        write_all(&workspace, &[temp, user]);

        scan(&workspace);

        let exists = |path: &str| workspace.root().join(path).exists();
        assert_eq!((exists(temp), exists(user)), (false, true));
    }

    #[test]
    fn full_scan_keeps_temp_files_when_not_opening() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(
            &workspace,
            "Faculdade/.folio-tmp-a1b2c3",
            b"conversao em andamento",
        );
        let later = ScanOptions {
            remove_temp_files: false,
        };

        workspace.full_scan(later, &mut |_| {}).unwrap();

        assert!(
            workspace
                .root()
                .join("Faculdade/.folio-tmp-a1b2c3")
                .exists()
        );
    }

    #[test]
    fn full_scan_removes_orphan_icons() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        env.dir("Folio/Faculdade");
        env.dir("Folio/Trabalho");
        for name in ["Faculdade", "Trabalho"] {
            let path = RelativePath::from_bytes(name).unwrap();
            workspace.set_icon(&path, Some("folder")).unwrap();
        }
        std::fs::remove_dir(workspace.root().join("Trabalho")).unwrap();

        scan(&workspace);

        let count: i64 = workspace
            .conn()
            .query_row("SELECT count(*) FROM icons", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn full_scan_reports_progress() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_all(
            &workspace,
            &[
                "a.md",
                "Faculdade/b.md",
                "Faculdade/TCC/c.md",
                "Faculdade/TCC/d.md",
            ],
        );
        let mut reports = Vec::new();

        workspace
            .full_scan(OPENING, &mut |progress| reports.push(progress))
            .unwrap();

        let last = ScanProgress {
            dirs: 3,
            documents: 4,
        };
        assert_eq!((reports.len(), reports.last()), (3, Some(&last)));
    }

    #[test]
    fn full_scan_marks_documents_of_vanished_dir_missing() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        write_doc(&workspace, "Faculdade/TCC/Trabalho.pdf", b"%PDF-1.7\n");
        write_doc(&workspace, "Trabalho/Contrato.md", b"# contrato");
        scan(&workspace);
        std::fs::remove_dir_all(workspace.root().join("Faculdade")).unwrap();

        let summary = scan(&workspace);

        assert_eq!(summary.missing, 1);
        assert_eq!(all_paths(&workspace), ["Trabalho/Contrato.md"]);
    }

    #[test]
    fn deleting_database_and_reopening_reindexes_everything() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        for path in [
            "leia-me.txt",
            "Faculdade/TCC/Trabalho.pdf",
            "Trabalho/Contrato.md",
        ] {
            write_doc(&workspace, path, b"texto");
        }
        scan(&workspace);
        let folio = workspace.root().join(".folio");
        drop(workspace);
        for name in ["folio.db", "folio.db-wal", "folio.db-shm"] {
            let _ = std::fs::remove_file(folio.join(name));
        }

        let reopened = env.open_workspace();
        let summary = scan(&reopened);

        assert_eq!(summary.added, 3);
        assert_eq!(all_paths(&reopened).len(), 3);
    }
}
