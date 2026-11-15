//! What the app builds once and hands to every operation.

use std::path::PathBuf;
use std::sync::Arc;

use crate::clock::Clock;
use crate::db::GlobalDb;
use crate::ids::IdGenerator;
use crate::integrity::Trash;
use crate::workspace::WorkspaceStore;

/// App-wide dependencies, injected so tests can replace each one.
pub struct Services {
    /// The user's home, which can never be a workspace.
    pub home: PathBuf,
    pub global: GlobalDb,
    pub store: Box<dyn WorkspaceStore>,
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
    pub trash: Arc<dyn Trash>,
}
