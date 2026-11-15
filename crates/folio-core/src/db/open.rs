//! Opening a Folio database: pragmas, identity, integrity and recovery.

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, ErrorCode};

use super::functions::register_functions;
use super::migrate::{migrate, schema_version};
use crate::{Error, Result};

/// `PRAGMA application_id` of every Folio database: "FOLI" in ASCII.
pub const FOLIO_APPLICATION_ID: i32 = 0x464F_4C49;

/// A database ready to use.
pub struct OpenedDatabase {
    pub conn: Connection,
    /// The database file, for error messages.
    pub path: PathBuf,
    /// The previous file was unusable and was kept as `folio.db.corrupt`; the
    /// user must be told that favorites, icons and relations may be lost.
    pub recovered: bool,
}

/// Opens (or creates) the Folio database at `path` and brings its schema up
/// to date with `migrations`.
///
/// A file that is not a usable Folio database is kept as `folio.db.corrupt`
/// (replacing an older one) and a fresh database takes its place.
pub fn open_database(path: &Path, migrations: &[&str]) -> Result<OpenedDatabase> {
    let (conn, recovered) = match open_if_usable(path)? {
        Some(conn) => (conn, false),
        None => {
            quarantine(path)?;
            (open_fresh(path)?, true)
        }
    };
    let conn = bring_up_to_date(path, conn, migrations)?;
    Ok(OpenedDatabase {
        conn,
        path: path.to_path_buf(),
        recovered,
    })
}

/// A database from a newer Folio is left untouched: it is not corruption.
fn bring_up_to_date(path: &Path, mut conn: Connection, migrations: &[&str]) -> Result<Connection> {
    let db_error = |e| Error::database(path, e);
    if schema_version(&conn).map_err(db_error)? > migrations.len() as i64 {
        return Err(Error::NewerDatabase(path.to_path_buf()));
    }
    register_functions(&conn).map_err(db_error)?;
    migrate(&mut conn, migrations).map_err(db_error)?;
    Ok(conn)
}

/// `None` when the file exists but is not a Folio database.
fn open_if_usable(path: &Path) -> Result<Option<Connection>> {
    let db_error = |e| Error::database(path, e);
    let conn = Connection::open(path).map_err(db_error)?;
    // Identity comes before the pragmas: enabling WAL writes to the file, and
    // a foreign file must reach `.corrupt` untouched.
    if !is_usable(&conn)
        .or_else(unusable_as_false)
        .map_err(db_error)?
    {
        return Ok(None);
    }
    apply_pragmas(&conn).map_err(db_error)?;
    Ok(Some(conn))
}

fn open_fresh(path: &Path) -> Result<Connection> {
    let db_error = |e| Error::database(path, e);
    let conn = Connection::open(path).map_err(db_error)?;
    stamp_identity(&conn).map_err(db_error)?;
    apply_pragmas(&conn).map_err(db_error)?;
    Ok(conn)
}

/// A Folio database that passes `PRAGMA quick_check` (spec 01, guarantee 6).
fn is_usable(conn: &Connection) -> rusqlite::Result<bool> {
    Ok(claim_identity(conn)? && passes_quick_check(conn)?)
}

fn passes_quick_check(conn: &Connection) -> rusqlite::Result<bool> {
    let first_row: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    Ok(first_row == "ok")
}

/// True for a Folio database. A brand-new empty one receives the id.
fn claim_identity(conn: &Connection) -> rusqlite::Result<bool> {
    let id: i32 = conn.pragma_query_value(None, "application_id", |row| row.get(0))?;
    if id == FOLIO_APPLICATION_ID {
        return Ok(true);
    }
    if id != 0 || !is_empty(conn)? {
        return Ok(false);
    }
    stamp_identity(conn)?;
    Ok(true)
}

/// A file SQLite cannot read as a database is unusable, not an I/O failure.
fn unusable_as_false(error: rusqlite::Error) -> rusqlite::Result<bool> {
    match error.sqlite_error_code() {
        Some(ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt) => Ok(false),
        _ => Err(error),
    }
}

fn stamp_identity(conn: &Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "application_id", FOLIO_APPLICATION_ID)
}

fn is_empty(conn: &Connection) -> rusqlite::Result<bool> {
    let objects: i64 =
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| row.get(0))?;
    Ok(objects == 0)
}

/// Keeps the unusable file as `<name>.corrupt` for manual recovery. It is
/// Folio's own file, so replacing an older `.corrupt` is intended.
fn quarantine(path: &Path) -> Result<()> {
    let mut corrupt = path.as_os_str().to_owned();
    corrupt.push(".corrupt");
    fs::rename(path, &corrupt).map_err(|e| Error::io(path, e))
}

