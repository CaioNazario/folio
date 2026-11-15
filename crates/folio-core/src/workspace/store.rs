//! Where a workspace keeps its own state (spec 02, `WorkspaceStore`).

use std::path::Path;
use std::{fs, io};

use super::validate::FOLIO_DIR;
use crate::db::{OpenedDatabase, open_database};
use crate::integrity::WorkspaceGuard;
use crate::{Error, Result};

/// Opens the state of a workspace: its lock, temp dir and database. Desktop
/// keeps it in `<workspace>/.folio/`; Android with SAF will keep it in app
/// storage.
pub trait WorkspaceStore: Send + Sync {
    /// Takes exclusive hold of the state of the workspace at `root`, empties
    /// its temp dir and opens its database with `migrations`.
    fn open(&self, root: &Path, migrations: &[&str]) -> Result<WorkspaceState>;
}

/// The open state of a workspace. Dropping it closes the database, then
/// releases the lock.
pub struct WorkspaceState {
    pub db: OpenedDatabase,
    pub guard: WorkspaceGuard,
}

/// State in `<workspace>/.folio/`: `folio.db`, `tmp/` and `lock`.
pub struct FolioDirStore;

impl WorkspaceStore for FolioDirStore {
    fn open(&self, root: &Path, migrations: &[&str]) -> Result<WorkspaceState> {
        let folio = root.join(FOLIO_DIR);
        create_dir_if_missing(&folio)?;
        // The lock comes before the database: another window's open is
        // refused before anything here is touched (spec 01, guarantee 7).
        let guard = WorkspaceGuard::acquire(&folio)?;
        let db = open_database(&folio.join("folio.db"), migrations)?;
        Ok(WorkspaceState { db, guard })
    }
}

/// `.folio/` is Folio's own directory, so finding it already there is normal.
fn create_dir_if_missing(dir: &Path) -> Result<()> {
    match fs::create_dir(dir) {
        Err(e) if e.kind() != io::ErrorKind::AlreadyExists => Err(Error::io(dir, e)),
        _ => Ok(()),
    }
}
