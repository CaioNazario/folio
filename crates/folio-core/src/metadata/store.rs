//! The `document_metadata` table (spec 04): one row per document, valid
//! while the file keeps the size and mtime it had when extracted.

use rusqlite::{Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use crate::documents::file_facts::FileFacts;
use crate::model::Metadata;

/// The cached metadata of `id`, when it was extracted from the file as it is
/// now (`facts`).
pub fn load(conn: &Connection, id: Uuid, facts: &FileFacts) -> rusqlite::Result<Option<Metadata>> {
    let stamp = (id.to_string(), facts.size as i64, facts.modified_at);
    conn.query_row(LOAD, stamp, metadata_row).optional()
}

const LOAD: &str = "SELECT title, author, subject, keywords, doc_created_at, doc_modified_at,
     pages, extra FROM document_metadata
     WHERE document_id = ?1 AND extracted_size = ?2 AND extracted_modified_at = ?3";

fn metadata_row(row: &Row<'_>) -> rusqlite::Result<Metadata> {
    Ok(Metadata {
        title: row.get(0)?,
        author: row.get(1)?,
        subject: row.get(2)?,
        keywords: from_json(row.get(3)?),
        created_at: row.get(4)?,
        modified_at: row.get(5)?,
        pages: row.get(6)?,
        extra: from_json(row.get(7)?),
    })
}

/// Replaces the cached metadata of `id`, stamped with the file it came from.
pub fn save(
    conn: &Connection,
    id: Uuid,
    facts: &FileFacts,
    metadata: &Metadata,
) -> rusqlite::Result<()> {
    conn.execute(
        SAVE,
        params![
            id.to_string(),
            metadata.title,
            metadata.author,
            metadata.subject,
            to_json(&metadata.keywords),
            metadata.created_at,
            metadata.modified_at,
            metadata.pages,
            to_json(&metadata.extra),
            facts.size as i64,
            facts.modified_at,
        ],
    )?;
    Ok(())
}

const SAVE: &str = "INSERT OR REPLACE INTO document_metadata (document_id, title, author,
     subject, keywords, doc_created_at, doc_modified_at, pages, extra, extracted_size,
     extracted_modified_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)";

/// Serializing strings and JSON values cannot fail.
fn to_json(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// The cache is rebuilt from the files, so an unreadable column is empty.
fn from_json<T: serde::de::DeserializeOwned + Default>(text: Option<String>) -> T {
    text.and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}
