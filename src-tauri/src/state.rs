//! What the app builds once at startup and shares with every command.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use folio_core::clock::SystemClock;
use folio_core::db::GlobalDb;
use folio_core::ids::UuidV7Ids;
use folio_core::integrity::SystemTrash;
use folio_core::services::Services;
use folio_core::workspace::FolioDirStore;
use tauri::{AppHandle, Manager};

/// The app's shared state, managed by Tauri.
pub struct AppState {
    pub services: Services,
}

/// Opens the global database in `<data dir>/folio/` (spec 02) and builds the
/// production services around it.
pub fn build(app: &AppHandle) -> Result<AppState, Box<dyn std::error::Error>> {
    let paths = app.path();
    let data_dir = paths.data_dir()?.join("folio");
    fs::create_dir_all(&data_dir)?;
    let services = services(&paths.home_dir()?, &data_dir)?;
    Ok(AppState { services })
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
