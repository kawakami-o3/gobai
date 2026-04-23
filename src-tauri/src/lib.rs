pub mod db;
pub mod settings;
pub mod storage;

use crate::settings::{load_settings, Settings};
use crate::storage::{init_storage, StoragePaths};
use sqlx::SqlitePool;
use tauri::{Manager, State};

/// Wraps the load+validate result so failures can be surfaced to the UI
/// instead of crashing the app at startup. Cloning a `String` is cheap and
/// avoids any `Send`/`Sync` worries for the underlying error types.
pub struct SettingsState(pub Result<Settings, String>);

/// Same shape as `SettingsState`: the storage init result is preserved as a
/// `Result` so errors propagate to the UI rather than aborting startup.
pub struct StorageState(pub Result<StoragePaths, String>);

/// SQLite pool wrapped in a `Result` so init failures (connect/migrate)
/// can be reported to the UI instead of aborting startup. `SqlitePool`
/// is internally `Arc` so cloning is cheap.
pub struct DbState(pub Result<SqlitePool, String>);

#[tauri::command]
fn get_settings(state: State<SettingsState>) -> Result<Settings, String> {
    state.0.clone()
}

#[tauri::command]
fn get_storage_paths(state: State<StorageState>) -> Result<StoragePaths, String> {
    state.0.clone()
}

#[tauri::command]
async fn get_db_status(state: State<'_, DbState>) -> Result<String, String> {
    let pool = state.0.as_ref().map_err(|e| e.clone())?.clone();
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master \
         WHERE type='table' AND name NOT LIKE '\\_%' ESCAPE '\\' \
         ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;
    let names: Vec<String> = rows.into_iter().map(|t| t.0).collect();
    Ok(format!(
        "connected ({} tables: {})",
        names.len(),
        names.join(", ")
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings_state = SettingsState(load_settings().map_err(|e| e.to_string()));
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(settings_state)
        .setup(|app| {
            let storage_result = init_storage(app.handle()).map_err(|e| e.to_string());
            let db_result = match &storage_result {
                Ok(paths) => tauri::async_runtime::block_on(db::init_pool(&paths.db))
                    .map_err(|e| e.to_string()),
                Err(e) => Err(format!("ストレージ未初期化: {e}")),
            };
            app.manage(StorageState(storage_result));
            app.manage(DbState(db_result));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            get_storage_paths,
            get_db_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
