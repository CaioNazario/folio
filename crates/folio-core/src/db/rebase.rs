//! Index rows tied to a directory path: carried when the directory is
//! renamed or moved through Folio (spec 03), dropped when it vanished
//! outside Folio (spec 05).

use rusqlite::Connection;

use super::columns::WITHIN;
use crate::paths::RelativePath;

/// Tables whose rows are tied to a path that moves with its directory.
#[derive(Debug, Clone, Copy)]
pub enum PathTable {
    Icons,
    Documents,
    /// Only favorites of directories have a path (spec 06).
    Favorites,
}

impl PathTable {
    fn name(self) -> &'static str {
        match self {
            Self::Icons => "icons",
            Self::Documents => "documents",
            Self::Favorites => "favorites",
        }
    }
}

/// Moves the rows of `table` at the directory `from` and inside it to the
/// same place under `to`. Rows already under `to` are stale: that path did
/// not exist on disk until now, so they are dropped first.
pub fn rebase_paths(
    conn: &Connection,
    table: PathTable,
    from: &RelativePath,
    to: &RelativePath,
) -> rusqlite::Result<()> {
    let table = table.name();
    conn.execute(
        &format!("DELETE FROM {table} WHERE {WITHIN}"),
        within_params(to),
    )?;
    for old in paths_within(conn, table, from)? {
        let new = old.rebase(from, to);
        conn.execute(
            &format!("UPDATE {table} SET path = ?2 WHERE path = ?1"),
            (&old, &new),
        )?;
    }
    Ok(())
}

fn within_params(dir: &RelativePath) -> (&RelativePath, Vec<u8>, Vec<u8>) {
    let (lower, upper) = dir.descendant_bounds();
    (dir, lower, upper)
}

fn paths_within(
    conn: &Connection,
    table: &str,
    dir: &RelativePath,
) -> rusqlite::Result<Vec<RelativePath>> {
    conn.prepare(&format!("SELECT path FROM {table} WHERE {WITHIN}"))?
        .query_map(within_params(dir), |row| row.get(0))?
        .collect()
}

/// Every path in `table`; rows without one (document favorites) are skipped.
pub fn table_paths(conn: &Connection, table: PathTable) -> rusqlite::Result<Vec<RelativePath>> {
    let sql = format!("SELECT path FROM {} WHERE path IS NOT NULL", table.name());
    conn.prepare(&sql)?
        .query_map([], |row| row.get(0))?
        .collect()
}

/// Drops the row of `table` at exactly `path`.
pub fn delete_path(
    conn: &Connection,
    table: PathTable,
    path: &RelativePath,
) -> rusqlite::Result<()> {
    let sql = format!("DELETE FROM {} WHERE path = ?1", table.name());
    conn.execute(&sql, [path]).map(drop)
}
