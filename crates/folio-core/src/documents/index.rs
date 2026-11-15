//! The document index: what Folio records about each document (spec 04).

use std::ffi::OsStr;

use rusqlite::{Connection, OptionalExtension, Row};
use uuid::Uuid;

use super::file_facts::FileFacts;
use super::{Format, check_signature, is_ignored_name};
use crate::db::columns::uuid_column;
use crate::indexing::fs_id::FsId;
use crate::paths::RelativePath;
use crate::workspace::Workspace;
use crate::{Error, Result};

/// A document as the index knows it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Document {
    pub id: Uuid,
    pub path: RelativePath,
    /// File name for the UI; invalid UTF-8 bytes show as U+FFFD.
    pub name: String,
    pub format: Format,
    pub valid: bool,
    pub protected: bool,
    pub size: u64,
    /// Filesystem mtime, in ms since the Unix epoch.
    pub modified_at: i64,
    /// Filesystem birth time, when the filesystem has one.
    pub created_at: Option<i64>,
}

impl Workspace {
    /// Recognizes the file at `path` and records it in the index, keeping its
    /// id when it is already there. `None` when it is not a document.
    pub fn index_file(&self, path: &RelativePath) -> Result<Option<Document>> {
        let Some(format) = document_format(path.file_name()) else {
            return Ok(None);
        };
        let full = path.resolve(self.root());
        let signature = check_signature(format, &full);
        let facts = FileFacts::read(&full)?;
        let doc = Document {
            id: self.ids.new_id(),
            path: path.clone(),
            name: path.file_name().to_string_lossy().into_owned(),
            format,
            valid: signature.valid,
            protected: signature.protected,
            size: facts.size,
            modified_at: facts.modified_at,
            created_at: facts.created_at,
        };
        let id = upsert(&self.conn(), &doc, facts.fs_id).map_err(|e| self.db_error(e))?;
        Ok(Some(Document { id, ..doc }))
    }

    /// Documents directly inside `parent` (the root for `None`, where the
    /// ones without a project live), sorted by name.
    pub fn list_documents(&self, parent: Option<&RelativePath>) -> Result<Vec<Document>> {
        let inside = self.documents_inside(parent)?;
        Ok(inside
            .into_iter()
            .filter(|doc| doc.path.parent().as_ref() == parent)
            .collect())
    }

    /// Present documents anywhere inside `dir` (the whole workspace for
    /// `None`), sorted by path.
    pub(crate) fn documents_inside(&self, dir: Option<&RelativePath>) -> Result<Vec<Document>> {
        // At the root there is no upper bound: any byte may start a path.
        let (lower, upper) = dir.map_or((Vec::new(), None), |dir| {
            let (lower, upper) = dir.descendant_bounds();
            (lower, Some(upper))
        });
        let sql = format!(
            "SELECT {COLUMNS} FROM documents WHERE missing_since IS NULL
             AND path > ?1 AND (?2 IS NULL OR path < ?2) ORDER BY path"
        );
        let conn = self.conn();
        let query = || -> rusqlite::Result<Vec<Document>> {
            conn.prepare(&sql)?
                .query_map((lower, upper), document_row)?
                .collect()
        };
        query().map_err(|e| self.db_error(e))
    }

    /// The indexed document with `id`.
    pub fn document(&self, id: Uuid) -> Result<Document> {
        let sql = format!("SELECT {COLUMNS} FROM documents WHERE id = ?1");
        self.conn()
            .query_row(&sql, [id.to_string()], document_row)
            .optional()
            .map_err(|e| self.db_error(e))?
            .ok_or(Error::DocumentNotFound(id))
    }
}

/// The format of a file that appears as a document, if any.
pub(crate) fn document_format(name: &OsStr) -> Option<Format> {
    if is_ignored_name(name) {
        return None;
    }
    Format::from_name(name)
}

/// Columns read by [`document_row`], in order.
pub(crate) const COLUMNS: &str =
    "id, path, format, valid, protected, size, modified_at, created_at";

pub(crate) fn document_row(row: &Row<'_>) -> rusqlite::Result<Document> {
    let path: RelativePath = row.get(1)?;
    Ok(Document {
        id: uuid_column(row, 0)?,
        name: path.file_name().to_string_lossy().into_owned(),
        path,
        format: row.get(2)?,
        valid: row.get(3)?,
        protected: row.get(4)?,
        size: row.get::<_, i64>(5)? as u64,
        modified_at: row.get(6)?,
        created_at: row.get(7)?,
    })
}

/// Inserts or updates the row for `doc.path` and returns the id it keeps.
fn upsert(conn: &Connection, doc: &Document, fs_id: FsId) -> rusqlite::Result<Uuid> {
    let params = (
        doc.id.to_string(),
        &doc.path,
        doc.format,
        doc.valid,
        doc.protected,
        doc.size as i64,
        doc.modified_at,
        doc.created_at,
        fs_id.dev,
        fs_id.ino,
    );
    conn.prepare_cached(UPSERT)?
        .query_row(params, |row| uuid_column(row, 0))
}

