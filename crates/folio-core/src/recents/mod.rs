//! Documents opened in the Reader of this workspace (spec 06, "Recentes").

mod position;

pub use position::ReadingPosition;

use rusqlite::{Connection, Row};
use uuid::Uuid;

use crate::Result;
use crate::db::columns::uuid_column;
use crate::paths::RelativePath;
use crate::workspace::Workspace;

/// A recently opened document, as the sidebar and the Home show it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecentDocument {
    pub id: Uuid,
    /// File name; invalid UTF-8 bytes show as U+FFFD.
    pub name: String,
    /// Name of the project it is in; `None` at the root.
    pub project: Option<String>,
    pub size: u64,
    /// When it was last opened, in ms UTC.
    pub opened_at: i64,
}

impl Workspace {
    /// Records that the Reader showed the document `id` now. An invalid
    /// document is never shown, so it does not count.
    pub fn record_opened(&self, id: Uuid) -> Result<()> {
        if !self.document(id)?.valid {
            return Ok(());
        }
        let now = self.clock.now_millis();
        record(&mut self.conn(), id, now).map_err(|e| self.db_error(e))
    }

    /// The `limit` most recently opened documents, newest first.
    pub fn recent_documents(&self, limit: usize) -> Result<Vec<RecentDocument>> {
        let conn = self.conn();
        let query = || -> rusqlite::Result<Vec<RecentDocument>> {
            conn.prepare(LIST)?
                .query_map([limit as i64], recent_row)?
                .collect()
        };
        query().map_err(|e| self.db_error(e))
    }
}

/// How many documents the list keeps (spec 06).
pub const MAX_RECENTS: usize = 100;

/// Records the opening and drops the oldest past [`MAX_RECENTS`], in one
/// transaction.
fn record(conn: &mut Connection, id: Uuid, now: i64) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    tx.execute(RECORD, (id.to_string(), now))?;
    tx.execute(TRIM, [MAX_RECENTS as i64])?;
    tx.commit()
}

const TRIM: &str = "DELETE FROM recent_documents WHERE document_id NOT IN
     (SELECT document_id FROM recent_documents ORDER BY opened_at DESC LIMIT ?1)";

/// Opening again only moves the document up, keeping its position.
const RECORD: &str = "INSERT INTO recent_documents (document_id, opened_at) VALUES (?1, ?2)
     ON CONFLICT (document_id) DO UPDATE SET opened_at = excluded.opened_at";

const LIST: &str = "SELECT d.id, d.path, d.size, r.opened_at
     FROM recent_documents r JOIN documents d ON d.id = r.document_id
     WHERE d.missing_since IS NULL
     ORDER BY r.opened_at DESC LIMIT ?1";

fn recent_row(row: &Row<'_>) -> rusqlite::Result<RecentDocument> {
    let path: RelativePath = row.get(1)?;
    Ok(RecentDocument {
        id: uuid_column(row, 0)?,
        name: path.file_name().to_string_lossy().into_owned(),
        project: project_name(&path),
        size: row.get::<_, i64>(2)? as u64,
        opened_at: row.get(3)?,
    })
}

/// The first component of `path`, when the document is inside a project.
fn project_name(path: &RelativePath) -> Option<String> {
    let bytes = path.as_bytes();
    let end = bytes.iter().position(|byte| *byte == b'/')?;
    Some(String::from_utf8_lossy(&bytes[..end]).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::indexing::MISSING_RETENTION_MS;
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str) -> Uuid {
        let path = write_doc(workspace, path, b"text");
        workspace.index_file(&path).unwrap().unwrap().id
    }

    /// Opens `id` at `millis`.
    fn opened(env: &TestEnv, workspace: &Workspace, id: Uuid, millis: i64) {
        env.clock.set(millis);
        workspace.record_opened(id).unwrap();
    }

    fn recent_names(workspace: &Workspace) -> Vec<String> {
        let recents = workspace.recent_documents(10).unwrap();
        recents.into_iter().map(|doc| doc.name).collect()
    }

    #[test]
    fn recording_open_puts_document_at_top_of_recents() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let (a, b) = (indexed(&workspace, "A.txt"), indexed(&workspace, "B.txt"));

        opened(&env, &workspace, a, 2_000);
        opened(&env, &workspace, b, 3_000);

        assert_eq!(recent_names(&workspace), ["B.txt", "A.txt"]);
    }

    #[test]
    fn reopening_document_moves_it_to_top_without_duplicating() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let (a, b) = (indexed(&workspace, "A.txt"), indexed(&workspace, "B.txt"));
        opened(&env, &workspace, a, 2_000);
        opened(&env, &workspace, b, 3_000);

        opened(&env, &workspace, a, 4_000);

        assert_eq!(recent_names(&workspace), ["A.txt", "B.txt"]);
    }

    #[test]
    fn recents_keep_only_100_most_recent() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let ids: Vec<Uuid> = (0..101)
            .map(|n| indexed(&workspace, &format!("Nota {n}.txt")))
            .collect();

        for (n, id) in (0..).zip(&ids) {
            opened(&env, &workspace, *id, 2_000 + n);
        }

        let kept: Vec<Uuid> = workspace
            .recent_documents(200)
            .unwrap()
            .into_iter()
            .map(|doc| doc.id)
            .collect();
        assert_eq!((kept.len(), kept.contains(&ids[0])), (100, false));
    }

    #[test]
    fn recents_return_requested_count_with_id_name_project_size_and_opened_at() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let root_doc = indexed(&workspace, "Solto.txt");
        let nested = indexed(&workspace, "Faculdade/Prova/Nota.txt");
        opened(&env, &workspace, root_doc, 2_000);
        opened(&env, &workspace, nested, 3_000);

        let recents = workspace.recent_documents(1).unwrap();

        let expected = RecentDocument {
            id: nested,
            name: "Nota.txt".to_string(),
            project: Some("Faculdade".to_string()),
            size: 4,
            opened_at: 3_000,
        };
        assert_eq!(recents, [expected]);
    }

    #[test]
    fn recording_open_of_invalid_document_is_ignored() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        // Not a PDF inside: indexed as invalid (spec 04).
        let invalid = indexed(&workspace, "Fatura.pdf");

        opened(&env, &workspace, invalid, 2_000);

        assert_eq!(recent_names(&workspace), Vec::<String>::new());
    }

    #[test]
    fn recents_hide_missing_documents() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let (a, b) = (indexed(&workspace, "A.txt"), indexed(&workspace, "B.txt"));
        opened(&env, &workspace, a, 2_000);
        opened(&env, &workspace, b, 3_000);

        workspace.trash_document(b).unwrap();

        assert_eq!(recent_names(&workspace), ["A.txt"]);
    }

    #[test]
    fn recent_leaves_with_purged_document() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let doc = indexed(&workspace, "A.txt");
        opened(&env, &workspace, doc, 2_000);
        workspace.trash_document(doc).unwrap();
        env.clock.set(2_000 + MISSING_RETENTION_MS + 1);

        workspace.purge_expired_missing().unwrap();

        let rows: i64 = workspace
            .conn()
            .query_row("SELECT count(*) FROM recent_documents", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(rows, 0);
    }
}
