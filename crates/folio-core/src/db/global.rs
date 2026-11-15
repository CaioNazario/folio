//! The app's global database: known workspaces and preferences (spec 02).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Row};
use serde_json::Value;
use uuid::Uuid;

use super::columns::uuid_column;
use super::open_database;
use crate::clock::Clock;
use crate::ids::IdGenerator;
use crate::paths::Location;
use crate::{Error, Result};

/// Schema of the global database. Never edit a published entry: append.
const MIGRATIONS: &[&str] = &["CREATE TABLE workspaces (
         id TEXT PRIMARY KEY NOT NULL,
         location BLOB NOT NULL UNIQUE,
         added_at INTEGER NOT NULL,
         opened_at INTEGER NOT NULL
     );
     CREATE TABLE preferences (
         key TEXT PRIMARY KEY NOT NULL,
         value TEXT NOT NULL
     );"];

/// A workspace the app knows about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownWorkspace {
    pub id: Uuid,
    pub location: Location,
    pub added_at: i64,
    pub opened_at: i64,
}

/// The global database, behind one shared connection.
pub struct GlobalDb {
    conn: Mutex<Connection>,
    path: PathBuf,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
    /// The previous file was unusable and was kept as `folio.db.corrupt`.
    pub recovered: bool,
}

impl GlobalDb {
    /// Opens (or creates) the global database at `path`.
    pub fn open(path: &Path, clock: Arc<dyn Clock>, ids: Arc<dyn IdGenerator>) -> Result<Self> {
        let opened = open_database(path, MIGRATIONS)?;
        Ok(Self {
            conn: Mutex::new(opened.conn),
            path: path.to_path_buf(),
            clock,
            ids,
            recovered: opened.recovered,
        })
    }

    /// Records that the workspace at `location` was opened now, adding it if
    /// it is new. A known workspace keeps its id and `added_at`.
    pub fn remember_workspace(&self, location: &Location) -> Result<KnownWorkspace> {
        let id = self.ids.new_id().to_string();
        let now = self.clock.now_millis();
        self.conn()
            .query_row(
                "INSERT INTO workspaces (id, location, added_at, opened_at)
                 VALUES (?1, ?2, ?3, ?3)
                 ON CONFLICT (location) DO UPDATE SET opened_at = excluded.opened_at
                 RETURNING id, location, added_at, opened_at",
                (id, location, now),
                known_workspace,
            )
            .map_err(|e| self.db_error(e))
    }

    /// Known workspaces, most recently opened first.
    pub fn recent_workspaces(&self) -> Result<Vec<KnownWorkspace>> {
        let conn = self.conn();
        let query = || -> rusqlite::Result<Vec<KnownWorkspace>> {
            conn.prepare(
                "SELECT id, location, added_at, opened_at FROM workspaces
                 ORDER BY opened_at DESC",
            )?
            .query_map([], known_workspace)?
            .collect()
        };
        query().map_err(|e| self.db_error(e))
    }

    /// Removes a workspace from the known list. Nothing on disk is touched.
    pub fn forget_workspace(&self, id: Uuid) -> Result<()> {
        self.conn()
            .execute("DELETE FROM workspaces WHERE id = ?1", [id.to_string()])
            .map(drop)
            .map_err(|e| self.db_error(e))
    }

    /// The workspace the app opens on start: the most recently opened one.
    pub fn last_opened_workspace(&self) -> Result<Option<KnownWorkspace>> {
        Ok(self.recent_workspaces()?.into_iter().next())
    }

    /// The stored value of preference `key`, if any.
    pub fn preference(&self, key: &str) -> Result<Option<Value>> {
        self.conn()
            .query_row(
                "SELECT value FROM preferences WHERE key = ?1",
                [key],
                json_column,
            )
            .optional()
            .map_err(|e| self.db_error(e))
    }

