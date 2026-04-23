pub mod settings;

use crate::settings::{load_settings, Settings};
use tauri::State;

#[tauri::command]
fn get_settings(state: State<Settings>) -> Settings {
    state.inner().clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = load_settings().expect("failed to load settings");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(settings)
        .invoke_handler(tauri::generate_handler![get_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
