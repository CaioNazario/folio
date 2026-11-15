//! Choosing and opening the workspace (spec 03), then indexing it in the
//! background (spec 05).

use std::path::PathBuf;
use std::thread;

use folio_core::indexing::ScanOptions;
use folio_core::workspace::{Opened, Workspace, default_workspace_dir};
use tauri::{AppHandle, Emitter, Manager, State};

use super::{CommandResult, bridged};
use crate::state::AppState;

/// Emitted after each directory of the scan, with a `ScanProgress`.
const SCAN_PROGRESS: &str = "scan-progress";
/// Emitted when the scan and the hash pass are over; the UI reloads.
const SCAN_FINISHED: &str = "scan-finished";

/// The directory the first-run dialog suggests: `Folio/` in the system's
/// documents directory.
#[tauri::command]
pub fn get_default_workspace_dir(app: AppHandle, state: State<'_, AppState>) -> String {
    let documents = app.path().document_dir().ok();
    let dir = default_workspace_dir(documents.as_deref(), &state.services.home);
    dir.display().to_string()
}

/// Opens the last workspace. `false` on the first run, when the UI shows the
/// dialog to choose one.
#[tauri::command]
pub fn open_last_workspace(app: AppHandle, state: State<'_, AppState>) -> CommandResult<bool> {
    let opened = state.slot.open_last(&state.services).map_err(bridged)?;
    let Some(opened) = opened else {
        return Ok(false);
    };
    start(&app, opened);
    Ok(true)
}

/// Opens the workspace at `path`, which must exist.
#[tauri::command]
pub fn open_workspace(
    app: AppHandle,
    state: State<'_, AppState>,
    path: PathBuf,
) -> CommandResult<()> {
    let opened = state.slot.open(&path, &state.services).map_err(bridged)?;
    start(&app, opened);
    Ok(())
}

/// Opens the workspace at `path`, creating the directory if needed: the
/// confirmation of the first-run dialog.
#[tauri::command]
pub fn create_workspace(
    app: AppHandle,
    state: State<'_, AppState>,
    path: PathBuf,
) -> CommandResult<()> {
    let opened = state.slot.create(&path, &state.services).map_err(bridged)?;
    start(&app, opened);
    Ok(())
}

/// Indexes a workspace just opened on its own thread. A reused one is
/// already indexed (a reload of the webview, StrictMode in dev).
fn start(app: &AppHandle, opened: Opened) {
    if !opened.fresh {
        return;
    }
    let workspace = opened.workspace;
    let app = app.clone();
    thread::spawn(move || index(&app, &workspace));
}

/// The scan on opening, then the hash pass (spec 05). Failures are logged:
/// the next scan catches up.
fn index(app: &AppHandle, workspace: &Workspace) {
    let options = ScanOptions {
        remove_temp_files: true,
    };
    let mut report = |progress| _ = app.emit(SCAN_PROGRESS, progress);
    if let Err(error) = workspace.full_scan(options, &mut report) {
        log::warn!("scan failed: {error}");
    }
    if let Err(error) = workspace.hash_stale_documents() {
        log::warn!("hash pass failed: {error}");
    }
    _ = app.emit(SCAN_FINISHED, ());
}
