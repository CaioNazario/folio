//! Which workspace the app opens (spec 03, "Ao iniciar o app" and
//! "Primeira execução").

use std::fs;
use std::path::{Path, PathBuf};

use super::Workspace;
use crate::paths::Location;
use crate::services::Services;
use crate::{Error, Result};

impl Workspace {
    /// The workspace opened most recently, or `None` on the first run, when
    /// the app shows the dialog to choose one.
    pub fn open_last(services: &Services) -> Result<Option<Self>> {
        let Some(root) = Self::last_root(services)? else {
            return Ok(None);
        };
        Self::open(&root, services).map(Some)
    }

    /// The directory of the workspace opened most recently, if any.
    pub(crate) fn last_root(services: &Services) -> Result<Option<PathBuf>> {
        let last = services.global.last_opened_workspace()?;
        Ok(last.map(|last| {
            let Location::Path(root) = last.location;
            root
        }))
    }

    /// Opens `root` as chosen in the first-run dialog, creating the directory
    /// (and its parents) when it does not exist yet.
    pub fn create(root: &Path, services: &Services) -> Result<Self> {
        fs::create_dir_all(root).map_err(|e| Error::io(root, e))?;
        Self::open(root, services)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::test_env::TestEnv;

    #[test]
    fn open_last_returns_none_on_first_run() {
        let env = TestEnv::new();

        let opened = Workspace::open_last(&env.services).unwrap();

        assert!(opened.is_none());
    }

    #[test]
    fn open_last_opens_most_recently_opened_workspace() {
        let env = TestEnv::new();
        let first = env.dir("Primeiro");
        let second = env.dir("Segundo");
        drop(Workspace::open(&first, &env.services).unwrap());
        env.clock.set(2_000);
        drop(Workspace::open(&second, &env.services).unwrap());

        let opened = Workspace::open_last(&env.services).unwrap().unwrap();

        assert_eq!(opened.root(), second);
    }

    #[test]
    fn open_last_fails_with_workspace_not_found_keeping_it_listed() {
        let env = TestEnv::new();
        let root = env.dir("Sumiu");
        drop(Workspace::open(&root, &env.services).unwrap());
        std::fs::remove_dir_all(&root).unwrap();

        let result = Workspace::open_last(&env.services);

        assert!(matches!(result, Err(Error::WorkspaceNotFound(path)) if path == root));
        assert_eq!(env.services.global.recent_workspaces().unwrap().len(), 1);
    }

    #[test]
    fn create_workspace_creates_missing_dir_and_opens_it() {
        let env = TestEnv::new();
        let root = env.home().join("Documentos/Folio");

        let workspace = Workspace::create(&root, &env.services).unwrap();

        assert_eq!(workspace.root(), root);
        assert!(root.join(".folio").is_dir());
    }
}
