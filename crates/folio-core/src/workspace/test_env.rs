//! A home directory, global database and fakes for workspace tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::db::GlobalDb;
use crate::services::Services;
use crate::testing::{ManualClock, RecordingTrash, SequentialIds};
use crate::workspace::{FolioDirStore, Workspace};

pub struct TestEnv {
    home: tempfile::TempDir,
    pub services: Services,
    pub clock: Arc<ManualClock>,
    pub trash: Arc<RecordingTrash>,
}

impl TestEnv {
    /// A fresh home with the global database in it; the clock starts at 1000.
    pub fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let clock = Arc::new(ManualClock::at(1_000));
        let ids = Arc::new(SequentialIds::default());
        let trash = Arc::new(RecordingTrash::default());
        let global_path = home.path().join(".global.db");
        let global = GlobalDb::open(&global_path, clock.clone(), ids.clone()).unwrap();
        let services = Services {
            home: home.path().to_path_buf(),
            global,
            store: Box::new(FolioDirStore),
            clock: clock.clone(),
            ids,
            trash: trash.clone(),
        };
        Self {
            home,
            services,
            clock,
            trash,
        }
    }

    pub fn home(&self) -> &Path {
        self.home.path()
    }

    /// Creates `<home>/<name>` and returns its path.
    pub fn dir(&self, name: &str) -> PathBuf {
        let dir = self.home().join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Opens `<home>/Folio` as a workspace, creating the directory.
    pub fn open_workspace(&self) -> Workspace {
        Workspace::open(&self.dir("Folio"), &self.services).unwrap()
    }
}
