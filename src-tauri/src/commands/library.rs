//! What the sidebar shows: the tree, favorites, recents and the footer
//! (Design spec 03; specs 04 and 06).

use folio_core::favorites::Favorite;
use folio_core::paths::{RelativePath, tilde_path};
use folio_core::recents::RecentDocument;
use folio_core::workspace::{DirContents, WorkspaceSummary};
use tauri::State;
use uuid::Uuid;

use super::{CommandResult, with_workspace};
use crate::state::AppState;

/// Folders (with document counts) and documents directly inside `parent`;
/// the workspace root for `null`.
#[tauri::command]
pub fn list_dir(
    state: State<'_, AppState>,
    parent: Option<RelativePath>,
) -> CommandResult<DirContents> {
    with_workspace(&state, |workspace| workspace.dir_contents(parent.as_ref()))
}

#[tauri::command]
pub fn get_favorites(state: State<'_, AppState>) -> CommandResult<Vec<Favorite>> {
    with_workspace(&state, |workspace| workspace.favorites())
}

/// Whether the document is a favorite after the toggle.
#[tauri::command]
pub fn toggle_favorite_document(state: State<'_, AppState>, id: Uuid) -> CommandResult<bool> {
    with_workspace(&state, |workspace| workspace.toggle_favorite_document(id))
}

/// Whether the project or folder is a favorite after the toggle.
#[tauri::command]
pub fn toggle_favorite_dir(state: State<'_, AppState>, path: RelativePath) -> CommandResult<bool> {
    with_workspace(&state, |workspace| workspace.toggle_favorite_dir(&path))
}

/// The `limit` documents opened most recently, newest first.
#[tauri::command]
pub fn get_recent_documents(
    state: State<'_, AppState>,
    limit: usize,
) -> CommandResult<Vec<RecentDocument>> {
    with_workspace(&state, |workspace| workspace.recent_documents(limit))
}

/// Where the open workspace is, its totals and the user's name.
#[tauri::command]
pub fn get_workspace_summary(state: State<'_, AppState>) -> CommandResult<WorkspaceSummary> {
    let home = state.services.home.clone();
    with_workspace(&state, |workspace| {
        Ok(WorkspaceSummary {
            location: tilde_path(workspace.root(), &home),
            totals: workspace.workspace_totals()?,
            user_name: user_name(&home),
        })
    })
}

/// The login name, falling back to the home directory's name.
fn user_name(home: &std::path::Path) -> String {
    let from_env = std::env::var("USER").or_else(|_| std::env::var("LOGNAME"));
    from_env.unwrap_or_else(|_| {
        home.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    })
}
