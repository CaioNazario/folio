//! Opening a workspace (spec 03, "Passos da abertura").

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rusqlite::Connection;
use uuid::Uuid;

use crate::clock::Clock;
use crate::ids::IdGenerator;
use crate::integrity::{Trash, WorkspaceGuard};
use crate::paths::{Location, RelativePath};
use crate::services::Services;
use crate::workspace::icons;
use crate::workspace::meta::{Meta, load_or_create_meta};
use crate::workspace::schema::MIGRATIONS;
use crate::workspace::store::WorkspaceState;
use crate::workspace::validate::validate_root;
use crate::{Error, Result};

/// An open workspace. Dropping it closes the database and releases the lock,
/// so switching workspaces is dropping one and opening another.
pub struct Workspace {
    root: PathBuf,
    meta: Meta,
    recovered: bool,
    db_path: PathBuf,
    pub(crate) trash: Arc<dyn Trash>,
    pub(crate) ids: Arc<dyn IdGenerator>,
    pub(crate) clock: Arc<dyn Clock>,
    // Declared before the guard: fields drop in order, so the database
    // closes before the lock is released.
    conn: Mutex<Connection>,
    guard: WorkspaceGuard,
}

impl Workspace {
    /// Validates `root`, takes its lock, opens its database and records it in
    /// the global database.
    pub fn open(root: &Path, services: &Services) -> Result<Self> {
        validate_root(root, &services.home)?;
        let WorkspaceState { db, guard } = services.store.open(root, MIGRATIONS)?;
        let meta = load_or_create_meta(&db.conn, &*services.clock, &*services.ids)
            .map_err(|e| Error::database(&db.path, e))?;
        services
            .global
            .remember_workspace(&Location::Path(root.to_path_buf()))?;
        let workspace = Self::assemble(root, meta, WorkspaceState { db, guard }, services);
        // Missing documents past 30 days leave the index on opening (spec 05).
        workspace.purge_expired_missing()?;
        Ok(workspace)
    }

    fn assemble(root: &Path, meta: Meta, state: WorkspaceState, services: &Services) -> Self {
        let WorkspaceState { db, guard } = state;
        Self {
            root: root.to_path_buf(),
            meta,
            recovered: db.recovered,
            db_path: db.path,
            trash: services.trash.clone(),
            ids: services.ids.clone(),
            clock: services.clock.clone(),
            conn: Mutex::new(db.conn),
            guard,
        }
    }

    /// The workspace's own id, kept in its database (table `meta`).
    pub fn id(&self) -> Uuid {
        self.meta.id
    }

    /// When Folio first opened this directory, in ms since the Unix epoch.
    pub fn created_at(&self) -> i64 {
        self.meta.created_at
    }

    /// The database was unusable and was rebuilt: the user must be told that
    /// favorites, icons, recents and conversion links may be lost (spec 01).
    pub fn recovered(&self) -> bool {
        self.recovered
    }

    /// Sets the icon of the project or folder at `path`; `None` restores the
    /// default. The core does not validate the icon name.
    pub fn set_icon(&self, path: &RelativePath, icon: Option<&str>) -> Result<()> {
        icons::set_icon(&self.conn(), path, icon).map_err(|e| self.db_error(e))
    }

    pub(crate) fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Index update for a change already done on disk. The disk is the truth:
    /// a failure here is logged and the operation still stands; the next scan
    /// fixes the index (spec 01, guarantee 5).
    pub(crate) fn update_index_after_disk(
        &self,
        update: impl FnOnce(&mut Connection) -> rusqlite::Result<()>,
    ) {
        if let Err(error) = update(&mut self.conn()) {
            log::warn!(
                "index update failed after a disk change: {}",
                self.db_error(error)
            );
        }
    }

    pub(crate) fn db_error(&self, source: rusqlite::Error) -> Error {
        Error::database(&self.db_path, source)
    }

