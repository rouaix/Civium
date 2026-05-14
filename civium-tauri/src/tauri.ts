import { invoke } from "@tauri-apps/api/core";

export async function tauriInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    if (String(e).includes("__TAURI_INTERNALS__") || String(e).includes("not a Tauri")) {
      throw new Error("Tauri IPC non disponible — lancez l'app via `cargo tauri dev`");
    }
    throw e;
  }
}