/// A file found again at an indexed path keeps its id and stops being missing.
const UPSERT: &str = "INSERT INTO documents
         (id, path, format, valid, protected, size, modified_at, created_at, fs_dev, fs_ino)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
     ON CONFLICT (path) DO UPDATE SET
         format = excluded.format, valid = excluded.valid,
         protected = excluded.protected, size = excluded.size,
         modified_at = excluded.modified_at, created_at = excluded.created_at,
         fs_dev = excluded.fs_dev, fs_ino = excluded.fs_ino, missing_since = NULL
     RETURNING id";

/// Marks the document missing since `since`, unless it already is.
pub(crate) fn mark_missing(conn: &mut Connection, id: Uuid, since: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE documents SET missing_since = ?2 WHERE id = ?1 AND missing_since IS NULL",
        (id.to_string(), since),
    )
    .map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_docs::{MTIME, write_doc};
    use crate::workspace::test_env::TestEnv;

    #[test]
    fn indexing_new_file_assigns_id_and_records_filesystem_data() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let path = write_doc(&workspace, "Faculdade/Trabalho.pdf", b"%PDF-1.7\n");

        let doc = workspace.index_file(&path).unwrap().unwrap();

        assert_eq!(
            (doc.path, doc.name.as_str(), doc.format),
            (path, "Trabalho.pdf", Format::Pdf)
        );
        assert_eq!(
            (doc.valid, doc.protected, doc.size, doc.modified_at),
            (true, false, 9, MTIME)
        );
        assert_ne!(doc.id, workspace.id());
    }

    #[test]
    fn reindexing_same_path_keeps_id_and_updates_data() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let path = write_doc(&workspace, "notas.md", b"# antes");
        let first = workspace.index_file(&path).unwrap().unwrap();
        write_doc(&workspace, "notas.md", b"# depois, mais longo");

        let second = workspace.index_file(&path).unwrap().unwrap();

        assert_eq!((second.id, second.size), (first.id, 20));
    }

    #[test]
    fn document_is_found_by_id() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let path = write_doc(&workspace, "Trabalho/Contrato.txt", b"texto");
        let indexed = workspace.index_file(&path).unwrap().unwrap();

        let found = workspace.document(indexed.id).unwrap();

        assert_eq!(found, indexed);
    }

    #[test]
    fn unknown_id_fails_with_document_not_found() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let unknown = Uuid::from_u128(999);

        let result = workspace.document(unknown);

        assert!(matches!(result, Err(crate::Error::DocumentNotFound(id)) if id == unknown));
    }

    fn index_all(workspace: &Workspace, paths: &[&str]) {
        for path in paths {
            let path = write_doc(workspace, path, b"texto");
            workspace.index_file(&path).unwrap();
        }
    }

    fn names(docs: &[Document]) -> Vec<&str> {
        docs.iter().map(|doc| doc.name.as_str()).collect()
    }

    #[test]
    fn listing_documents_returns_those_directly_in_dir() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let paths = [
            "leia-me.txt",
            "Fac/b.md",
            "Fac/a.md",
            "Fac/TCC/c.md",
            "Faculdade/d.md",
        ];
        index_all(&workspace, &paths);
        let fac = RelativePath::from_bytes("Fac").unwrap();

        let at_root = workspace.list_documents(None).unwrap();
        let in_fac = workspace.list_documents(Some(&fac)).unwrap();

        assert_eq!(names(&at_root), ["leia-me.txt"]);
        assert_eq!(names(&in_fac), ["a.md", "b.md"]);
    }

    #[test]
    fn listing_includes_invalid_documents_marked_invalid() {
        let env = TestEnv::new();
        let workspace = env.open_workspace();
        let path = write_doc(&workspace, "Boleto.pdf", b"<html>erro</html>");
        workspace.index_file(&path).unwrap();

        let docs = workspace.list_documents(None).unwrap();

        assert_eq!((names(&docs), docs[0].valid), (vec!["Boleto.pdf"], false));
    }

    fn sample_document() -> Document {
        Document {
            id: Uuid::from_u128(1),
            path: RelativePath::from_bytes("A/Nota.pdf").unwrap(),
            name: "Nota.pdf".to_string(),
            format: Format::Pdf,
            valid: true,
            protected: false,
            size: 10,
            modified_at: 5,
            created_at: None,
        }
    }

    #[test]
    fn document_serializes_with_string_id_and_camel_case_fields() {
        assert_eq!(
            serde_json::to_value(sample_document()).unwrap(),
            serde_json::json!({
                "id": "00000000-0000-0000-0000-000000000001",
                "path": [65, 47, 78, 111, 116, 97, 46, 112, 100, 102],
                "name": "Nota.pdf",
                "format": "pdf",
                "valid": true,
                "protected": false,
                "size": 10,
                "modifiedAt": 5,
                "createdAt": null,
            })
        );
    }
}
