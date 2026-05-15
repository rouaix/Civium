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
  AdminActionInfo,
  DelegationInfo,
  DirectoryEntryInfo,
  FederationInfo,
  GuardianLinkInfo,
  RrmEntryInfo,
  TrustedRrmInfo,
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

  // Delegation state
  const [myDelegations, setMyDelegations] = useState<DelegationInfo[]>([]);
  const [delegatingTo, setDelegatingTo] = useState<Record<string, string>>({}); // proposalId|"global" → cid
  const [savingDelegation, setSavingDelegation] = useState<string | null>(null);

  // Garde-fou state
  const [adminActions, setAdminActions] = useState<AdminActionInfo[]>([]);
  const [contesting, setContesting] = useState<string | null>(null);
  const [now, setNow] = useState(() => Math.floor(Date.now() / 1000));

  // Directory state
  const [dirEntries, setDirEntries] = useState<DirectoryEntryInfo[]>([]);
  const [dirSearchQuery, setDirSearchQuery] = useState("");
  const [dirSearchResults, setDirSearchResults] = useState<DirectoryEntryInfo[] | null>(null);
  const [showPublishForm, setShowPublishForm] = useState(false);
  const [pubSubjectCid, setPubSubjectCid] = useState("");
  const [pubSubjectName, setPubSubjectName] = useState("");
  const [pubDescription, setPubDescription] = useState("");
  const [pubKind, setPubKind] = useState<"network" | "member">("network");
  const [pubTags, setPubTags] = useState("");
  const [publishing, setPublishing] = useState(false);

  // Federation state
  const [federations, setFederations] = useState<FederationInfo[]>([]);
  const [showFedForm, setShowFedForm] = useState(false);
  const [fedPeerCid, setFedPeerCid] = useState("");
  const [fedPeerName, setFedPeerName] = useState("");
  const [fedPeerAddr, setFedPeerAddr] = useState("");
  const [savingFed, setSavingFed] = useState(false);
  const [includeFederated, setIncludeFederated] = useState(false);

  // RRM state
  const [rrmEntries, setRrmEntries] = useState<RrmEntryInfo[]>([]);
  const [showReportForm, setShowReportForm] = useState(false);
  const [reportNetCid, setReportNetCid] = useState("");
  const [reportNetName, setReportNetName] = useState("");
  const [reportReason, setReportReason] = useState("");
  const [reportEvidence, setReportEvidence] = useState("");
  const [reporting, setReporting] = useState(false);

  // Trusted RRMs state (for standard networks)
  const [trustedRrms, setTrustedRrms] = useState<TrustedRrmInfo[]>([]);
  const [showTrustForm, setShowTrustForm] = useState(false);
  const [trustRrmCid, setTrustRrmCid] = useState("");
  const [trustRrmName, setTrustRrmName] = useState("");
  const [savingTrust, setSavingTrust] = useState(false);

  // Minor / Guardian state
  const [expandedMember, setExpandedMember] = useState<string | null>(null);
  const [guardians, setGuardians] = useState<Record<string, GuardianLinkInfo[]>>({});
  const [settingMinor, setSettingMinor] = useState<string | null>(null);
  const [newGuardianCid, setNewGuardianCid] = useState("");
  const [savingGuardian, setSavingGuardian] = useState(false);

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

  const refreshAdminActions = useCallback((cid: string) => {
    tauriInvoke<AdminActionInfo[]>("admin_action_list", { networkCid: cid }).then(setAdminActions);
  }, []);

  const refreshDelegations = useCallback((cid: string) => {
    tauriInvoke<DelegationInfo[]>("vote_list_delegations", { networkCid: cid }).then(setMyDelegations);
  }, []);

  const refreshDirEntries = useCallback((cid: string) => {
    tauriInvoke<DirectoryEntryInfo[]>("directory_list", { directoryCid: cid }).then(setDirEntries);
  }, []);

  const refreshFederations = useCallback((cid: string) => {
    tauriInvoke<FederationInfo[]>("directory_federations", { directoryCid: cid }).then(setFederations);
  }, []);

  const refreshRrmEntries = useCallback((cid: string) => {
    tauriInvoke<RrmEntryInfo[]>("rrm_list", { rrmCid: cid }).then(setRrmEntries);
  }, []);

  const refreshTrustedRrms = useCallback((cid: string) => {
    tauriInvoke<TrustedRrmInfo[]>("network_trusted_rrms", { networkCid: cid }).then(setTrustedRrms);
  }, []);

  useEffect(() => {
    if (!selected) return;
    refreshNetwork(selected.cid_short);
    refreshMessages(selected.cid_short);
    refreshProposals(selected.cid_short);
    refreshAdminActions(selected.cid_short);
    refreshDelegations(selected.cid_short);
    if (selected.is_directory) {
      refreshDirEntries(selected.cid_short);
      refreshFederations(selected.cid_short);
    }
    if (selected.is_rrm) {
      refreshRrmEntries(selected.cid_short);
    }
    if (!selected.is_directory && !selected.is_rrm) {
      refreshTrustedRrms(selected.cid_short);
    }
    setInviteLink(null);
    setMessages([]);
    setProposals([]);
    setVoteResults({});
    setShowProposalForm(false);
    setAdminActions([]);
    setMyDelegations([]);
    setDirEntries([]);
    setDirSearchResults(null);
    setDirSearchQuery("");
    setShowPublishForm(false);
    setFederations([]);
    setShowFedForm(false);
    setIncludeFederated(false);
    setRrmEntries([]);
    setShowReportForm(false);
    setTrustedRrms([]);
    setShowTrustForm(false);
    setExpandedMember(null);
    setGuardians({});
    setNewGuardianCid("");
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

  async function handleDelegate(proposalId: string | null, delegateCid: string) {
    if (!selected || !delegateCid.trim()) return;
    const key = proposalId ?? "global";
    setSavingDelegation(key);
    try {
      await tauriInvoke("vote_delegate", {
        networkCid: selected.cid_short,
        delegateCidShort: delegateCid.trim(),
        proposalId,
      });
      refreshDelegations(selected.cid_short);
      setDelegatingTo((prev) => ({ ...prev, [key]: "" }));
    } catch (e) {
      alert(String(e));
    } finally {
      setSavingDelegation(null);
    }
  }

  async function handleRevokeDelegation(proposalId: string | null) {
    if (!selected) return;
    try {
      await tauriInvoke("vote_revoke_delegation", {
        networkCid: selected.cid_short,
        proposalId,
      });
      refreshDelegations(selected.cid_short);
    } catch (e) {
      alert(String(e));
    }
  }

  // Update "now" every minute so countdown stays fresh.
  useEffect(() => {
    const t = setInterval(() => setNow(Math.floor(Date.now() / 1000)), 60_000);
    return () => clearInterval(t);
  }, []);

  async function handleContest(actionId: string) {
    if (!selected) return;
    setContesting(actionId);
    try {
      const updated = await tauriInvoke<AdminActionInfo>("admin_action_contest", {
        networkCid: selected.cid_short,
        actionId,
      });
      setAdminActions((prev) => prev.map((a) => (a.id === actionId ? updated : a)));
      if (updated.status === "suspended") {
        // Refresh proposals so the auto-created one appears
        refreshProposals(selected.cid_short);
      }
    } catch (e) {
      alert(String(e));
    } finally {
      setContesting(null);
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

  async function handleFederate() {
    if (!selected || !fedPeerCid.trim() || !fedPeerName.trim()) return;
    setSavingFed(true);
    try {
      const fed = await tauriInvoke<FederationInfo>("directory_federate", {
        directoryCid: selected.cid_short,
        peerCid: fedPeerCid.trim(),
        peerName: fedPeerName.trim(),
        peerAddr: fedPeerAddr.trim() || null,
      });
      setFederations((prev) => [...prev, fed]);
      setFedPeerCid("");
      setFedPeerName("");
      setFedPeerAddr("");
      setShowFedForm(false);
    } catch (e) {
      alert(String(e));
    } finally {
      setSavingFed(false);
    }
  }

  async function handleUnfederate(peerCid: string) {
    if (!selected) return;
    try {
      await tauriInvoke("directory_unfederate", {
        directoryCid: selected.cid_short,
        peerCid,
      });
      setFederations((prev) => prev.filter((f) => f.peer_cid_short !== peerCid));
    } catch (e) {
      alert(String(e));
    }
  }

  async function handleDirSearch() {
    if (!selected || !dirSearchQuery.trim()) return;
    try {
      const results = await tauriInvoke<DirectoryEntryInfo[]>("directory_search", {
        directoryCid: selected.cid_short,
        query: dirSearchQuery.trim(),
        includeFederated,
      });
      setDirSearchResults(results);
    } catch (e) {
      alert(String(e));
    }
  }

  async function handlePublish() {
    if (!selected || !pubSubjectCid.trim() || !pubSubjectName.trim()) return;
    setPublishing(true);
    try {
      const tags = pubTags.split(",").map((t) => t.trim()).filter(Boolean);
      const entry = await tauriInvoke<DirectoryEntryInfo>("directory_publish", {
        directoryCid: selected.cid_short,
        kind: pubKind,
        subjectCidShort: pubSubjectCid.trim(),
        subjectName: pubSubjectName.trim(),
        description: pubDescription.trim(),
        contactAddr: null,
        tags,
      });
      setDirEntries((prev) => [...prev, entry]);
      setPubSubjectCid("");
      setPubSubjectName("");
      setPubDescription("");
      setPubTags("");
      setShowPublishForm(false);
    } catch (e) {
      alert(String(e));
    } finally {
      setPublishing(false);
    }
  }

  async function handleRemoveEntry(entryId: string) {
    if (!selected) return;
    try {
      await tauriInvoke("directory_remove", {
        directoryCid: selected.cid_short,
        entryId,
      });
      setDirEntries((prev) => prev.filter((e) => e.id !== entryId));
      setDirSearchResults((prev) => prev ? prev.filter((e) => e.id !== entryId) : null);
    } catch (e) {
      alert(String(e));
    }
  }

  async function handleReport() {
    if (!selected || !reportNetCid.trim() || !reportNetName.trim() || !reportReason.trim()) return;
    setReporting(true);
    try {
      const entry = await tauriInvoke<RrmEntryInfo>("rrm_report", {
        rrmCid: selected.cid_short,
        networkCidShort: reportNetCid.trim(),
        networkName: reportNetName.trim(),
        reason: reportReason.trim(),
        evidenceUrl: reportEvidence.trim() || null,
      });
      setRrmEntries((prev) => [entry, ...prev]);
      setReportNetCid("");
      setReportNetName("");
      setReportReason("");
      setReportEvidence("");
      setShowReportForm(false);
    } catch (e) {
      alert(String(e));
    } finally {
      setReporting(false);
    }
  }

  async function handleRemoveRrmEntry(entryId: string) {
    if (!selected) return;
    try {
      await tauriInvoke("rrm_remove", { rrmCid: selected.cid_short, entryId });
      setRrmEntries((prev) => prev.filter((e) => e.id !== entryId));
    } catch (e) {
      alert(String(e));
    }
  }

  async function handleTrustRrm() {
    if (!selected || !trustRrmCid.trim() || !trustRrmName.trim()) return;
    setSavingTrust(true);
    try {
      const trust = await tauriInvoke<TrustedRrmInfo>("network_trust_rrm", {
        networkCid: selected.cid_short,
        rrmCid: trustRrmCid.trim(),
        rrmName: trustRrmName.trim(),
      });
      setTrustedRrms((prev) => [...prev, trust]);
      setTrustRrmCid("");
      setTrustRrmName("");
      setShowTrustForm(false);
    } catch (e) {
      alert(String(e));
    } finally {
      setSavingTrust(false);
    }
  }

  async function handleUntrustRrm(rrmCid: string) {
    if (!selected) return;
    try {
      await tauriInvoke("network_untrust_rrm", { networkCid: selected.cid_short, rrmCid });
      setTrustedRrms((prev) => prev.filter((t) => t.rrm_cid_short !== rrmCid));
    } catch (e) {
      alert(String(e));
    }
  }

  async function handleToggleMinor(memberCid: string, isMinor: boolean) {
    if (!selected) return;
    setSettingMinor(memberCid);
    try {
      await tauriInvoke("member_set_minor", {
        networkCid: selected.cid_short,
        memberCid,
        isMinor,
      });
      refreshNetwork(selected.cid_short);
      if (!isMinor) {
        setGuardians((prev) => { const n = { ...prev }; delete n[memberCid]; return n; });
      }
    } catch (e) {
      alert(String(e));
    } finally {
      setSettingMinor(null);
    }
  }

  async function handleExpandMember(memberCid: string) {
    if (expandedMember === memberCid) { setExpandedMember(null); return; }
    setExpandedMember(memberCid);
    if (!selected) return;
    const links = await tauriInvoke<GuardianLinkInfo[]>("member_guardians", {
      networkCid: selected.cid_short,
      minorCid: memberCid,
    }).catch(() => []);
    setGuardians((prev) => ({ ...prev, [memberCid]: links }));
  }

  async function handleAddGuardian(minorCid: string) {
    if (!selected || !newGuardianCid.trim()) return;
    setSavingGuardian(true);
    try {
      const link = await tauriInvoke<GuardianLinkInfo>("member_set_guardian", {
        networkCid: selected.cid_short,
        minorCid,
        guardianCid: newGuardianCid.trim(),
      });
      setGuardians((prev) => ({ ...prev, [minorCid]: [...(prev[minorCid] ?? []), link] }));
      setNewGuardianCid("");
    } catch (e) {
      alert(String(e));
    } finally {
      setSavingGuardian(false);
    }
  }

  async function handleRemoveGuardian(minorCid: string, guardianCid: string) {
    if (!selected) return;
    try {
      await tauriInvoke("member_remove_guardian", {
        networkCid: selected.cid_short,
        minorCid,
        guardianCid,
      });
      setGuardians((prev) => ({
        ...prev,
        [minorCid]: (prev[minorCid] ?? []).filter((l) => l.guardian_cid_short !== guardianCid),
      }));
    } catch (e) {
      alert(String(e));
    }
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
              <div className="font-medium truncate flex items-center gap-1.5">
                {net.name}
                {net.is_directory && (
                  <span className="text-xs bg-civium-500 text-white px-1 py-0.5 rounded">Annuaire</span>
                )}
                {net.is_rrm && (
                  <span className="text-xs bg-red-600 text-white px-1 py-0.5 rounded">RRM</span>
                )}
              </div>
              <div className="text-xs opacity-70">
                {net.is_directory ? "Annuaire" : net.is_rrm ? "Registre malveillants" : `${net.member_count} membre(s)`}
              </div>
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
                  <div key={m.cid_short}>
                    <div
                      className="flex items-center px-4 py-3 gap-3 cursor-pointer hover:bg-gray-50 transition-colors"
                      onClick={() => handleExpandMember(m.cid_short)}
                    >
                      <div className="w-8 h-8 rounded-full bg-civium-100 flex items-center justify-center
                                      text-civium-700 text-sm font-semibold shrink-0">
                        {m.display_name[0]?.toUpperCase()}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="text-sm font-medium truncate">{m.display_name}</div>
                        <div className="text-xs text-gray-400 font-mono">{m.cid_short}</div>
                      </div>
                      <div className="flex gap-1.5 shrink-0">
                        <span className="text-xs px-2 py-0.5 bg-gray-100 text-gray-600 rounded-full">
                          {circleLabel(m.circle)}
                        </span>
                        {m.role === "admin" && (
                          <span className="text-xs px-2 py-0.5 bg-amber-100 text-amber-700 rounded-full">
                            admin
                          </span>
                        )}
                        {m.is_minor && (
                          <span className="text-xs px-2 py-0.5 bg-blue-100 text-blue-700 rounded-full">
                            mineur
                          </span>
                        )}
                      </div>
                    </div>
                    {expandedMember === m.cid_short && (
                      <div className="px-4 pb-3 ml-11 space-y-2 border-b border-gray-100">
                        <div className="flex items-center gap-2">
                          <span className="text-xs text-gray-500">Profil mineur :</span>
                          <button
                            className={`text-xs px-2 py-0.5 rounded-full border transition-colors ${
                              m.is_minor
                                ? "bg-blue-50 border-blue-300 text-blue-700 hover:bg-blue-100"
                                : "bg-gray-50 border-gray-300 text-gray-600 hover:bg-gray-100"
                            }`}
                            disabled={settingMinor === m.cid_short}
                            onClick={(e) => { e.stopPropagation(); handleToggleMinor(m.cid_short, !m.is_minor); }}
                          >
                            {settingMinor === m.cid_short ? "…" : m.is_minor ? "Retirer le statut mineur" : "Marquer comme mineur"}
                          </button>
                        </div>
                        {m.is_minor && (
                          <div className="space-y-1">
                            <p className="text-xs text-gray-500 font-medium">Tuteurs :</p>
                            {(guardians[m.cid_short] ?? []).length === 0 && (
                              <p className="text-xs text-gray-400">Aucun tuteur.</p>
                            )}
                            {(guardians[m.cid_short] ?? []).map((l) => (
                              <div key={l.guardian_cid_short} className="flex items-center gap-2 text-xs text-gray-700">
                                <span className="font-mono">{l.guardian_cid_short}</span>
                                <button
                                  className="text-red-500 hover:text-red-700 text-xs"
                                  onClick={(e) => { e.stopPropagation(); handleRemoveGuardian(m.cid_short, l.guardian_cid_short); }}
                                >
                                  Retirer
                                </button>
                              </div>
                            ))}
                            <div className="flex gap-2 mt-1" onClick={(e) => e.stopPropagation()}>
                              <input
                                type="text"
                                className="flex-1 text-xs border border-gray-200 rounded px-2 py-1"
                                placeholder="CID short du tuteur"
                                value={newGuardianCid}
                                onChange={(e) => setNewGuardianCid(e.target.value)}
                              />
                              <button
                                className="text-xs bg-civium-600 text-white px-2 py-1 rounded hover:bg-civium-700 disabled:opacity-50"
                                disabled={savingGuardian || !newGuardianCid.trim()}
                                onClick={() => handleAddGuardian(m.cid_short)}
                              >
                                {savingGuardian ? "…" : "Ajouter"}
                              </button>
                            </div>
                          </div>
                        )}
                      </div>
                    )}
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

            {/* Garde-fou majoritaire */}
            {adminActions.length > 0 && (
              <section>
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide mb-3">
                  Actions admin — Garde-fou
                </h3>
                <div className="space-y-2">
                  {adminActions.map((a) => {
                    const windowEnd = a.taken_at + a.contest_window_secs;
                    const remaining = windowEnd - now;
                    const isContestable = a.status === "active" && remaining > 0;
                    return (
                      <div
                        key={a.id}
                        className={`bg-white border rounded-xl px-4 py-3 flex items-center gap-3 ${
                          a.status === "suspended"
                            ? "border-orange-300 bg-orange-50"
                            : "border-gray-200"
                        }`}
                      >
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium text-gray-900 truncate">{a.kind}</p>
                          <p className="text-xs text-gray-400">
                            {a.contest_count} conteste(s)
                            {isContestable && remaining > 0 && (
                              <span className="ml-1 text-amber-600">
                                · {Math.ceil(remaining / 3600)}h restantes
                              </span>
                            )}
                            {a.status === "suspended" && (
                              <span className="ml-1 text-orange-600 font-medium">· SUSPENDU → vote en cours</span>
                            )}
                          </p>
                        </div>
                        {isContestable && (
                          <button
                            onClick={() => handleContest(a.id)}
                            disabled={contesting === a.id}
                            className="flex-shrink-0 text-xs px-3 py-1.5 bg-orange-100 border
                                       border-orange-300 text-orange-700 rounded-lg
                                       hover:bg-orange-200 disabled:opacity-50 transition-colors"
                          >
                            {contesting === a.id ? "…" : "Contester"}
                          </button>
                        )}
                        {!isContestable && (
                          <span className={`flex-shrink-0 text-xs px-2 py-0.5 rounded-full ${
                            a.status === "confirmed" ? "bg-green-100 text-green-700" :
                            a.status === "suspended" ? "bg-orange-100 text-orange-700" :
                            "bg-gray-100 text-gray-500"
                          }`}>
                            {a.status}
                          </span>
                        )}
                      </div>
                    );
                  })}
                </div>
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

              {/* Network-wide delegation */}
              {(() => {
                const globalDel = myDelegations.find((d) => d.proposal_id === null);
                return globalDel ? (
                  <div className="flex items-center gap-2 text-xs mb-2 text-blue-600">
                    <span>Délégation réseau active → <span className="font-mono">{globalDel.delegate_cid_short}</span></span>
                    <button
                      onClick={() => handleRevokeDelegation(null)}
                      className="text-gray-400 hover:text-red-500 transition-colors"
                    >
                      Révoquer
                    </button>
                  </div>
                ) : (
                  <div className="flex items-center gap-2 mb-2">
                    <input
                      type="text"
                      value={delegatingTo["global"] ?? ""}
                      onChange={(e) => setDelegatingTo((p) => ({ ...p, global: e.target.value }))}
                      placeholder="Délégation réseau (CID court)…"
                      className="border border-gray-200 rounded px-2 py-1 text-xs
                                 focus:outline-none focus:ring-1 focus:ring-blue-300 w-52"
                    />
                    <button
                      onClick={() => handleDelegate(null, delegatingTo["global"] ?? "")}
                      disabled={savingDelegation === "global" || !delegatingTo["global"]?.trim()}
                      className="text-xs px-2 py-1 bg-blue-50 border border-blue-200 text-blue-700
                                 rounded hover:bg-blue-100 disabled:opacity-50 transition-colors"
                    >
                      {savingDelegation === "global" ? "…" : "Déléguer tout"}
                    </button>
                  </div>
                );
              })()}

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

                      {/* Delegation for this proposal */}
                      {prop.status === "open" && (() => {
                        const propDel = myDelegations.find((d) => d.proposal_id === prop.id);
                        const globalDel = myDelegations.find((d) => d.proposal_id === null);
                        const activeDel = propDel ?? globalDel;
                        const key = prop.id;
                        return (
                          <div className="flex items-center gap-2 text-xs">
                            {activeDel ? (
                              <>
                                <span className="text-blue-600">
                                  Vote délégué → <span className="font-mono">{activeDel.delegate_cid_short}</span>
                                  {activeDel.proposal_id === null && " (réseau)"}
                                </span>
                                <button
                                  onClick={() => handleRevokeDelegation(propDel ? prop.id : null)}
                                  className="text-gray-400 hover:text-red-500 transition-colors"
                                >
                                  Révoquer
                                </button>
                              </>
                            ) : (
                              <>
                                <input
                                  type="text"
                                  value={delegatingTo[key] ?? ""}
                                  onChange={(e) =>
                                    setDelegatingTo((p) => ({ ...p, [key]: e.target.value }))
                                  }
                                  placeholder="Déléguer à (CID court)…"
                                  className="border border-gray-200 rounded px-2 py-1 text-xs
                                             focus:outline-none focus:ring-1 focus:ring-blue-300 w-44"
                                />
                                <button
                                  onClick={() => handleDelegate(prop.id, delegatingTo[key] ?? "")}
                                  disabled={savingDelegation === key || !delegatingTo[key]?.trim()}
                                  className="px-2 py-1 bg-blue-50 border border-blue-200 text-blue-700
                                             rounded hover:bg-blue-100 disabled:opacity-50 transition-colors"
                                >
                                  {savingDelegation === key ? "…" : "Déléguer"}
                                </button>
                              </>
                            )}
                          </div>
                        );
                      })()}

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

            {/* ── Annuaire section (directory networks only) ── */}
            {/* ── RRM section (RRM networks only) ── */}
            {selected.is_rrm && (
              <section>
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                    Réseaux signalés ({rrmEntries.length})
                  </h3>
                  <button
                    onClick={() => setShowReportForm((v) => !v)}
                    className="text-xs px-3 py-1.5 bg-red-600 text-white rounded-lg
                               hover:bg-red-700 transition-colors"
                  >
                    {showReportForm ? "Annuler" : "+ Signaler"}
                  </button>
                </div>

                {showReportForm && (
                  <div className="bg-red-50 border border-red-200 rounded-xl p-4 mb-4 space-y-3">
                    <input
                      value={reportNetCid}
                      onChange={(e) => setReportNetCid(e.target.value)}
                      placeholder="CID court du réseau à signaler"
                      className="w-full text-sm border border-red-200 rounded-lg px-3 py-1.5
                                 focus:outline-none focus:ring-2 focus:ring-red-400
                                 font-mono placeholder:font-sans placeholder:text-gray-400 bg-white"
                    />
                    <input
                      value={reportNetName}
                      onChange={(e) => setReportNetName(e.target.value)}
                      placeholder="Nom du réseau"
                      className="w-full text-sm border border-red-200 rounded-lg px-3 py-1.5
                                 focus:outline-none focus:ring-2 focus:ring-red-400
                                 placeholder:text-gray-400 bg-white"
                    />
                    <input
                      value={reportReason}
                      onChange={(e) => setReportReason(e.target.value)}
                      placeholder="Motif du signalement"
                      className="w-full text-sm border border-red-200 rounded-lg px-3 py-1.5
                                 focus:outline-none focus:ring-2 focus:ring-red-400
                                 placeholder:text-gray-400 bg-white"
                    />
                    <input
                      value={reportEvidence}
                      onChange={(e) => setReportEvidence(e.target.value)}
                      placeholder="URL preuve (optionnel)"
                      className="w-full text-sm border border-red-200 rounded-lg px-3 py-1.5
                                 focus:outline-none focus:ring-2 focus:ring-red-400
                                 font-mono placeholder:font-sans placeholder:text-gray-400 bg-white"
                    />
                    <button
                      onClick={handleReport}
                      disabled={reporting || !reportNetCid.trim() || !reportNetName.trim() || !reportReason.trim()}
                      className="text-xs px-4 py-2 bg-red-600 text-white rounded-lg
                                 hover:bg-red-700 disabled:opacity-50 transition-colors font-medium"
                    >
                      {reporting ? "Signalement…" : "Signaler"}
                    </button>
                  </div>
                )}

                {rrmEntries.length === 0 ? (
                  <p className="text-sm text-gray-400 text-center py-6">
                    Aucun réseau signalé dans ce registre.
                  </p>
                ) : (
                  <div className="bg-white border border-red-100 rounded-xl divide-y divide-red-50">
                    {rrmEntries.map((entry) => (
                      <div key={entry.id} className="px-4 py-3 flex items-start gap-3">
                        <span className="mt-0.5 text-xs font-medium px-2 py-0.5 rounded-full flex-shrink-0
                                         bg-red-100 text-red-700">
                          ⚠
                        </span>
                        <div className="flex-1 min-w-0">
                          <div className="text-sm font-medium text-gray-900">{entry.network_name}</div>
                          <div className="text-xs text-gray-400 font-mono">{entry.network_cid_short}</div>
                          <p className="text-xs text-red-700 mt-0.5">{entry.reason}</p>
                          {entry.evidence_url && (
                            <p className="text-xs text-blue-500 mt-0.5 truncate">
                              {entry.evidence_url}
                            </p>
                          )}
                          <p className="text-xs text-gray-400 mt-0.5">
                            par {entry.reported_by} · {new Date(entry.reported_at * 1000).toLocaleDateString("fr-FR")}
                          </p>
                        </div>
                        <button
                          onClick={() => handleRemoveRrmEntry(entry.id)}
                          className="text-xs text-gray-300 hover:text-red-400 transition-colors flex-shrink-0"
                          title="Supprimer"
                        >
                          ✕
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </section>
            )}

            {/* ── Trusted RRMs section (standard networks only) ── */}
            {!selected.is_directory && !selected.is_rrm && (trustedRrms.length > 0 || showTrustForm) && (
              <section>
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                    RRM de confiance ({trustedRrms.length})
                  </h3>
                  <button
                    onClick={() => setShowTrustForm((v) => !v)}
                    className="text-xs px-3 py-1.5 bg-white border border-gray-200 text-gray-600
                               rounded-lg hover:bg-gray-50 transition-colors"
                  >
                    {showTrustForm ? "Annuler" : "+ Ajouter"}
                  </button>
                </div>

                {showTrustForm && (
                  <div className="bg-orange-50 border border-orange-200 rounded-xl p-4 mb-3 space-y-2">
                    <input
                      value={trustRrmCid}
                      onChange={(e) => setTrustRrmCid(e.target.value)}
                      placeholder="CID court du RRM"
                      className="w-full text-sm border border-orange-200 rounded-lg px-3 py-1.5
                                 focus:outline-none focus:ring-2 focus:ring-orange-400
                                 font-mono placeholder:font-sans placeholder:text-gray-400 bg-white"
                    />
                    <input
                      value={trustRrmName}
                      onChange={(e) => setTrustRrmName(e.target.value)}
                      placeholder="Nom du RRM"
                      className="w-full text-sm border border-orange-200 rounded-lg px-3 py-1.5
                                 focus:outline-none focus:ring-2 focus:ring-orange-400
                                 placeholder:text-gray-400 bg-white"
                    />
                    <button
                      onClick={handleTrustRrm}
                      disabled={savingTrust || !trustRrmCid.trim() || !trustRrmName.trim()}
                      className="text-xs px-3 py-1.5 bg-orange-600 text-white rounded-lg
                                 hover:bg-orange-700 disabled:opacity-50 transition-colors"
                    >
                      {savingTrust ? "…" : "Faire confiance"}
                    </button>
                  </div>
                )}

                <div className="bg-orange-50 border border-orange-100 rounded-xl divide-y divide-orange-50">
                  {trustedRrms.map((t) => (
                    <div key={t.rrm_cid_short} className="flex items-center justify-between px-4 py-2.5 text-xs">
                      <span className="text-orange-800">
                        {t.rrm_name}
                        <span className="text-orange-400 font-mono ml-1">({t.rrm_cid_short})</span>
                      </span>
                      <button
                        onClick={() => handleUntrustRrm(t.rrm_cid_short)}
                        className="text-orange-300 hover:text-red-400 transition-colors"
                        title="Retirer la confiance"
                      >
                        ✕
                      </button>
                    </div>
                  ))}
                </div>
              </section>
            )}

            {/* ── Add trusted RRM button when none yet ── */}
            {!selected.is_directory && !selected.is_rrm && trustedRrms.length === 0 && !showTrustForm && (
              <button
                onClick={() => setShowTrustForm(true)}
                className="text-xs text-gray-400 hover:text-orange-600 transition-colors block"
              >
                + Faire confiance à un RRM
              </button>
            )}

            {/* ── Annuaire section (directory networks only) ── */}
            {selected.is_directory && (
              <section>
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                    Annuaire
                  </h3>
                  <button
                    onClick={() => setShowPublishForm((v) => !v)}
                    className="text-xs px-3 py-1.5 bg-civium-600 text-white rounded-lg
                               hover:bg-civium-700 transition-colors"
                  >
                    {showPublishForm ? "Annuler" : "+ Publier"}
                  </button>
                </div>

                {showPublishForm && (
                  <div className="bg-white border border-gray-200 rounded-xl p-4 mb-4 space-y-3">
                    <div className="flex gap-3">
                      <select
                        value={pubKind}
                        onChange={(e) => setPubKind(e.target.value as "network" | "member")}
                        className="text-sm border border-gray-200 rounded-lg px-2 py-1.5
                                   focus:outline-none focus:ring-2 focus:ring-civium-400"
                      >
                        <option value="network">Réseau</option>
                        <option value="member">Membre</option>
                      </select>
                      <input
                        value={pubSubjectCid}
                        onChange={(e) => setPubSubjectCid(e.target.value)}
                        placeholder="CID court du sujet"
                        className="flex-1 text-sm border border-gray-200 rounded-lg px-3 py-1.5
                                   focus:outline-none focus:ring-2 focus:ring-civium-400
                                   font-mono placeholder:font-sans placeholder:text-gray-400"
                      />
                    </div>
                    <input
                      value={pubSubjectName}
                      onChange={(e) => setPubSubjectName(e.target.value)}
                      placeholder="Nom affiché"
                      className="w-full text-sm border border-gray-200 rounded-lg px-3 py-1.5
                                 focus:outline-none focus:ring-2 focus:ring-civium-400
                                 placeholder:text-gray-400"
                    />
                    <input
                      value={pubDescription}
                      onChange={(e) => setPubDescription(e.target.value)}
                      placeholder="Description (optionnel)"
                      className="w-full text-sm border border-gray-200 rounded-lg px-3 py-1.5
                                 focus:outline-none focus:ring-2 focus:ring-civium-400
                                 placeholder:text-gray-400"
                    />
                    <input
                      value={pubTags}
                      onChange={(e) => setPubTags(e.target.value)}
                      placeholder="Tags, séparés par des virgules (optionnel)"
                      className="w-full text-sm border border-gray-200 rounded-lg px-3 py-1.5
                                 focus:outline-none focus:ring-2 focus:ring-civium-400
                                 placeholder:text-gray-400"
                    />
                    <button
                      onClick={handlePublish}
                      disabled={publishing || !pubSubjectCid.trim() || !pubSubjectName.trim()}
                      className="text-xs px-4 py-2 bg-civium-600 text-white rounded-lg
                                 hover:bg-civium-700 disabled:opacity-50 transition-colors font-medium"
                    >
                      {publishing ? "Publication…" : "Publier"}
                    </button>
                  </div>
                )}

                {/* Federations */}
                {(federations.length > 0 || showFedForm) && (
                  <div className="mb-4 bg-blue-50 border border-blue-100 rounded-xl p-3 space-y-2">
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-semibold text-blue-700 uppercase tracking-wide">
                        Fédérations ({federations.length})
                      </span>
                      <button
                        onClick={() => setShowFedForm((v) => !v)}
                        className="text-xs text-blue-600 hover:underline"
                      >
                        {showFedForm ? "Annuler" : "+ Ajouter"}
                      </button>
                    </div>
                    {showFedForm && (
                      <div className="space-y-2 pt-1">
                        <input
                          value={fedPeerCid}
                          onChange={(e) => setFedPeerCid(e.target.value)}
                          placeholder="CID court de l'annuaire pair"
                          className="w-full text-sm border border-blue-200 rounded-lg px-3 py-1.5
                                     focus:outline-none focus:ring-2 focus:ring-blue-400
                                     font-mono placeholder:font-sans placeholder:text-gray-400 bg-white"
                        />
                        <input
                          value={fedPeerName}
                          onChange={(e) => setFedPeerName(e.target.value)}
                          placeholder="Nom de l'annuaire pair"
                          className="w-full text-sm border border-blue-200 rounded-lg px-3 py-1.5
                                     focus:outline-none focus:ring-2 focus:ring-blue-400
                                     placeholder:text-gray-400 bg-white"
                        />
                        <input
                          value={fedPeerAddr}
                          onChange={(e) => setFedPeerAddr(e.target.value)}
                          placeholder="Adresse P2P (optionnel)"
                          className="w-full text-sm border border-blue-200 rounded-lg px-3 py-1.5
                                     focus:outline-none focus:ring-2 focus:ring-blue-400
                                     font-mono placeholder:font-sans placeholder:text-gray-400 bg-white"
                        />
                        <button
                          onClick={handleFederate}
                          disabled={savingFed || !fedPeerCid.trim() || !fedPeerName.trim()}
                          className="text-xs px-3 py-1.5 bg-blue-600 text-white rounded-lg
                                     hover:bg-blue-700 disabled:opacity-50 transition-colors"
                        >
                          {savingFed ? "…" : "Fédérer"}
                        </button>
                      </div>
                    )}
                    {federations.map((f) => (
                      <div key={f.peer_cid_short} className="flex items-center justify-between text-xs">
                        <span className="text-blue-800">
                          {f.peer_name}
                          <span className="text-blue-400 font-mono ml-1">({f.peer_cid_short})</span>
                        </span>
                        <button
                          onClick={() => handleUnfederate(f.peer_cid_short)}
                          className="text-blue-300 hover:text-red-400 transition-colors"
                          title="Supprimer cette fédération"
                        >
                          ✕
                        </button>
                      </div>
                    ))}
                  </div>
                )}
                {federations.length === 0 && !showFedForm && (
                  <button
                    onClick={() => setShowFedForm(true)}
                    className="text-xs text-gray-400 hover:text-civium-600 mb-3 block"
                  >
                    + Ajouter une fédération
                  </button>
                )}

                {/* Search */}
                <div className="flex gap-2 mb-2">
                  <input
                    value={dirSearchQuery}
                    onChange={(e) => setDirSearchQuery(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleDirSearch()}
                    placeholder="Rechercher dans l'annuaire…"
                    className="flex-1 text-sm border border-gray-200 rounded-lg px-3 py-1.5
                               focus:outline-none focus:ring-2 focus:ring-civium-400
                               placeholder:text-gray-400"
                  />
                  <button
                    onClick={handleDirSearch}
                    disabled={!dirSearchQuery.trim()}
                    className="text-xs px-3 py-1.5 bg-white border border-gray-200 text-gray-600
                               rounded-lg hover:bg-gray-50 disabled:opacity-50 transition-colors"
                  >
                    Rechercher
                  </button>
                  {dirSearchResults !== null && (
                    <button
                      onClick={() => { setDirSearchResults(null); setDirSearchQuery(""); }}
                      className="text-xs px-2 py-1.5 text-gray-400 hover:text-gray-600"
                    >
                      ✕
                    </button>
                  )}
                </div>
                {federations.length > 0 && (
                  <label className="flex items-center gap-2 text-xs text-gray-500 mb-4 cursor-pointer select-none">
                    <input
                      type="checkbox"
                      checked={includeFederated}
                      onChange={(e) => setIncludeFederated(e.target.checked)}
                      className="accent-civium-600"
                    />
                    Inclure les annuaires fédérés dans la recherche
                  </label>
                )}

                {/* Entries list */}
                {(() => {
                  const items = dirSearchResults ?? dirEntries;
                  if (items.length === 0) {
                    return (
                      <p className="text-sm text-gray-400 text-center py-6">
                        {dirSearchResults !== null
                          ? `Aucun résultat pour « ${dirSearchQuery} ».`
                          : "Aucune entrée. Publiez la première avec le bouton ci-dessus."}
                      </p>
                    );
                  }
                  return (
                    <div className="bg-white border border-gray-200 rounded-xl divide-y divide-gray-100">
                      {items.map((entry) => (
                        <div key={entry.id} className="px-4 py-3 flex items-start gap-3">
                          <span className={`mt-0.5 text-xs font-medium px-2 py-0.5 rounded-full flex-shrink-0 ${
                            entry.kind === "network"
                              ? "bg-blue-100 text-blue-700"
                              : "bg-purple-100 text-purple-700"
                          }`}>
                            {entry.kind === "network" ? "Réseau" : "Membre"}
                          </span>
                          <div className="flex-1 min-w-0">
                            <div className="text-sm font-medium">{entry.subject_name}</div>
                            <div className="flex items-center gap-2">
                              <span className="text-xs text-gray-400 font-mono">{entry.subject_cid_short}</span>
                              {entry.source_dir_name && (
                                <span className="text-xs bg-blue-50 text-blue-500 px-1.5 py-0.5 rounded">
                                  via {entry.source_dir_name}
                                </span>
                              )}
                            </div>
                            {entry.description && (
                              <p className="text-xs text-gray-500 mt-0.5">{entry.description}</p>
                            )}
                            {entry.tags.length > 0 && (
                              <div className="flex flex-wrap gap-1 mt-1">
                                {entry.tags.map((tag) => (
                                  <span key={tag} className="text-xs bg-gray-100 text-gray-500 px-1.5 py-0.5 rounded">
                                    {tag}
                                  </span>
                                ))}
                              </div>
                            )}
                          </div>
                          <button
                            onClick={() => handleRemoveEntry(entry.id)}
                            className="text-xs text-gray-300 hover:text-red-400 transition-colors flex-shrink-0"
                            title="Supprimer"
                          >
                            ✕
                          </button>
                        </div>
                      ))}
                    </div>
                  );
                })()}
              </section>
            )}
          </div>
        )}
      </main>
    </div>
  );
}
