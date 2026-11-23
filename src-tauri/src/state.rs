//! What the app builds once at startup and shares with every command.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use folio_core::clock::SystemClock;
use folio_core::conversion::ConversionQueue;
use folio_core::db::GlobalDb;
use folio_core::ids::UuidV7Ids;
use folio_core::integrity::SystemTrash;
use folio_core::services::Services;
use folio_core::workspace::{FolioDirStore, Workspace, WorkspaceSlot};
use tauri::{AppHandle, Emitter, Manager};

/// The app's shared state, managed by Tauri.
pub struct AppState {
    pub services: Services,
    /// The open workspace: one per window, and 0.1.0 has one window.
    pub slot: WorkspaceSlot,
    /// Conversions, one at a time in the background (spec 08).
    pub conversions: ConversionQueue,
}

/// The event that carries each [`folio_core::conversion::ConversionEvent`].
pub const CONVERSION_EVENT: &str = "conversion";

impl AppState {
    /// The open workspace, if any.
    pub fn workspace(&self) -> Option<Arc<Workspace>> {
        self.slot.current()
    }
}

/// Opens the global database in `<data dir>/folio/` (spec 02) and builds the
/// production services around it.
pub fn build(app: &AppHandle) -> Result<AppState, Box<dyn std::error::Error>> {
    let paths = app.path();
    let data_dir = paths.data_dir()?.join("folio");
    fs::create_dir_all(&data_dir)?;
    let services = services(&paths.home_dir()?, &data_dir)?;
    let emitter = app.clone();
    let conversions = ConversionQueue::new(move |event| {
        if let Err(error) = emitter.emit(CONVERSION_EVENT, event) {
            log::warn!("cannot emit a conversion event: {error}");
        }
    });
    Ok(AppState {
        services,
        slot: WorkspaceSlot::default(),
        conversions,
    })
}

fn services(home: &Path, data_dir: &Path) -> folio_core::Result<Services> {
    let clock = Arc::new(SystemClock);
    let ids = Arc::new(UuidV7Ids);
    let global = GlobalDb::open(&data_dir.join("folio.db"), clock.clone(), ids.clone())?;
    Ok(Services {
        home: home.to_path_buf(),
        global,
        store: Box::new(FolioDirStore),
        clock,
        ids,
        trash: Arc::new(SystemTrash),
    })
}
