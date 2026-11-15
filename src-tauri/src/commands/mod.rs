//! Tauri commands: thin wrappers that deserialize, call `folio-core` and
//! return its result. Errors cross the bridge as `BridgeError` data; their
//! technical detail goes to the log.

pub mod preferences;

use folio_core::Error;
use folio_core::bridge_error::BridgeError;

/// The result every command returns.
pub type CommandResult<T> = Result<T, BridgeError>;

/// Logs `error` in full and keeps only what the UI needs.
pub fn bridged(error: Error) -> BridgeError {
    log::warn!("command failed: {error}");
    BridgeError::from(&error)
}
