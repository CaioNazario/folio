//! Interface language and theme (spec 02, `preferences`).

use folio_core::preferences::{Language, Preferences, Theme};
use tauri::State;

use super::{CommandResult, bridged};
use crate::state::AppState;

#[tauri::command]
pub fn get_preferences(state: State<'_, AppState>) -> CommandResult<Preferences> {
    state.services.global.preferences().map_err(bridged)
}

#[tauri::command]
pub fn set_language(state: State<'_, AppState>, language: Language) -> CommandResult<()> {
    state
        .services
        .global
        .set_language(language)
        .map_err(bridged)
}

#[tauri::command]
pub fn set_theme(state: State<'_, AppState>, theme: Theme) -> CommandResult<()> {
    state.services.global.set_theme(theme).map_err(bridged)
}