/// WAL with `synchronous = NORMAL` survives crashes without losing committed
/// transactions (spec 01, guarantee 6).
fn apply_pragmas(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pragma_i64(conn: &Connection, name: &str) -> i64 {
        conn.pragma_query_value(None, name, |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn open_creates_database_with_folio_application_id() {
        let dir = tempfile::tempdir().unwrap();

        let db = open_database(&dir.path().join("folio.db"), &[]).unwrap();

        assert_eq!(
            pragma_i64(&db.conn, "application_id"),
            i64::from(FOLIO_APPLICATION_ID)
        );
    }

    #[test]
    fn open_applies_wal_normal_sync_foreign_keys_and_busy_timeout() {
        let dir = tempfile::tempdir().unwrap();

        let db = open_database(&dir.path().join("folio.db"), &[]).unwrap();

        let mode: String = db
            .conn
            .pragma_query_value(None, "journal_mode", |r| r.get(0))
            .unwrap();
        assert_eq!(mode, "wal");
        assert_eq!(pragma_i64(&db.conn, "synchronous"), 1, "NORMAL");
        assert_eq!(pragma_i64(&db.conn, "foreign_keys"), 1);
        assert_eq!(pragma_i64(&db.conn, "busy_timeout"), 5000);
    }

    fn db_path() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("folio.db");
        (dir, path)
    }

    #[test]
    fn open_keeps_existing_data_on_reopen() {
        let (_dir, path) = db_path();
        let first = open_database(&path, &[]).unwrap();
        first
            .conn
            .execute_batch("CREATE TABLE t (x); INSERT INTO t VALUES (42);")
            .unwrap();
        drop(first);

        let reopened = open_database(&path, &[]).unwrap();

        let x: i64 = reopened
            .conn
            .query_row("SELECT x FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!((x, reopened.recovered), (42, false));
    }

    #[test]
    fn open_reports_no_recovery_for_new_database() {
        let (_dir, path) = db_path();

        let db = open_database(&path, &[]).unwrap();

        assert!(!db.recovered);
    }

    #[test]
    fn open_moves_foreign_sqlite_file_to_corrupt_and_reports_recovery() {
        let (dir, path) = db_path();
        let foreign = Connection::open(&path).unwrap();
        foreign
            .execute_batch("CREATE TABLE alheia (x); INSERT INTO alheia VALUES (1);")
            .unwrap();
        drop(foreign);
        let original = fs::read(&path).unwrap();

        let db = open_database(&path, &[]).unwrap();

        assert!(db.recovered);
        assert_eq!(
            fs::read(dir.path().join("folio.db.corrupt")).unwrap(),
            original
        );
        assert_eq!(
            pragma_i64(&db.conn, "application_id"),
            i64::from(FOLIO_APPLICATION_ID)
        );
    }

    #[test]
    fn open_moves_unreadable_file_to_corrupt() {
        let (dir, path) = db_path();
        fs::write(&path, "isto não é um banco SQLite, só texto qualquer").unwrap();

        let db = open_database(&path, &[]).unwrap();

        assert!(db.recovered);
        assert!(dir.path().join("folio.db.corrupt").is_file());
    }

    /// A Folio database with several pages, closed so the WAL is checkpointed.
    fn create_multi_page_database(path: &Path) {
        let db = open_database(path, &[]).unwrap();
        db.conn
            .execute_batch(
                "CREATE TABLE t (x TEXT);
                 WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 2000)
                 INSERT INTO t SELECT printf('linha %d com algum texto', i) FROM n;",
            )
            .unwrap();
    }

    #[test]
    fn open_moves_database_failing_quick_check_to_corrupt() {
        let (dir, path) = db_path();
        create_multi_page_database(&path);
        let mut bytes = fs::read(&path).unwrap();
        bytes[3 * 4096..4 * 4096].fill(0xA5);
        fs::write(&path, bytes).unwrap();

        let db = open_database(&path, &[]).unwrap();

        assert!(db.recovered);
        assert!(dir.path().join("folio.db.corrupt").is_file());
    }

    #[test]
    fn open_replaces_previous_corrupt_file() {
        let (dir, path) = db_path();
        let corrupt = dir.path().join("folio.db.corrupt");
        fs::write(&corrupt, "corrompido da semana passada").unwrap();
        fs::write(&path, "corrompido de hoje").unwrap();

        open_database(&path, &[]).unwrap();

        assert_eq!(fs::read_to_string(&corrupt).unwrap(), "corrompido de hoje");
    }

    #[test]
    fn open_refuses_newer_database_without_touching_it() {
        let (dir, path) = db_path();
        let newer = open_database(&path, &[]).unwrap();
        newer.conn.pragma_update(None, "user_version", 5).unwrap();
        drop(newer);
        let original = fs::read(&path).unwrap();

        let result = open_database(&path, &["CREATE TABLE a (x)"]);

        assert!(matches!(result, Err(Error::NewerDatabase(p)) if p == path));
        assert_eq!(fs::read(&path).unwrap(), original);
        assert!(!dir.path().join("folio.db.corrupt").exists());
    }
}
