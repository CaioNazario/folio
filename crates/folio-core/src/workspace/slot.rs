//! The workspace open in this process (spec 03, "Ao iniciar o app"). Opening
//! the one already open reuses it: its lock belongs to this process, so only
//! another process gets `WorkspaceInUse`.

use std::fs;
use std::path::Path;
use std::sync::{Arc, PoisonError, RwLock};

use super::Workspace;
use crate::services::Services;
use crate::{Error, Result};

/// Holds the open workspace, one per window.
#[derive(Default)]
pub struct WorkspaceSlot {
    current: RwLock<Option<Arc<Workspace>>>,
}

/// The result of opening through the slot.
pub struct Opened {
    pub workspace: Arc<Workspace>,
    /// `false` when it was already open and was reused: nothing to index.
    pub fresh: bool,
}

impl WorkspaceSlot {
    /// The open workspace, if any.
    pub fn current(&self) -> Option<Arc<Workspace>> {
        let current = self.current.read().unwrap_or_else(PoisonError::into_inner);
        current.clone()
    }

    /// Opens the last workspace, or reuses it when it is the one already
    /// open; `None` on the first run.
    pub fn open_last(&self, services: &Services) -> Result<Option<Opened>> {
        let Some(root) = Workspace::last_root(services)? else {
            return Ok(None);
        };
        self.open(&root, services).map(Some)
    }

    /// Creates the directory `root` when needed and opens it: the
    /// confirmation of the first-run dialog.
    pub fn create(&self, root: &Path, services: &Services) -> Result<Opened> {
        fs::create_dir_all(root).map_err(|e| Error::io(root, e))?;
        self.open(root, services)
    }

    /// Opens `root`, or reuses it when it is the one already open.
    pub fn open(&self, root: &Path, services: &Services) -> Result<Opened> {
        // Held while opening: two calls at once (StrictMode, a reload) must
        // not both try to take the lock.
        let mut current = self.current.write().unwrap_or_else(PoisonError::into_inner);
        if let Some(open) = current.as_ref().filter(|open| open.root() == root) {
            return Ok(Opened {
                workspace: open.clone(),
                fresh: false,
            });
        }
        let workspace = Arc::new(Workspace::open(root, services)?);
        *current = Some(workspace.clone());
        Ok(Opened {
            workspace,
            fresh: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::test_env::TestEnv;

    #[test]
    fn opening_same_root_again_reuses_the_open_workspace() {
        let env = TestEnv::new();
        let root = env.dir("Folio");
        let slot = WorkspaceSlot::default();
        let first = slot.open(&root, &env.services).unwrap();

        let again = slot.open(&root, &env.services).unwrap();

        assert!(Arc::ptr_eq(&first.workspace, &again.workspace));
        assert_eq!((first.fresh, again.fresh), (true, false));
    }

    #[test]
    fn opening_last_workspace_again_reuses_the_open_workspace() {
        let env = TestEnv::new();
        let slot = WorkspaceSlot::default();
        let first = slot.open(&env.dir("Folio"), &env.services).unwrap();

        let again = slot.open_last(&env.services).unwrap().unwrap();

        assert!(Arc::ptr_eq(&first.workspace, &again.workspace));
        assert!(!again.fresh);
    }

    #[test]
    fn opening_other_root_replaces_the_open_workspace_and_releases_its_lock() {
        let env = TestEnv::new();
        let (first, second) = (env.dir("Primeiro"), env.dir("Segundo"));
        let slot = WorkspaceSlot::default();
        drop(slot.open(&first, &env.services).unwrap());

        slot.open(&second, &env.services).unwrap();

        assert_eq!(slot.current().unwrap().root(), second);
        let reopened = Workspace::open(&first, &env.services).unwrap();
        assert_eq!(reopened.root(), first);
    }

    #[test]
    fn opening_root_locked_by_another_instance_still_fails_with_workspace_in_use() {
        let env = TestEnv::new();
        let root = env.dir("Folio");
        let _elsewhere = Workspace::open(&root, &env.services).unwrap();
        let slot = WorkspaceSlot::default();

        let opened = slot.open(&root, &env.services);

        assert!(matches!(opened, Err(Error::WorkspaceInUse)));
    }

    #[test]
    fn creating_root_makes_it_the_open_workspace() {
        let env = TestEnv::new();
        let root = env.services.home.join("Documentos/Folio");
        let slot = WorkspaceSlot::default();

        let created = slot.create(&root, &env.services).unwrap();

        assert!(root.is_dir());
        assert!(Arc::ptr_eq(&created.workspace, &slot.current().unwrap()));
    }
}