    /// Stores `value` as preference `key`.
    pub fn set_preference(&self, key: &str, value: &Value) -> Result<()> {
        self.conn()
            .execute(
                "INSERT INTO preferences (key, value) VALUES (?1, ?2)
                 ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                (key, value.to_string()),
            )
            .map(drop)
            .map_err(|e| self.db_error(e))
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn db_error(&self, source: rusqlite::Error) -> Error {
        Error::database(&self.path, source)
    }
}

fn known_workspace(row: &Row<'_>) -> rusqlite::Result<KnownWorkspace> {
    Ok(KnownWorkspace {
        id: uuid_column(row, 0)?,
        location: row.get(1)?,
        added_at: row.get(2)?,
        opened_at: row.get(3)?,
    })
}

/// Preference values are stored as JSON text.
fn json_column(row: &Row<'_>) -> rusqlite::Result<Value> {
    let text: String = row.get(0)?;
    serde_json::from_str(&text)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, Type::Text, Box::new(e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{ManualClock, SequentialIds};

    struct Fixture {
        _dir: tempfile::TempDir,
        clock: Arc<ManualClock>,
        db: GlobalDb,
    }

    /// A global database whose clock starts at 1000 ms.
    fn global_db() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let clock = Arc::new(ManualClock::at(1_000));
        let ids = Arc::new(SequentialIds::default());
        let db = GlobalDb::open(&dir.path().join("folio.db"), clock.clone(), ids).unwrap();
        Fixture {
            _dir: dir,
            clock,
            db,
        }
    }

    fn location(path: &str) -> Location {
        Location::Path(PathBuf::from(path))
    }

    #[test]
    fn remembering_new_workspace_records_location_and_dates() {
        let fixture = global_db();

        let known = fixture
            .db
            .remember_workspace(&location("/home/caio/Folio"))
            .unwrap();

        let expected = KnownWorkspace {
            id: Uuid::from_u128(1),
            location: location("/home/caio/Folio"),
            added_at: 1_000,
            opened_at: 1_000,
        };
        assert_eq!(known, expected);
    }

    #[test]
    fn remembering_known_workspace_keeps_its_id_and_updates_opened_at() {
        let fixture = global_db();
        fixture
            .db
            .remember_workspace(&location("/home/caio/Folio"))
            .unwrap();
        fixture.clock.set(5_000);

        let known = fixture
            .db
            .remember_workspace(&location("/home/caio/Folio"))
            .unwrap();

        let ids_and_dates = (known.id, known.added_at, known.opened_at);
        assert_eq!(ids_and_dates, (Uuid::from_u128(1), 1_000, 5_000));
    }

    /// Opens the given workspaces in order, one second apart.
    fn open_in_order(fixture: &Fixture, paths: &[&str]) {
        for (second, path) in (1..).zip(paths) {
            fixture.clock.set(second * 1_000);
            fixture.db.remember_workspace(&location(path)).unwrap();
        }
    }

    #[test]
    fn recent_workspaces_are_ordered_by_last_opened() {
        let fixture = global_db();
        open_in_order(&fixture, &["/a", "/b", "/c", "/a"]);

        let recent = fixture.db.recent_workspaces().unwrap();

        let locations: Vec<_> = recent.into_iter().map(|known| known.location).collect();
        assert_eq!(locations, [location("/a"), location("/c"), location("/b")]);
    }

    #[test]
    fn last_opened_workspace_is_the_most_recent() {
        let fixture = global_db();
        open_in_order(&fixture, &["/a", "/b"]);

        let last = fixture.db.last_opened_workspace().unwrap();

        assert_eq!(last.map(|known| known.location), Some(location("/b")));
    }

    #[test]
    fn last_opened_workspace_is_none_without_workspaces() {
        let fixture = global_db();

        assert_eq!(fixture.db.last_opened_workspace().unwrap(), None);
    }

    #[test]
    fn preference_round_trips_json_value() {
        let fixture = global_db();
        let value = serde_json::json!({ "path": "/home/caio/Downloads" });

        fixture.db.set_preference("inbox.location", &value).unwrap();

        assert_eq!(
            fixture.db.preference("inbox.location").unwrap(),
            Some(value)
        );
    }

    #[test]
    fn missing_preference_returns_none() {
        let fixture = global_db();

        assert_eq!(fixture.db.preference("theme").unwrap(), None);
    }

    #[test]
    fn setting_preference_replaces_previous_value() {
        let fixture = global_db();
        fixture.db.set_preference("theme", &"dark".into()).unwrap();

        fixture.db.set_preference("theme", &"light".into()).unwrap();

        assert_eq!(
            fixture.db.preference("theme").unwrap(),
            Some("light".into())
        );
    }
}
