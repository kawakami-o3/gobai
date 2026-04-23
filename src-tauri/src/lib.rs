pub mod settings;

use crate::settings::{load_settings, Settings};
use tauri::State;

/// Wraps the load+validate result so failures can be surfaced to the UI
/// instead of crashing the app at startup. Cloning a `String` is cheap and
/// avoids any `Send`/`Sync` worries for the underlying error types.
pub struct SettingsState(pub Result<Settings, String>);

#[tauri::command]
fn get_settings(state: State<SettingsState>) -> Result<Settings, String> {
    state.0.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = SettingsState(load_settings().map_err(|e| e.to_string()));
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
