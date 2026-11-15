//! Recognizing a file renamed or moved outside Folio as the document it was,
//! so favorites, recents and conversion links survive (spec 05).

use std::fs;

use rusqlite::{Params, Row};
use uuid::Uuid;

use super::fs_id::{FileIdentity, FsId, is_same_file};
use super::hashing::hash_file;
use crate::Result;
use crate::db::columns::uuid_column;
use crate::documents::file_facts::FileFacts;
use crate::paths::RelativePath;
use crate::workspace::Workspace;

/// An indexed document that a new file might be.
struct Candidate {
    id: Uuid,
    path: RelativePath,
    identity: FileIdentity,
    missing: bool,
    /// BLAKE3, only when it is current (computed for the recorded size and
    /// mtime).
    hash: Option<Vec<u8>>,
}

impl Workspace {
    /// The indexed document the new file at `path` is, if any.
    ///
    /// Tries stage 1 (filesystem identity), then stage 2 (content).
    pub(super) fn find_reconnection(&self, path: &RelativePath) -> Result<Option<Uuid>> {
        let facts = FileFacts::read(&path.resolve(self.root()))?;
        let found = FileIdentity {
            fs_id: facts.fs_id,
            size: facts.size,
            modified_at: facts.modified_at,
        };
        if let Some(id) = self.same_file_by_identity(&found)? {
            return Ok(Some(id));
        }
        self.same_file_by_content(path, facts.size)
    }

    /// Stage 1: same device and inode, size and mtime (`mv`, file managers).
    fn same_file_by_identity(&self, found: &FileIdentity) -> Result<Option<Uuid>> {
        let candidates = self.candidates(
            "fs_dev = ?1 AND fs_ino = ?2",
            (found.fs_id.dev, found.fs_id.ino),
        )?;
        let matching = candidates
            .into_iter()
            .find(|c| is_same_file(&c.identity, found) && self.is_gone(c));
        Ok(matching.map(|c| c.id))
    }

    /// Stage 2: among the gone documents of the same size, the only one with
    /// the new file's hash (moves across filesystems, copy then delete). With
    /// two or more identical ones, none: the new file gets a new id.
    fn same_file_by_content(&self, path: &RelativePath, size: u64) -> Result<Option<Uuid>> {
        let candidates: Vec<_> = self
            .candidates(CURRENT_HASH_WITH_SIZE, [size as i64])?
            .into_iter()
            .filter(|c| self.is_gone(c))
            .collect();
        if candidates.is_empty() {
            return Ok(None);
        }
        let hash = hash_file(&path.resolve(self.root()))?;
        let mut matching = candidates
            .into_iter()
            .filter(|c| c.hash.as_deref() == Some(hash.as_bytes()));
        match (matching.next(), matching.next()) {
            (Some(only), None) => Ok(Some(only.id)),
            _ => Ok(None),
        }
    }

    /// Points the document `id` at `path` and refreshes it from disk; the id
    /// and everything tied to it stay.
    pub(super) fn reconnect(&self, id: Uuid, path: &RelativePath) -> Result<()> {
        self.conn()
            .execute(
                "UPDATE documents SET path = ?2 WHERE id = ?1",
                (id.to_string(), path),
            )
            .map_err(|e| self.db_error(e))?;
        self.index_file(path)?;
        Ok(())
    }

    /// A candidate can only be the new file if its own file is gone.
    fn is_gone(&self, candidate: &Candidate) -> bool {
        candidate.missing || fs::symlink_metadata(candidate.path.resolve(self.root())).is_err()
    }

    fn candidates(&self, condition: &str, params: impl Params) -> Result<Vec<Candidate>> {
        let sql = format!("SELECT {CANDIDATE_COLUMNS} FROM documents WHERE {condition}");
        let conn = self.conn();
        let query = || -> rusqlite::Result<Vec<Candidate>> {
            conn.prepare_cached(&sql)?
                .query_map(params, candidate_row)?
                .collect()
        };
        query().map_err(|e| self.db_error(e))
    }
}

/// Only documents whose hash is current can be compared: filtering here
/// keeps a first scan, when nothing is hashed yet, from reading every
/// document of the same size for each new file.
const CURRENT_HASH_WITH_SIZE: &str =
    "size = ?1 AND hash IS NOT NULL AND hash_size = size AND hash_modified_at = modified_at";

/// The hash column is NULL unless it was computed for the recorded size and
/// mtime.
const CANDIDATE_COLUMNS: &str = "id, path, fs_dev, fs_ino, size, modified_at,
     missing_since IS NOT NULL,
     CASE WHEN hash_size = size AND hash_modified_at = modified_at THEN hash END";

