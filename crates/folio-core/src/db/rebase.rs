//! Carrying index rows when a directory is renamed or moved (spec 03).

use rusqlite::Connection;

use super::columns::WITHIN;
use crate::paths::RelativePath;

/// Tables whose rows are tied to a path that moves with its directory.
#[derive(Debug, Clone, Copy)]
pub enum PathTable {
    Icons,
    Documents,
}

impl PathTable {
    fn name(self) -> &'static str {
        match self {
            Self::Icons => "icons",
            Self::Documents => "documents",
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
