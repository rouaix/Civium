import { invoke } from "@tauri-apps/api/core";

export function isTauriContext(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Commands that involve P2P networking and can legitimately take up to 35 s. */
const P2P_COMMANDS = new Set([
  "network_join", "network_join_p2p", "node_sync",
  "connection_accept", "connection_refuse", "connection_block", "connection_revoke",
  "hub_sync", "hub_network_register", "hub_member_join",
  "rcc_register", "rcc_force_retry",
]);

/** Default: 10 s for local commands, 35 s for P2P commands. */
function timeoutFor(cmd: string): number {
  return P2P_COMMANDS.has(cmd) ? 35_000 : 10_000;
}

export async function tauriInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  const ms = timeoutFor(cmd);
  return Promise.race([
    invoke<T>(cmd, args),
    new Promise<never>((_, reject) =>
      setTimeout(() => reject(new Error(`Délai dépassé (${ms / 1000} s) pour la commande « ${cmd} »`)), ms)
    ),
  ]);
}
