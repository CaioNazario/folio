//! Icons of projects and folders, stored by directory path (spec 03).
//!
//! Directories have no stable id: renaming or moving through Folio carries
//! the icon; doing it outside Folio drops back to the default icon.

use rusqlite::{Connection, OptionalExtension};

use crate::paths::RelativePath;

/// Sets the icon of the directory at `path`; `None` restores the default.
pub fn set_icon(
    conn: &Connection,
    path: &RelativePath,
    icon: Option<&str>,
) -> rusqlite::Result<()> {
    let Some(icon) = icon else {
        return conn
            .execute("DELETE FROM icons WHERE path = ?1", [path])
            .map(drop);
    };
    conn.execute(
        "INSERT INTO icons (path, icon) VALUES (?1, ?2)
         ON CONFLICT (path) DO UPDATE SET icon = excluded.icon",
        (path, icon),
    )
    .map(drop)
}

/// The icon chosen for the directory at `path`, if any.
pub fn icon_of(conn: &Connection, path: &RelativePath) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT icon FROM icons WHERE path = ?1", [path], |row| {
        row.get(0)
    })
    .optional()
}

/// Moves the icons of the directory `from` and of everything inside it to
/// `to`, in one transaction. Leftover icons already under `to` (from a
/// directory renamed outside Folio) are dropped first.
pub fn carry_icons(
    conn: &mut Connection,
    from: &RelativePath,
    to: &RelativePath,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    delete_within(&tx, to)?;
    for old in paths_within(&tx, from)? {
        let new = old.rebase(from, to);
        tx.execute("UPDATE icons SET path = ?2 WHERE path = ?1", (&old, &new))?;
    }
    tx.commit()
}

/// `dir` itself plus the byte range `dir/` .. `dir0`: `0` is the byte right
/// after `/`, so the range holds exactly the paths inside `dir`.
const WITHIN: &str = "path = ?1 OR (path > ?2 AND path < ?3)";

fn within_params(dir: &RelativePath) -> (&RelativePath, Vec<u8>, Vec<u8>) {
    let bound = |last: u8| [dir.as_bytes(), &[last]].concat();
    (dir, bound(b'/'), bound(b'0'))
}

fn paths_within(conn: &Connection, dir: &RelativePath) -> rusqlite::Result<Vec<RelativePath>> {
    conn.prepare(&format!("SELECT path FROM icons WHERE {WITHIN}"))?
        .query_map(within_params(dir), |row| row.get(0))?
        .collect()
}

fn delete_within(conn: &Connection, dir: &RelativePath) -> rusqlite::Result<()> {
    conn.execute(
        &format!("DELETE FROM icons WHERE {WITHIN}"),
        within_params(dir),
    )
    .map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrate;
    use crate::workspace::schema::MIGRATIONS;

    fn workspace_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn, MIGRATIONS).unwrap();
        conn
    }

    fn path(text: &str) -> RelativePath {
        RelativePath::from_bytes(text).unwrap()
    }

    #[test]
    fn set_icon_stores_icon_for_dir() {
        let conn = workspace_db();

        set_icon(&conn, &path("Faculdade"), Some("graduation-cap")).unwrap();

        assert_eq!(
            icon_of(&conn, &path("Faculdade")).unwrap().as_deref(),
            Some("graduation-cap")
        );
    }

    #[test]
    fn clearing_icon_restores_default() {
        let conn = workspace_db();
        set_icon(&conn, &path("Faculdade"), Some("graduation-cap")).unwrap();

        set_icon(&conn, &path("Faculdade"), None).unwrap();

        assert_eq!(icon_of(&conn, &path("Faculdade")).unwrap(), None);
    }
}
