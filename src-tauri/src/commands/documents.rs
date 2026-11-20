//! Opening documents in the Reader (specs 06 and 07).

use folio_core::display::{
    DisplayText, DocumentProperties, DocumentView, LinkTarget, MarkdownContent,
};
use folio_core::recents::ReadingPosition;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
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

/// Follows a link clicked inside the document `id` (spec 07, Links): the
/// core says what it leads to, and an external one opens here, in the
/// system browser or mail app, so the webview never gets the opener. The
/// target goes back for the UI to handle the rest.
#[tauri::command]
pub fn follow_link(
    app: AppHandle,
    state: State<'_, AppState>,
    id: Uuid,
    href: String,
) -> CommandResult<LinkTarget> {
    let target = with_workspace(&state, |workspace| workspace.resolve_link(id, &href))?;
    if let LinkTarget::External { url } = &target {
        open_external(&app, url);
    }
    Ok(target)
}

/// Opens `url` with the system's handler; a failure only goes to the log,
/// like a click that the desktop did not answer.
fn open_external(app: &AppHandle, url: &str) {
    if let Err(error) = app.opener().open_url(url, None::<&str>) {
        log::warn!("cannot open external link: {error}");
    }
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
