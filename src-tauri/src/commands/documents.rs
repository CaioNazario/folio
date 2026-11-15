//! Opening documents in the Reader (specs 06 and 07).

use folio_core::display::DisplayText;
use tauri::State;
use uuid::Uuid;

use super::{CommandResult, with_workspace};
use crate::state::AppState;

/// Records that the Reader is showing the document now (spec 06): it goes
/// to the top of the recents. Invalid documents are ignored.
#[tauri::command]
pub fn record_opened(state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    with_workspace(&state, |workspace| workspace.record_opened(id))
}

/// The text of a TXT document, cut past 10 MB (spec 07).
#[tauri::command]
pub fn get_text_content(state: State<'_, AppState>, id: Uuid) -> CommandResult<DisplayText> {
    with_workspace(&state, |workspace| workspace.text_content(id))
}
