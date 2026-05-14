import { useState, useEffect, useCallback, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { tauriInvoke } from "../tauri";
import type {
  NetworkInfo,
  MemberInfo,
  PendingMemberInfo,
  NodeStatus,
  MessageDisplay,
  ProposalInfo,
  VoteResultInfo,
} from "../types";

function formatTime(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString("fr-FR", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export default function Dashboard() {
  const [networks, setNetworks] = useState<NetworkInfo[]>([]);
  const [selected, setSelected] = useState<NetworkInfo | null>(null);
  const [members, setMembers] = useState<MemberInfo[]>([]);
  const [pending, setPending] = useState<PendingMemberInfo[]>([]);
  const [messages, setMessages] = useState<MessageDisplay[]>([]);
  const [msgBody, setMsgBody] = useState("");
  const [sending, setSending] = useState(false);
  const [inviteLink, setInviteLink] = useState<string | null>(null);
  const [loadingInvite, setLoadingInvite] = useState(false);
  const [admitting, setAdmitting] = useState<string | null>(null);
  const [nodeStatus, setNodeStatus] = useState<NodeStatus>({
    running: false,
    listen_addrs: [],
  });
  const [syncing, setSyncing] = useState(false);

  // Governance state
  const [proposals, setProposals] = useState<ProposalInfo[]>([]);
  const [showProposalForm, setShowProposalForm] = useState(false);
  const [propTitle, setPropTitle] = useState("");
  const [propDescription, setPropDescription] = useState("");
  const [propOptions, setPropOptions] = useState("Pour,Contre,Abstention");
  const [propHours, setPropHours] = useState(72);
  const [creatingProposal, setCreatingProposal] = useState(false);
  const [voteResults, setVoteResults] = useState<Record<string, VoteResultInfo>>({});
  const [voting, setVoting] = useState<string | null>(null);

  // Keep refs so event listeners always read the latest value.
  const selectedRef = useRef<NetworkInfo | null>(null);
  useEffect(() => {
    selectedRef.current = selected;
  }, [selected]);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when messages change.
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

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

  const refreshMessages = useCallback((cid: string) => {
    tauriInvoke<MessageDisplay[]>("message_list", { networkCid: cid }).then(setMessages);
  }, []);

  const refreshProposals = useCallback((cid: string) => {
    tauriInvoke<ProposalInfo[]>("proposal_list", { networkCid: cid }).then(setProposals);
  }, []);

  useEffect(() => {
    if (!selected) return;
    refreshNetwork(selected.cid_short);
    refreshMessages(selected.cid_short);
    refreshProposals(selected.cid_short);
    setInviteLink(null);
    setMessages([]);
    setProposals([]);
    setVoteResults({});
    setShowProposalForm(false);
  }, [selected?.cid_short]);

  // Poll node status + listen for sync-completed events.
  useEffect(() => {
    let mounted = true;

    const pollStatus = () => {
      tauriInvoke<NodeStatus>("node_status")
        .then((s) => {
          if (mounted) setNodeStatus(s);
        })
        .catch(() => {});
    };
    pollStatus();
    const interval = setInterval(pollStatus, 5000);

    let unlisten: UnlistenFn | null = null;
    listen<string>("civium://sync-completed", (event) => {
      const cid = event.payload;
      tauriInvoke<NetworkInfo[]>("network_list").then((nets) => {
        if (mounted) setNetworks(nets);
      });
      if (selectedRef.current?.cid_short === cid) {
        refreshNetwork(cid);
        refreshMessages(cid);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      mounted = false;
      clearInterval(interval);
      unlisten?.();
    };
  }, [refreshNetwork, refreshMessages]);

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

  async function handleCreateProposal() {
    if (!selected || !propTitle.trim()) return;
    setCreatingProposal(true);
    try {
      const opts = propOptions
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      if (opts.length < 2) { alert("Au moins 2 options requises."); return; }
      await tauriInvoke("proposal_create", {
        networkCid: selected.cid_short,
        title: propTitle.trim(),
        description: propDescription.trim(),
        options: opts,
        hours: propHours,
        quorumPercent: 0,
      });
      setPropTitle("");
      setPropDescription("");
      setPropOptions("Pour,Contre,Abstention");
      setPropHours(72);
      setShowProposalForm(false);
      refreshProposals(selected.cid_short);
    } catch (e) {
      alert(String(e));
    } finally {
      setCreatingProposal(false);
    }
  }

  async function handleVote(proposalId: string, choiceIndex: number) {
    if (!selected) return;
    setVoting(proposalId);
    try {
      await tauriInvoke("vote_cast", {
        networkCid: selected.cid_short,
        proposalId,
        choiceIndex,
      });
      const result = await tauriInvoke<VoteResultInfo>("vote_results", {
        networkCid: selected.cid_short,
        proposalId,
      });
      setVoteResults((prev) => ({ ...prev, [proposalId]: result }));
    } catch (e) {
      alert(String(e));
    } finally {
      setVoting(null);
    }
  }

  async function loadResults(proposalId: string) {
    if (!selected) return;
    try {
      const result = await tauriInvoke<VoteResultInfo>("vote_results", {
        networkCid: selected.cid_short,
        proposalId,
      });
      setVoteResults((prev) => ({ ...prev, [proposalId]: result }));
    } catch {}
  }

  async function handleSendMessage() {
    if (!selected || !msgBody.trim()) return;
    setSending(true);
    try {
      const msg = await tauriInvoke<MessageDisplay>("message_send", {
        networkCid: selected.cid_short,
        body: msgBody.trim(),
      });
      setMessages((prev) => [...prev, msg]);
      setMsgBody("");
    } catch (e) {
      alert(String(e));
    } finally {
      setSending(false);
    }
  }

  function handleMsgKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
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

            {/* Pending members */}
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

            {/* Governance */}
            <section>
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                  Propositions ({proposals.length})
                </h3>
                <button
                  onClick={() => setShowProposalForm((v) => !v)}
                  className="text-xs px-3 py-1.5 bg-civium-600 text-white rounded-lg
                             hover:bg-civium-700 transition-colors"
                >
                  {showProposalForm ? "Annuler" : "+ Proposer"}
                </button>
              </div>

              {/* New proposal form */}
              {showProposalForm && (
                <div className="bg-white border border-gray-200 rounded-xl p-4 mb-3 space-y-3">
                  <div>
                    <label className="block text-xs font-medium text-gray-700 mb-1">Titre</label>
                    <input
                      type="text"
                      value={propTitle}
                      onChange={(e) => setPropTitle(e.target.value)}
                      placeholder="ex. Changer le nom du réseau"
                      className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm
                                 focus:outline-none focus:ring-2 focus:ring-civium-400"
                    />
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-gray-700 mb-1">Description (optionnel)</label>
                    <textarea
                      value={propDescription}
                      onChange={(e) => setPropDescription(e.target.value)}
                      rows={2}
                      className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm resize-none
                                 focus:outline-none focus:ring-2 focus:ring-civium-400"
                    />
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-gray-700 mb-1">
                      Options (séparées par des virgules)
                    </label>
                    <input
                      type="text"
                      value={propOptions}
                      onChange={(e) => setPropOptions(e.target.value)}
                      className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm
                                 focus:outline-none focus:ring-2 focus:ring-civium-400"
                    />
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-gray-700 mb-1">
                      Durée (heures, 0 = illimitée)
                    </label>
                    <input
                      type="number"
                      value={propHours}
                      onChange={(e) => setPropHours(Number(e.target.value))}
                      min={0}
                      className="w-24 border border-gray-200 rounded-lg px-3 py-2 text-sm
                                 focus:outline-none focus:ring-2 focus:ring-civium-400"
                    />
                  </div>
                  <button
                    onClick={handleCreateProposal}
                    disabled={creatingProposal || !propTitle.trim()}
                    className="w-full py-2 bg-civium-600 text-white rounded-lg text-sm font-medium
                               hover:bg-civium-700 disabled:opacity-50 transition-colors"
                  >
                    {creatingProposal ? "Création…" : "Créer la proposition"}
                  </button>
                </div>
              )}

              {/* Proposals list */}
              {proposals.length === 0 && !showProposalForm && (
                <p className="text-sm text-gray-400">Aucune proposition. Créez-en une !</p>
              )}
              <div className="space-y-3">
                {proposals.map((prop) => {
                  const result = voteResults[prop.id];
                  return (
                    <div key={prop.id} className="bg-white border border-gray-200 rounded-xl p-4 space-y-3">
                      <div className="flex items-start justify-between gap-2">
                        <div>
                          <p className="text-sm font-semibold text-gray-900">{prop.title}</p>
                          {prop.description && (
                            <p className="text-xs text-gray-500 mt-0.5">{prop.description}</p>
                          )}
                        </div>
                        <span className={`flex-shrink-0 text-xs px-2 py-0.5 rounded-full ${
                          prop.status === "open"
                            ? "bg-green-100 text-green-700"
                            : "bg-gray-100 text-gray-500"
                        }`}>
                          {prop.status}
                        </span>
                      </div>

                      {/* Vote buttons */}
                      {prop.status === "open" && (
                        <div className="flex flex-wrap gap-2">
                          {prop.options.map((opt, i) => (
                            <button
                              key={i}
                              onClick={() => handleVote(prop.id, i)}
                              disabled={voting === prop.id}
                              className="text-xs px-3 py-1.5 bg-civium-50 border border-civium-200
                                         text-civium-700 rounded-lg hover:bg-civium-100
                                         disabled:opacity-50 transition-colors"
                            >
                              {opt}
                            </button>
                          ))}
                          <button
                            onClick={() => loadResults(prop.id)}
                            className="text-xs px-3 py-1.5 bg-white border border-gray-200
                                       text-gray-500 rounded-lg hover:bg-gray-50 transition-colors"
                          >
                            Voir résultats
                          </button>
                        </div>
                      )}

                      {/* Results */}
                      {result && (
                        <div className="space-y-1.5 pt-1 border-t border-gray-100">
                          <p className="text-xs text-gray-500">
                            {result.total_votes}/{result.total_members} vote(s) —{" "}
                            {result.participation_percent.toFixed(1)}% participation
                            {!result.quorum_reached && (
                              <span className="ml-1 text-amber-600">(quorum non atteint)</span>
                            )}
                          </p>
                          {result.options.map((opt, i) => (
                            <div key={i} className="flex items-center gap-2">
                              <span className={`text-xs w-20 truncate ${
                                result.winner === i ? "font-semibold text-civium-700" : "text-gray-600"
                              }`}>
                                {opt.label}
                              </span>
                              <div className="flex-1 bg-gray-100 rounded-full h-2">
                                <div
                                  className={`h-2 rounded-full transition-all ${
                                    result.winner === i ? "bg-civium-500" : "bg-gray-300"
                                  }`}
                                  style={{ width: `${opt.percent}%` }}
                                />
                              </div>
                              <span className="text-xs text-gray-500 w-12 text-right">
                                {opt.percent.toFixed(0)}%
                              </span>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </section>

            {/* Thread messages */}
            <section>
              <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide mb-3">
                Fil de discussion
                {messages.length > 0 && (
                  <span className="ml-1 font-normal text-gray-400 normal-case">
                    ({messages.filter((m) => !m.is_direct).length})
                  </span>
                )}
              </h3>

              {/* Message list */}
              <div className="bg-white rounded-t-xl border border-b-0 border-gray-200
                              max-h-72 overflow-y-auto p-4 space-y-4">
                {messages.filter((m) => !m.is_direct).length === 0 ? (
                  <p className="text-sm text-gray-400 text-center py-4">
                    Aucun message. Soyez le premier à écrire !
                  </p>
                ) : (
                  messages
                    .filter((m) => !m.is_direct)
                    .map((msg) => (
                      <div key={msg.id} className="flex gap-3">
                        <div className="w-7 h-7 rounded-full bg-civium-100 flex-shrink-0 flex
                                        items-center justify-center text-civium-700 text-xs font-semibold">
                          {msg.author_name[0]?.toUpperCase()}
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-baseline gap-2">
                            <span className="text-sm font-medium text-gray-900">
                              {msg.author_name}
                            </span>
                            <span className="text-xs text-gray-400">{formatTime(msg.sent_at)}</span>
                          </div>
                          <p className="text-sm text-gray-700 mt-0.5 whitespace-pre-wrap break-words">
                            {msg.body}
                          </p>
                        </div>
                      </div>
                    ))
                )}
                <div ref={messagesEndRef} />
              </div>

              {/* Send form */}
              <div className="flex gap-2 bg-white border border-gray-200 rounded-b-xl p-3">
                <textarea
                  value={msgBody}
                  onChange={(e) => setMsgBody(e.target.value)}
                  onKeyDown={handleMsgKeyDown}
                  placeholder="Écrire un message… (Entrée pour envoyer, Maj+Entrée pour sauter une ligne)"
                  rows={2}
                  disabled={sending}
                  className="flex-1 text-sm resize-none border border-gray-200 rounded-lg px-3 py-2
                             focus:outline-none focus:ring-2 focus:ring-civium-400 disabled:opacity-50
                             placeholder:text-gray-400"
                />
                <button
                  onClick={handleSendMessage}
                  disabled={sending || !msgBody.trim()}
                  className="self-end text-xs px-4 py-2 bg-civium-600 text-white rounded-lg
                             hover:bg-civium-700 disabled:opacity-50 transition-colors font-medium"
                >
                  {sending ? "…" : "Envoyer"}
                </button>
              </div>
            </section>
          </div>
        )}
      </main>
    </div>
  );
}
