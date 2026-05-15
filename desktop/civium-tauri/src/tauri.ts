import { invoke } from "@tauri-apps/api/core";

export function isTauriContext(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function tauriInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  return invoke<T>(cmd, args);
}
