//! Converting documents (spec 08): the queue runs in the core, and its
//! events reach the UI as `conversion` events.

use folio_core::Error;
use folio_core::documents::Format;
use tauri::State;
use uuid::Uuid;

use super::{CommandResult, bridged};
use crate::state::AppState;

/// Queues the conversion of the document `id` to `target` and returns the
/// job id its events carry. `password` opens a protected original.
#[tauri::command]
pub fn convert_document(
    state: State<'_, AppState>,
    id: Uuid,
    target: Format,
    password: Option<String>,
) -> CommandResult<u32> {
    let workspace = state
        .workspace()
        .ok_or(Error::NoWorkspaceOpen)
        .map_err(bridged)?;
    Ok(state.conversions.enqueue(workspace, id, target, password))
}

/// Cancels the job `job`, waiting or running.
#[tauri::command]
pub fn cancel_conversion(state: State<'_, AppState>, job: u32) {
    state.conversions.cancel(job);
}
