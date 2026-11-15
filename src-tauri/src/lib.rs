#[macro_use]
mod commands;
mod state;

use tauri::ipc::Invoke;
use tauri::{App, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(setup)
        .invoke_handler(commands())
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}

/// Every command the UI can invoke.
fn commands() -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
    command_list!()
}

/// Logging in debug builds, then the app state the commands share.
fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        app.handle().plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )?;
    }
    app.manage(state::build(app.handle())?);
    Ok(())
}
