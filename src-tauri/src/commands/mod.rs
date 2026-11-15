//! Tauri commands: thin wrappers that deserialize, call `folio-core` and
//! return its result. Errors cross the bridge as `BridgeError` data; their
//! technical detail goes to the log.

pub mod documents;
pub mod library;
pub mod preferences;
pub mod workspace;

use folio_core::Error;
use folio_core::bridge_error::BridgeError;
use folio_core::workspace::Workspace;

use crate::state::AppState;

/// The result every command returns.
pub type CommandResult<T> = Result<T, BridgeError>;

/// Logs `error` in full and keeps only what the UI needs.
pub fn bridged(error: Error) -> BridgeError {
    log::warn!("command failed: {error}");
    BridgeError::from(&error)
}

/// Runs `action` on the open workspace, or fails with `NoWorkspaceOpen`.
pub fn with_workspace<T>(
    state: &AppState,
    action: impl FnOnce(&Workspace) -> folio_core::Result<T>,
) -> CommandResult<T> {
    let workspace = state
        .workspace()
        .ok_or(Error::NoWorkspaceOpen)
        .map_err(bridged)?;
    action(&workspace).map_err(bridged)
}
