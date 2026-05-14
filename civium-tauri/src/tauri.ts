// Wrapper autour de l'API Tauri injectée via withGlobalTauri.
// Utilise window.__TAURI__.core.invoke (Tauri 2.x avec withGlobalTauri: true).

declare global {
  interface Window {
    __TAURI__?: {
      core: {
        invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
      };
    };
  }
}

export function tauriInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  const tauri = window.__TAURI__?.core;
  if (!tauri) {
    return Promise.reject(
      new Error("Tauri IPC non disponible — lancez l'app via `cargo tauri dev`")
    );
  }
  return tauri.invoke<T>(cmd, args);
}
