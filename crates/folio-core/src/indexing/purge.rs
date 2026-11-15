//! Missing documents leave the index after 30 days (spec 05, "Ausentes").

use crate::Result;
use crate::workspace::Workspace;

/// How long a missing document waits for its file to come back.
pub const MISSING_RETENTION_MS: i64 = 30 * 24 * 60 * 60 * 1000;

impl Workspace {
    /// Deletes documents missing for longer than [`MISSING_RETENTION_MS`],
    /// with what is tied to them (`ON DELETE CASCADE`). Files are never
    /// touched. Returns how many left the index.
    pub fn purge_expired_missing(&self) -> Result<u64> {
        let cutoff = self.clock.now_millis() - MISSING_RETENTION_MS;
        let purged = self
            .conn()
            .execute("DELETE FROM documents WHERE missing_since < ?1", [cutoff])
            .map_err(|e| self.db_error(e))?;
        Ok(purged as u64)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    const DAY: i64 = 24 * 60 * 60 * 1000;

    fn indexed(workspace: &Workspace, path: &str) -> Uuid {
        let path = write_doc(workspace, path, b"%PDF-1.7\n");
        workspace.index_file(&path).unwrap().unwrap().id
    }

    fn count(workspace: &Workspace, table: &str) -> i64 {
        let sql = format!("SELECT count(*) FROM {table}");
        workspace
            .conn()
            .query_row(&sql, [], |row| row.get(0))
            .unwrap()
    }

    /// `source` converted to `target`, as spec 08 records it.
    fn link(workspace: &Workspace, source: Uuid, target: Uuid) {
        let sql = "INSERT INTO conversions VALUES (?1, ?2, ?3, 'pdf', 'pdf', 0, 0, 0)";
        let id = workspace.ids.new_id().to_string();
        workspace
            .conn()
            .execute(sql, (id, source.to_string(), target.to_string()))
            .unwrap();
    }

    #[test]
    fn purge_removes_documents_missing_for_more_than_30_days_with_their_links() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let (gone, kept) = (
            indexed(&workspace, "Velho.pdf"),
            indexed(&workspace, "Novo.pdf"),
        );
        link(&workspace, gone, kept);
        workspace.trash_document(gone).unwrap();
        env.clock.set(1_000 + 31 * DAY);

        let purged = workspace.purge_expired_missing().unwrap();

        assert_eq!(purged, 1);
        assert_eq!(
            (
                count(&workspace, "documents"),
                count(&workspace, "conversions")
            ),
            (1, 0)
        );
    }

    #[test]
    fn purge_keeps_documents_missing_for_less_than_30_days() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let gone = indexed(&workspace, "Velho.pdf");
        workspace.trash_document(gone).unwrap();
        env.clock.set(1_000 + 29 * DAY);

        assert_eq!(workspace.purge_expired_missing().unwrap(), 0);
        assert_eq!(count(&workspace, "documents"), 1);
    }

    #[test]
    fn opening_workspace_purges_expired_missing_documents() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let gone = indexed(&workspace, "Velho.pdf");
        workspace.trash_document(gone).unwrap();
        drop(workspace);
        env.clock.set(1_000 + 31 * DAY);

        let reopened = env.open_workspace();

        assert_eq!(count(&reopened, "documents"), 0);
    }
}
