import { invoke } from "@tauri-apps/api/core";

interface Settings {
  confirm_level: string;
}

interface StoragePaths {
  root: string;
  db: string;
  content_dir: string;
}

window.addEventListener("DOMContentLoaded", async () => {
  const settingsEl = document.querySelector("#settings-display");
  if (settingsEl) {
    try {
      const s = await invoke<Settings>("get_settings");
      settingsEl.textContent = `confirm_level: ${s.confirm_level}`;
    } catch (e) {
      settingsEl.textContent = `設定エラー: ${e}`;
    }
  }

  const storageEl = document.querySelector("#storage-display");
  if (storageEl) {
    try {
      const p = await invoke<StoragePaths>("get_storage_paths");
      storageEl.textContent = `db: ${p.db}`;
    } catch (e) {
      storageEl.textContent = `ストレージエラー: ${e}`;
    }
  }

  const dbEl = document.querySelector("#db-status");
  if (dbEl) {
    try {
      const status = await invoke<string>("get_db_status");
      dbEl.textContent = `db status: ${status}`;
    } catch (e) {
      dbEl.textContent = `DB エラー: ${e}`;
    }
  }
});
