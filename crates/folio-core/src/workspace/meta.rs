//! The workspace's own identity, kept in its database (table `meta`).

use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

use crate::clock::Clock;
use crate::db::columns::uuid_column;
use crate::ids::IdGenerator;

/// Identity of a workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Meta {
    pub id: Uuid,
    pub created_at: i64,
}

/// The workspace's meta row, created on first open.
pub fn load_or_create_meta(
    conn: &Connection,
    clock: &dyn Clock,
    ids: &dyn IdGenerator,
) -> rusqlite::Result<Meta> {
    if let Some(meta) = load_meta(conn)? {
        return Ok(meta);
    }
    let meta = Meta {
        id: ids.new_id(),
        created_at: clock.now_millis(),
    };
    conn.execute(
        "INSERT INTO meta (id, created_at) VALUES (?1, ?2)",
        (meta.id.to_string(), meta.created_at),
    )?;
    Ok(meta)
}

fn load_meta(conn: &Connection) -> rusqlite::Result<Option<Meta>> {
    conn.query_row("SELECT id, created_at FROM meta", [], |row| {
        Ok(Meta {
            id: uuid_column(row, 0)?,
            created_at: row.get(1)?,
        })
    })
    .optional()
}
