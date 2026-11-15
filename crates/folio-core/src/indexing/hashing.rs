//! Content hashes, used only for reconnection (spec 05, "Hash").

use std::fs::File;
use std::path::Path;

use rusqlite::Row;
use uuid::Uuid;

use crate::db::columns::uuid_column;
use crate::paths::RelativePath;
use crate::workspace::Workspace;
use crate::{Error, Result};

/// A document whose hash must be computed, with the facts it is computed for.
struct ToHash {
    id: Uuid,
    path: RelativePath,
    size: i64,
    modified_at: i64,
}

impl Workspace {
    /// Hashes every present document whose hash is missing or stale (its
    /// `hash_of` no longer matches size and mtime). Meant to run in the
    /// background after a scan. Returns how many were hashed.
    pub fn hash_stale_documents(&self) -> Result<u64> {
        let mut hashed = 0;
        for doc in &self.documents_to_hash()? {
            // Read without holding the database lock: hashing is the slow part.
            match hash_file(&doc.path.resolve(self.root())) {
                Ok(hash) => self.store_hash(doc, &hash)?,
                // A broken symlink or a file gone since the scan: the next
                // scan sorts it out; the other documents still get hashed.
                Err(error) => {
                    log::debug!("skipping hash: {error}");
                    continue;
                }
            }
            hashed += 1;
        }
        Ok(hashed)
    }

    fn documents_to_hash(&self) -> Result<Vec<ToHash>> {
        let sql = "SELECT id, path, size, modified_at FROM documents
                   WHERE missing_since IS NULL AND (hash IS NULL
                       OR hash_size IS NOT size OR hash_modified_at IS NOT modified_at)";
        let conn = self.conn();
        let query = || -> rusqlite::Result<Vec<ToHash>> {
            conn.prepare(sql)?.query_map([], to_hash_row)?.collect()
        };
        query().map_err(|e| self.db_error(e))
    }

    /// `hash_of` records the size and mtime the hash was computed for.
    fn store_hash(&self, doc: &ToHash, hash: &blake3::Hash) -> Result<()> {
        self.conn()
            .execute(
                "UPDATE documents SET hash = ?2, hash_size = ?3, hash_modified_at = ?4 WHERE id = ?1",
                (doc.id.to_string(), hash.as_bytes().as_slice(), doc.size, doc.modified_at),
            )
            .map(drop)
            .map_err(|e| self.db_error(e))
    }
}

fn to_hash_row(row: &Row<'_>) -> rusqlite::Result<ToHash> {
    Ok(ToHash {
        id: uuid_column(row, 0)?,
        path: row.get(1)?,
        size: row.get(2)?,
        modified_at: row.get(3)?,
    })
}

/// BLAKE3 of the file at `path`, following symlinks.
pub(crate) fn hash_file(path: &Path) -> Result<blake3::Hash> {
    let file = File::open(path).map_err(|e| Error::io(path, e))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update_reader(file).map_err(|e| Error::io(path, e))?;
    Ok(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use rusqlite::OptionalExtension;
    use uuid::Uuid;

    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    /// The stored hash of `id`, if any.
    fn stored_hash(workspace: &Workspace, id: Uuid) -> Option<Vec<u8>> {
        workspace
            .conn()
            .query_row(
                "SELECT hash FROM documents WHERE id = ?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .unwrap()
            .flatten()
    }

    fn indexed(workspace: &Workspace, path: &str, content: &[u8]) -> Uuid {
        let path = write_doc(workspace, path, content);
        workspace.index_file(&path).unwrap().unwrap().id
    }

    #[test]
    fn hashing_computes_hash_of_documents_without_one() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "Trabalho.pdf", b"%PDF-1.7\n");

        let hashed = workspace.hash_stale_documents().unwrap();

        assert_eq!(hashed, 1);
        assert_eq!(
            stored_hash(&workspace, id),
            Some(blake3::hash(b"%PDF-1.7\n").as_bytes().to_vec())
        );
    }

    #[test]
    fn hashing_skips_documents_with_current_hash() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        indexed(&workspace, "Trabalho.pdf", b"%PDF-1.7\n");
        workspace.hash_stale_documents().unwrap();

        assert_eq!(workspace.hash_stale_documents().unwrap(), 0);
    }

    #[test]
    fn hashing_recomputes_stale_hash_after_edit() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "notas.md", b"# antes");
        workspace.hash_stale_documents().unwrap();
        write_doc(&workspace, "notas.md", b"# depois de editar");
        workspace.reconcile_dir(None).unwrap();

        let hashed = workspace.hash_stale_documents().unwrap();

        assert_eq!(hashed, 1);
        assert_eq!(
            stored_hash(&workspace, id),
            Some(blake3::hash(b"# depois de editar").as_bytes().to_vec())
        );
    }

    #[test]
    fn invalid_documents_are_hashed_too() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let id = indexed(&workspace, "download.pdf", b"%PD");

        workspace.hash_stale_documents().unwrap();

        assert_eq!(
            stored_hash(&workspace, id),
            Some(blake3::hash(b"%PD").as_bytes().to_vec())
        );
    }

    #[test]
    fn hashing_skips_unreadable_files_and_hashes_the_rest() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let link = workspace.root().join("Atalho.pdf");
        std::os::unix::fs::symlink(workspace.root().join("sumiu.pdf"), &link).unwrap();
        workspace
            .index_file(&crate::paths::RelativePath::from_bytes("Atalho.pdf").unwrap())
            .unwrap();
        let id = indexed(&workspace, "notas.md", b"# notas");

        let hashed = workspace.hash_stale_documents().unwrap();

        assert_eq!(hashed, 1);
        assert!(stored_hash(&workspace, id).is_some());
    }
}