fn candidate_row(row: &Row<'_>) -> rusqlite::Result<Candidate> {
    Ok(Candidate {
        id: uuid_column(row, 0)?,
        path: row.get(1)?,
        identity: FileIdentity {
            fs_id: FsId {
                dev: row.get(2)?,
                ino: row.get(3)?,
            },
            size: row.get::<_, i64>(4)? as u64,
            modified_at: row.get(5)?,
        },
        missing: row.get(6)?,
        hash: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::documents::test_docs::write_doc;
    use crate::paths::RelativePath;
    use crate::workspace::Workspace;
    use crate::workspace::test_env::TestEnv;

    fn rel(text: &str) -> RelativePath {
        RelativePath::from_bytes(text).unwrap()
    }

    /// Writes and reconciles a document, returning its id.
    fn reconciled(workspace: &Workspace, path: &str, content: &[u8]) -> uuid::Uuid {
        let path = write_doc(workspace, path, content);
        workspace.reconcile_dir(path.parent().as_ref()).unwrap();
        workspace
            .list_documents(path.parent().as_ref())
            .unwrap()
            .into_iter()
            .find(|doc| doc.path == path)
            .unwrap()
            .id
    }

    fn mv(workspace: &Workspace, from: &str, to: &str) {
        let (from, to) = (workspace.root().join(from), workspace.root().join(to));
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        fs::rename(from, to).unwrap();
    }

    #[test]
    fn renaming_in_same_dir_outside_folio_keeps_id() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = reconciled(&workspace, "Faculdade/Trabalho.pdf", b"%PDF-1.7\n");
        mv(&workspace, "Faculdade/Trabalho.pdf", "Faculdade/Final.pdf");

        let summary = workspace.reconcile_dir(Some(&rel("Faculdade"))).unwrap();

        assert_eq!(
            workspace.document(id).unwrap().path,
            rel("Faculdade/Final.pdf")
        );
        assert_eq!(
            (summary.reconnected, summary.added, summary.missing),
            (1, 0, 0)
        );
    }

    #[test]
    fn moving_between_dirs_outside_folio_keeps_id() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = reconciled(&workspace, "Faculdade/Trabalho.pdf", b"%PDF-1.7\n");
        mv(&workspace, "Faculdade/Trabalho.pdf", "Arquivo/Trabalho.pdf");

        workspace.reconcile_dir(Some(&rel("Arquivo"))).unwrap();
        let old_dir = workspace.reconcile_dir(Some(&rel("Faculdade"))).unwrap();

        assert_eq!(
            workspace.document(id).unwrap().path,
            rel("Arquivo/Trabalho.pdf")
        );
        assert_eq!(old_dir.missing, 0);
    }

    /// Copy then delete, as a move across filesystems does: new inode and
    /// new mtime, same content.
    fn copy_and_delete(workspace: &Workspace, from: &str, to: &str) {
        let (from, to) = (workspace.root().join(from), workspace.root().join(to));
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        fs::copy(&from, &to).unwrap();
        fs::remove_file(from).unwrap();
    }

    #[test]
    fn moving_across_filesystems_keeps_id_by_hash() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = reconciled(&workspace, "Faculdade/Trabalho.pdf", b"%PDF-1.7\n");
        workspace.hash_stale_documents().unwrap();
        copy_and_delete(&workspace, "Faculdade/Trabalho.pdf", "Arquivo/Trabalho.pdf");

        let summary = workspace.reconcile_dir(Some(&rel("Arquivo"))).unwrap();

        assert_eq!((summary.reconnected, summary.added), (1, 0));
        assert_eq!(
            workspace.document(id).unwrap().path,
            rel("Arquivo/Trabalho.pdf")
        );
    }

    #[test]
    fn editor_save_over_same_path_keeps_id() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = reconciled(&workspace, "notas.md", b"# antes");
        let temp = workspace.root().join(".notas.md.swp");
        fs::write(&temp, b"# depois de salvar").unwrap();
        fs::rename(&temp, workspace.root().join("notas.md")).unwrap();

        let summary = workspace.reconcile_dir(None).unwrap();

        assert_eq!((summary.updated, summary.added), (1, 0));
        assert_eq!(workspace.document(id).unwrap().size, 18);
    }

    /// Reconciles `dir` and hashes, as a scan followed by the hash pass does.
    fn scan(workspace: &Workspace, dir: Option<&RelativePath>) {
        workspace.reconcile_dir(dir).unwrap();
        workspace.hash_stale_documents().unwrap();
    }

    #[test]
    fn two_missing_with_same_hash_are_not_reconnected() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        reconciled(&workspace, "A/um.md", b"# igual");
        reconciled(&workspace, "A/dois.md", b"# igual");
        scan(&workspace, Some(&rel("A")));
        fs::remove_dir_all(workspace.root().join("A")).unwrap();
        fs::create_dir(workspace.root().join("A")).unwrap();
        scan(&workspace, Some(&rel("A")));
        fs::create_dir(workspace.root().join("B")).unwrap();
        fs::write(workspace.root().join("B/tres.md"), b"# igual").unwrap();

        let summary = workspace.reconcile_dir(Some(&rel("B"))).unwrap();

        assert_eq!((summary.reconnected, summary.added), (0, 1));
    }

    #[test]
    fn single_missing_identical_to_new_file_is_reconnected() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        reconciled(&workspace, "A/um.md", b"# igual");
        let gone = reconciled(&workspace, "A/dois.md", b"# igual");
        scan(&workspace, Some(&rel("A")));
        fs::remove_file(workspace.root().join("A/dois.md")).unwrap();
        scan(&workspace, Some(&rel("A")));
        fs::create_dir(workspace.root().join("B")).unwrap();
        fs::write(workspace.root().join("B/tres.md"), b"# igual").unwrap();

        workspace.reconcile_dir(Some(&rel("B"))).unwrap();

        assert_eq!(workspace.document(gone).unwrap().path, rel("B/tres.md"));
    }

    #[test]
    fn trashing_and_restoring_keeps_id() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = reconciled(&workspace, "Faculdade/Trabalho.pdf", b"%PDF-1.7\n");
        workspace.trash_document(id).unwrap();
        let trash = env.dir("Lixeira").join("Trabalho.pdf");
        fs::rename(workspace.root().join("Faculdade/Trabalho.pdf"), &trash).unwrap();
        workspace.reconcile_dir(Some(&rel("Faculdade"))).unwrap();
        fs::rename(&trash, workspace.root().join("Faculdade/Trabalho.pdf")).unwrap();

        let summary = workspace.reconcile_dir(Some(&rel("Faculdade"))).unwrap();

        assert_eq!((summary.restored, summary.added), (1, 0));
        assert_eq!(
            workspace.list_documents(Some(&rel("Faculdade"))).unwrap()[0].id,
            id
        );
    }
}
