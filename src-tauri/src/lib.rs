mod commands;
mod state;

use commands::{documents, library, preferences, workspace};
use tauri::ipc::Invoke;
use tauri::{App, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .setup(setup)
        .invoke_handler(commands())
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}

/// Every command the UI can invoke.
fn commands() -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        preferences::get_preferences,
        preferences::set_language,
        preferences::set_theme,
        workspace::get_default_workspace_dir,
        workspace::open_last_workspace,
        workspace::open_workspace,
        workspace::create_workspace,
        library::list_dir,
        library::get_favorites,
        library::toggle_favorite_document,
        library::toggle_favorite_dir,
        library::get_recent_documents,
        library::get_workspace_summary,
        documents::record_opened,
        documents::get_text_content,
    ]
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
