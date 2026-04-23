pub mod settings;
pub mod storage;

use crate::settings::{load_settings, Settings};
use crate::storage::{init_storage, StoragePaths};
use tauri::{Manager, State};

/// Wraps the load+validate result so failures can be surfaced to the UI
/// instead of crashing the app at startup. Cloning a `String` is cheap and
/// avoids any `Send`/`Sync` worries for the underlying error types.
pub struct SettingsState(pub Result<Settings, String>);

/// Same shape as `SettingsState`: the storage init result is preserved as a
/// `Result` so errors propagate to the UI rather than aborting startup.
pub struct StorageState(pub Result<StoragePaths, String>);

#[tauri::command]
fn get_settings(state: State<SettingsState>) -> Result<Settings, String> {
    state.0.clone()
}

#[tauri::command]
fn get_storage_paths(state: State<StorageState>) -> Result<StoragePaths, String> {
    state.0.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings_state = SettingsState(load_settings().map_err(|e| e.to_string()));
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(settings_state)
        .setup(|app| {
            let storage_state = StorageState(init_storage(app.handle()).map_err(|e| e.to_string()));
            app.manage(storage_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_settings, get_storage_paths])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
