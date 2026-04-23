import { invoke } from "@tauri-apps/api/core";

interface Settings {
  confirm_level: string;
}

window.addEventListener("DOMContentLoaded", async () => {
  const el = document.querySelector("#settings-display");
  if (!el) return;
  const s = await invoke<Settings>("get_settings");
  el.textContent = `confirm_level: ${s.confirm_level}`;
});
