//! Tauri commands: thin wrappers that deserialize, call `folio-core` and
//! return its result. Errors cross the bridge as `BridgeError` data; their
//! technical detail goes to the log.

/// Every command, for `tauri::generate_handler!`: a list, not logic, so it
/// lives in a macro and grows without one long function.
macro_rules! command_list {
    () => {
        tauri::generate_handler![
            $crate::commands::preferences::get_preferences,
            $crate::commands::preferences::set_language,
            $crate::commands::preferences::set_theme,
            $crate::commands::workspace::get_default_workspace_dir,
            $crate::commands::workspace::open_last_workspace,
            $crate::commands::workspace::open_workspace,
            $crate::commands::workspace::create_workspace,
            $crate::commands::library::list_dir,
            $crate::commands::library::get_favorites,
            $crate::commands::library::toggle_favorite_document,
            $crate::commands::library::toggle_favorite_dir,
            $crate::commands::library::get_recent_documents,
            $crate::commands::library::get_workspace_summary,
            $crate::commands::documents::record_opened,
            $crate::commands::documents::get_text_content,
            $crate::commands::documents::open_document,
            $crate::commands::documents::get_markdown_content,
            $crate::commands::documents::resolve_link,
            $crate::commands::documents::save_position,
        ]
    };
}

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