    /// The workspace directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `.folio/tmp/`, where new files are born (spec 01, guarantee 2).
    pub(crate) fn tmp_dir(&self) -> &Path {
        self.guard.tmp_dir()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::*;
    use crate::paths::Location;
    use crate::workspace::test_env::TestEnv;

    #[test]
    fn open_fails_with_workspace_not_found_when_dir_is_missing() {
        let env = TestEnv::new();
        let missing = env.home().join("Sumiu");

        let result = Workspace::open(&missing, &env.services);

        assert!(matches!(result, Err(Error::WorkspaceNotFound(path)) if path == missing));
    }

    #[test]
    fn open_fails_with_workspace_not_found_when_path_is_a_file() {
        let env = TestEnv::new();
        let file = env.home().join("arquivo.txt");
        fs::write(&file, "não é diretório").unwrap();

        let result = Workspace::open(&file, &env.services);

        assert!(matches!(result, Err(Error::WorkspaceNotFound(path)) if path == file));
    }

    #[test]
    fn open_fails_with_permission_denied_when_dir_is_read_only() {
        let env = TestEnv::new();
        let read_only = env.dir("SoLeitura");
        fs::set_permissions(&read_only, fs::Permissions::from_mode(0o555)).unwrap();

        let result = Workspace::open(&read_only, &env.services);

        assert!(matches!(result, Err(Error::PermissionDenied(path)) if path == read_only));
    }

    #[test]
    fn open_refuses_filesystem_root() {
        let env = TestEnv::new();

        let result = Workspace::open(Path::new("/"), &env.services);

        assert!(matches!(result, Err(Error::ForbiddenRoot(path)) if path == Path::new("/")));
    }

    #[test]
    fn open_refuses_home_dir() {
        let env = TestEnv::new();
        let home = env.home().to_path_buf();

        let result = Workspace::open(&home, &env.services);

        assert!(matches!(result, Err(Error::ForbiddenRoot(path)) if path == home));
    }

    #[test]
    fn open_refuses_dir_inside_another_workspace_naming_the_ancestor() {
        let env = TestEnv::new();
        let outer = env.dir("Folio");
        fs::create_dir(outer.join(".folio")).unwrap();
        let inner = env.dir("Folio/Faculdade/TCC");

        let result = Workspace::open(&inner, &env.services);

        assert!(matches!(result, Err(Error::NestedWorkspace { ancestor }) if ancestor == outer));
    }

    #[test]
    fn open_creates_folio_dir_tmp_and_database() {
        let env = TestEnv::new();

        let workspace = env.open_workspace();

        let folio = workspace.root().join(".folio");
        assert!(folio.join("tmp").is_dir());
        assert!(folio.join("folio.db").is_file());
    }

    #[test]
    fn opening_new_workspace_records_its_id_and_creation_date() {
        let env = TestEnv::new();

        let workspace = env.open_workspace();

        assert_eq!(
            (workspace.id(), workspace.created_at()),
            (Uuid::from_u128(1), 1_000)
        );
    }

    #[test]
    fn reopening_workspace_keeps_its_id() {
        let env = TestEnv::new();
        drop(env.open_workspace());
        env.clock.set(9_000);

        let reopened = env.open_workspace();

        assert_eq!(
            (reopened.id(), reopened.created_at()),
            (Uuid::from_u128(1), 1_000)
        );
    }

    #[test]
    fn open_fails_with_workspace_in_use_when_already_open() {
        let env = TestEnv::new();
        let _first = env.open_workspace();

        let second = Workspace::open(&env.home().join("Folio"), &env.services);

        assert!(matches!(second, Err(Error::WorkspaceInUse)));
    }

    #[test]
    fn dropping_workspace_releases_its_lock() {
        let env = TestEnv::new();
        drop(env.open_workspace());

        let reopened = Workspace::open(&env.home().join("Folio"), &env.services);

        assert!(reopened.is_ok_and(|workspace| workspace.root() == env.home().join("Folio")));
    }

    #[test]
    fn open_registers_workspace_in_global_db() {
        let env = TestEnv::new();

        let workspace = env.open_workspace();

        let last = env
            .services
            .global
            .last_opened_workspace()
            .unwrap()
            .unwrap();
        assert_eq!(
            last.location,
            Location::Path(workspace.root().to_path_buf())
        );
    }

    fn recent_locations(env: &TestEnv) -> Vec<Location> {
        let recent = env.services.global.recent_workspaces().unwrap();
        recent.into_iter().map(|known| known.location).collect()
    }

    #[test]
    fn failed_open_keeps_workspace_in_recent_list() {
        let env = TestEnv::new();
        let root = env.open_workspace().root().to_path_buf();
        fs::remove_dir_all(&root).unwrap();

        let result = Workspace::open(&root, &env.services);

        assert!(matches!(result, Err(Error::WorkspaceNotFound(_))));
        assert_eq!(recent_locations(&env), [Location::Path(root)]);
    }

    #[test]
    fn forgetting_workspace_removes_it_from_recent_list_without_touching_disk() {
        let env = TestEnv::new();
        let root = env.open_workspace().root().to_path_buf();
        let known = env
            .services
            .global
            .last_opened_workspace()
            .unwrap()
            .unwrap();

        env.services.global.forget_workspace(known.id).unwrap();

        assert_eq!(recent_locations(&env), []);
        assert!(root.join(".folio/folio.db").is_file());
    }

    #[test]
    fn open_reports_recovery_when_workspace_db_was_corrupt() {
        let env = TestEnv::new();
        let root = env.open_workspace().root().to_path_buf();
        fs::write(root.join(".folio/folio.db"), "corrompido").unwrap();

        let workspace = env.open_workspace();

        assert!(workspace.recovered());
    }
}
