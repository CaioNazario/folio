//! Opening documents in the Reader (specs 06 and 07).

use folio_core::display::{
    DisplayText, DocumentProperties, DocumentView, LinkTarget, MarkdownContent,
};
use folio_core::recents::ReadingPosition;
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

/// The document with what the view shows around the stage: breadcrumb,
/// derived badge, conversion targets and where the user stopped.
#[tauri::command]
pub fn open_document(state: State<'_, AppState>, id: Uuid) -> CommandResult<DocumentView> {
    with_workspace(&state, |workspace| workspace.document_view(id))
}

/// Everything Folio knows about a document, for the Properties dialog:
/// file, format metadata and conversions (spec 07).
#[tauri::command]
pub fn get_properties(state: State<'_, AppState>, id: Uuid) -> CommandResult<DocumentProperties> {
    with_workspace(&state, |workspace| workspace.document_properties(id))
}

/// The HTML of a Markdown document, sanitized and with its images resolved.
#[tauri::command]
pub fn get_markdown_content(
    state: State<'_, AppState>,
    id: Uuid,
) -> CommandResult<MarkdownContent> {
    with_workspace(&state, |workspace| workspace.markdown_content(id))
}

/// What a link clicked inside the document `id` leads to (spec 07, Links).
#[tauri::command]
pub fn resolve_link(
    state: State<'_, AppState>,
    id: Uuid,
    href: String,
) -> CommandResult<LinkTarget> {
    with_workspace(&state, |workspace| workspace.resolve_link(id, &href))
}

/// Where the user stopped in the document, saved when leaving it (spec 06).
#[tauri::command]
pub fn save_position(
    state: State<'_, AppState>,
    id: Uuid,
    position: ReadingPosition,
) -> CommandResult<()> {
    with_workspace(&state, |workspace| workspace.save_position(id, position))
}
