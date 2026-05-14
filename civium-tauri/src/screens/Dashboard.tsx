import { useState, useEffect, useCallback, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { tauriInvoke } from "../tauri";
import type { NetworkInfo, MemberInfo, PendingMemberInfo, NodeStatus } from "../types";

export default function Dashboard() {
  const [networks, setNetworks] = useState<NetworkInfo[]>([]);
  const [selected, setSelected] = useState<NetworkInfo | null>(null);
  const [members, setMembers] = useState<MemberInfo[]>([]);
  const [pending, setPending] = useState<PendingMemberInfo[]>([]);
  const [inviteLink, setInviteLink] = useState<string | null>(null);
  const [loadingInvite, setLoadingInvite] = useState(false);
  const [admitting, setAdmitting] = useState<string | null>(null);
  const [nodeStatus, setNodeStatus] = useState<NodeStatus>({ running: false, listen_addrs: [] });
  const [syncing, setSyncing] = useState(false);

  // Keep a ref to selected so the event listener always has the latest value.
  const selectedRef = useRef<NetworkInfo | null>(null);
  useEffect(() => { selectedRef.current = selected; }, [selected]);

  useEffect(() => {
    tauriInvoke<NetworkInfo[]>("network_list").then(setNetworks);
  }, []);

  const refreshNetwork = useCallback((cid: string) => {
    tauriInvoke<MemberInfo[]>("member_list", { networkCid: cid }).then(setMembers);
    tauriInvoke<PendingMemberInfo[]>("member_pending_list", { networkCid: cid }).then(setPending);
    tauriInvoke<NetworkInfo[]>("network_list").then((nets) => {
      setNetworks(nets);
      const updated = nets.find((n) => n.cid_short === cid);
      if (updated) setSelected(updated);
    });
  }, []);

  useEffect(() => {
    if (!selected) return;
    refreshNetwork(selected.cid_short);
    setInviteLink(null);
  }, [selected?.cid_short]);

  // Poll node status + listen for sync-completed events.
  useEffect(() => {
    let mounted = true;

    const pollStatus = () => {
      tauriInvoke<NodeStatus>("node_status")
        .then((s) => { if (mounted) setNodeStatus(s); })
        .catch(() => {});
    };
    pollStatus();
    const interval = setInterval(pollStatus, 5000);

    let unlisten: UnlistenFn | null = null;
    listen<string>("civium://sync-completed", (event) => {
      const cid = event.payload;
      // Refresh the network list so member counts stay current.
      tauriInvoke<NetworkInfo[]>("network_list").then((nets) => {
        if (mounted) setNetworks(nets);
      });
      // If this network is currently selected, refresh its members too.
      if (selectedRef.current?.cid_short === cid) {
        refreshNetwork(cid);
      }
    }).then((fn) => { unlisten = fn; });

    return () => {
      mounted = false;
      clearInterval(interval);
      unlisten?.();
    };
  }, [refreshNetwork]);

  async function generateInvite() {
    if (!selected) return;
    setLoadingInvite(true);
    try {
      const link = await tauriInvoke<string>("network_invite", {
        networkCid: selected.cid_short,
        expiresIn: 0,
      });
      setInviteLink(link);
    } finally {
      setLoadingInvite(false);
    }
  }

  async function admitMember(cid: string, circle: number) {
    if (!selected) return;
    setAdmitting(cid);
    try {
      await tauriInvoke("member_admit", {
        networkCid: selected.cid_short,
        memberCid: cid,
        circle,
      });
      refreshNetwork(selected.cid_short);
    } catch (e) {
      alert(String(e));
    } finally {
      setAdmitting(null);
    }
  }

  async function rejectMember(cid: string) {
    if (!selected) return;
    try {
      await tauriInvoke("member_reject", {
        networkCid: selected.cid_short,
        memberCid: cid,
      });
      refreshNetwork(selected.cid_short);
    } catch (e) {
      alert(String(e));
    }
  }

  async function handleSync() {
    if (!selected) return;
    setSyncing(true);
    try {
      await tauriInvoke("node_sync", { networkCid: selected.cid_short });
    } catch (e) {
      console.error("Sync error:", e);
    } finally {
      setSyncing(false);
    }
  }

  const circleLabel = (c: number) =>
    ["Annuaire", "Connaissance", "Confiance"][c] ?? `Cercle ${c}`;

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <aside className="w-64 bg-civium-900 text-white flex flex-col">
        <div className="px-5 py-4 border-b border-civium-700">
          <h1 className="text-lg font-bold tracking-wide">Civium</h1>
          <p className="text-xs text-civium-100 mt-0.5">Phase 1</p>
        </div>
        <nav className="flex-1 overflow-y-auto px-3 py-3 space-y-1">
          {networks.length === 0 && (
            <p className="text-xs text-civium-100 px-2 py-2">Aucun réseau.</p>
          )}
          {networks.map((net) => (
            <button
              key={net.cid_short}
              onClick={() => setSelected(net)}
              className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                selected?.cid_short === net.cid_short
                  ? "bg-civium-600 text-white"
                  : "text-civium-100 hover:bg-civium-700"
              }`}
            >
              <div className="font-medium truncate">{net.name}</div>
              <div className="text-xs opacity-70">{net.member_count} membre(s)</div>
            </button>
          ))}
        </nav>

        {/* P2P node status */}
        <div className="px-4 py-3 border-t border-civium-700 space-y-0.5">
          <div className="flex items-center gap-2">
            <span
              className={`w-2 h-2 rounded-full flex-shrink-0 ${
                nodeStatus.running ? "bg-green-400" : "bg-gray-500"
              }`}
            />
            <span className="text-xs text-civium-100">
              {nodeStatus.running ? "En ligne" : "Hors ligne"}
            </span>
          </div>
          {nodeStatus.running && nodeStatus.listen_addrs.length > 0 && (
            <p className="text-xs text-civium-300 font-mono truncate pl-4">
              {nodeStatus.listen_addrs[0]}
            </p>
          )}
        </div>

        <div className="px-4 py-3 border-t border-civium-700">
          <p className="text-xs text-civium-100">
            CLI : <code className="font-mono">civium --help</code>
          </p>
        </div>
      </aside>

      {/* Main */}
      <main className="flex-1 overflow-y-auto">
        {!selected ? (
          <div className="flex items-center justify-center h-full text-gray-400">
            Sélectionnez un réseau
          </div>
        ) : (
          <div className="max-w-2xl mx-auto py-8 px-6 space-y-6">
            {/* Header */}
            <div className="flex items-start justify-between gap-4">
              <div>
                <h2 className="text-2xl font-bold text-gray-900">{selected.name}</h2>
                <p className="text-xs text-gray-400 font-mono mt-0.5">{selected.cid_short}</p>
              </div>
              {nodeStatus.running && (
                <button
                  onClick={handleSync}
                  disabled={syncing}
                  className="flex-shrink-0 text-xs px-3 py-1.5 bg-white border border-gray-200
                             text-gray-600 rounded-lg hover:bg-gray-50 disabled:opacity-50
                             transition-colors flex items-center gap-1.5"
                >
                  <span className={syncing ? "animate-spin inline-block" : ""}>↻</span>
                  {syncing ? "Synchronisation…" : "Synchroniser"}
                </button>
              )}
            </div>

            {/* Pending members (admin view) */}
            {pending.length > 0 && (
              <section>
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide mb-3">
                  Demandes en attente ({pending.length})
                </h3>
                <div className="bg-amber-50 border border-amber-200 rounded-xl divide-y divide-amber-100">
                  {pending.map((p) => (
                    <div key={p.cid_short} className="flex items-center px-4 py-3 gap-3">
                      <div className="w-8 h-8 rounded-full bg-amber-200 flex items-center justify-center
                                      text-amber-800 text-sm font-semibold">
                        {p.display_name[0]?.toUpperCase()}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="text-sm font-medium truncate">{p.display_name}</div>
                        <div className="text-xs text-gray-400 font-mono">{p.cid_short}</div>
                      </div>
                      <div className="flex gap-2">
                        <button
                          onClick={() => admitMember(p.cid_short, 1)}
                          disabled={admitting === p.cid_short}
                          className="text-xs px-2 py-1 bg-green-600 text-white rounded-lg
                                     hover:bg-green-700 disabled:opacity-50 transition-colors"
                        >
                          Admettre
                        </button>
                        <button
                          onClick={() => rejectMember(p.cid_short)}
                          disabled={admitting === p.cid_short}
                          className="text-xs px-2 py-1 bg-white border border-gray-300 text-gray-600
                                     rounded-lg hover:bg-gray-50 disabled:opacity-50 transition-colors"
                        >
                          Refuser
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </section>
            )}

            {/* Members */}
            <section>
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                  Membres ({members.length})
                </h3>
                <button
                  onClick={generateInvite}
                  disabled={loadingInvite}
                  className="text-xs px-3 py-1.5 bg-civium-600 text-white rounded-lg
                             hover:bg-civium-700 disabled:opacity-50 transition-colors"
                >
                  {loadingInvite ? "…" : "+ Inviter"}
                </button>
              </div>
              <div className="bg-white rounded-xl border border-gray-200 divide-y divide-gray-100">
                {members.length === 0 && (
                  <p className="px-4 py-3 text-sm text-gray-400">Aucun membre.</p>
                )}
                {members.map((m) => (
                  <div key={m.cid_short} className="flex items-center px-4 py-3 gap-3">
                    <div className="w-8 h-8 rounded-full bg-civium-100 flex items-center justify-center
                                    text-civium-700 text-sm font-semibold">
                      {m.display_name[0]?.toUpperCase()}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-medium truncate">{m.display_name}</div>
                      <div className="text-xs text-gray-400 font-mono">{m.cid_short}</div>
                    </div>
                    <div className="flex gap-1.5">
                      <span className="text-xs px-2 py-0.5 bg-gray-100 text-gray-600 rounded-full">
                        {circleLabel(m.circle)}
                      </span>
                      {m.role === "admin" && (
                        <span className="text-xs px-2 py-0.5 bg-amber-100 text-amber-700 rounded-full">
                          admin
                        </span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </section>

            {/* Invite link */}
            {inviteLink && (
              <section>
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide mb-2">
                  Lien d'invitation
                </h3>
                <div
                  className="bg-gray-50 border border-gray-200 rounded-xl p-4 text-xs font-mono
                             break-all text-gray-700 cursor-pointer hover:bg-gray-100 transition-colors"
                  onClick={() => navigator.clipboard.writeText(inviteLink)}
                  title="Cliquer pour copier"
                >
                  {inviteLink}
                </div>
                <p className="text-xs text-gray-400 mt-1">Cliquer pour copier</p>
              </section>
            )}
          </div>
        )}
      </main>
    </div>
  );
}
