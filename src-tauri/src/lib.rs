mod commands;
mod state;

use commands::preferences;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(state::build(app.handle())?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            preferences::get_preferences,
            preferences::set_language,
            preferences::set_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
