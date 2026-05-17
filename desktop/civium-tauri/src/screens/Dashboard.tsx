import { useState, useEffect, useCallback, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { tauriInvoke } from "../tauri";
import type {
  NetworkInfo,
  MemberInfo,
  PendingMemberInfo,
  NodeStatus,
  MessageDisplay,
  PluginInfo,
  ProposalInfo,
  VoteResultInfo,
  AdminActionInfo,
  AgendaEventInfo,
  ActivityEventInfo,
  DocumentInfo,
  McpStatus,
  PairingInitInfo,
  PairedDeviceInfo,
  NotificationInfo,
  DelegationInfo,
  DirectoryEntryInfo,
  FederationInfo,
  GuardianLinkInfo,
  RrmEntryInfo,
  TrustedRrmInfo,
  OutboxCountInfo,
  RccStatusInfo,
  ApFollowerInfo,
  ApPostInfo,
  ApPostResult,
  FraudAlertInfo,
  NodeSettingsInfo,
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
  const [settingRole, setSettingRole] = useState<string | null>(null);
  const [removingMember, setRemovingMember] = useState<string | null>(null);

  // Direct messages state
  const [dmBody, setDmBody] = useState<Record<string, string>>({});
  const [sendingDm, setSendingDm] = useState<string | null>(null);
  const [dmE2EMode, setDmE2EMode] = useState<Record<string, boolean>>({});

  // Activity & Notifications
  const [activityEvents, setActivityEvents] = useState<ActivityEventInfo[]>([]);
  const [notifications, setNotifications] = useState<NotificationInfo[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);

  // Agenda state
  const [agendaEvents, setAgendaEvents] = useState<AgendaEventInfo[]>([]);
  const [showAgendaForm, setShowAgendaForm] = useState(false);
  const [agendaTitle, setAgendaTitle] = useState("");
  const [agendaDescription, setAgendaDescription] = useState("");
  const [agendaStart, setAgendaStart] = useState("");
  const [agendaEnd, setAgendaEnd] = useState("");
  const [agendaLocation, setAgendaLocation] = useState("");

  // Documents state
  const [documents, setDocuments] = useState<DocumentInfo[]>([]);
  const [showDocForm, setShowDocForm] = useState(false);
  const [docTitle, setDocTitle] = useState("");
  const [docBody, setDocBody] = useState("");
  const [creatingDoc, setCreatingDoc] = useState(false);
  const [expandedDocId, setExpandedDocId] = useState<string | null>(null);

  // MCP state
  const [mcpStatus, setMcpStatus] = useState<McpStatus>({ running: false, port: null, token: null, url: null });
  const [mcpPort, setMcpPort] = useState("7523");
  const [showMcpToken, setShowMcpToken] = useState(false);

  // Outbox state
  const [outboxCounts, setOutboxCounts] = useState<Record<string, number>>({});

  // RCC state
  const [rccStatuses, setRccStatuses] = useState<Record<string, RccStatusInfo>>({});
  const [rccEmail, setRccEmail] = useState("");
  const [rccRegistering, setRccRegistering] = useState(false);

  // ActivityPub state
  const [apFollowers, setApFollowers] = useState<ApFollowerInfo[]>([]);
  const [apPosts, setApPosts] = useState<ApPostInfo[]>([]);
  const [apContent, setApContent] = useState("");
  const [apPosting, setApPosting] = useState(false);
  const [apEnabling, setApEnabling] = useState(false);
  const [apError, setApError] = useState<string | null>(null);

  // Hub sync state
  const [hubConfig, setHubConfig] = useState<{ hub_url: string; enabled: boolean; last_sync_ts: number } | null>(null);
  const [hubUrl, setHubUrl] = useState("");
  const [hubSyncing, setHubSyncing] = useState(false);
  const [hubSaving, setHubSaving] = useState(false);
  const [hubMsg, setHubMsg] = useState<{ ok: boolean; text: string } | null>(null);

  // Node settings state
  const [nodeSettings, setNodeSettings] = useState<NodeSettingsInfo | null>(null);
  const [nodeTcpPort, setNodeTcpPort] = useState("");
  const [nodeWsPort, setNodeWsPort] = useState("");
  const [nodeExternalAddr, setNodeExternalAddr] = useState("");
  const [nodeSaving, setNodeSaving] = useState(false);

  // Pairing state
  const [pairedDevices, setPairedDevices] = useState<PairedDeviceInfo[]>([]);
  const [pairingSession, setPairingSession] = useState<PairingInitInfo | null>(null);
  const [pairLabel, setPairLabel] = useState("");
  const [pairLink, setPairLink] = useState("");
  const [pairCompleteLabel, setPairCompleteLabel] = useState("");
  const [showPairCompleteForm, setShowPairCompleteForm] = useState(false);
  const [creatingEvent, setCreatingEvent] = useState(false);

  const [plugins, setPlugins] = useState<PluginInfo[]>([]);
  const [togglingPlugin, setTogglingPlugin] = useState<string | null>(null);

  // Settings panel
  const [showSettings, setShowSettings] = useState(false);
  const [identity, setIdentity] = useState<{ cid_short: string; cid_full: string; secret_b58: string } | null>(null);
  const [secretVisible, setSecretVisible] = useState(false);

  // Active view within selected network
  type ActiveView = 'messages' | 'membres' | 'gouvernance' | 'agenda' | 'documents' | 'activite' | 'notifications' | 'annuaire' | 'rrm' | 'extensions';
  const [activeView, setActiveView] = useState<ActiveView>('messages');

  // Create network form
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [createName, setCreateName] = useState("");
  const [createAdminEmail, setCreateAdminEmail] = useState("");
  const [createIsPublic, setCreateIsPublic] = useState(false);
  const [creating, setCreating] = useState(false);

  // Email invite
  const [inviteEmail, setInviteEmail] = useState("");

  // Join network form
  const [showJoinForm, setShowJoinForm] = useState(false);
  const [joinInviteLink, setJoinInviteLink] = useState("");
  const [joinPeerAddr, setJoinPeerAddr] = useState("");
  const [joinDisplayName, setJoinDisplayName] = useState("");
  const [joining, setJoining] = useState(false);

  // Fraud alerts
  const [activeAlerts, setActiveAlerts] = useState<FraudAlertInfo[]>([]);
  const [rootConnected, setRootConnected] = useState<string | null>(null);

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

  const refreshOutboxCounts = useCallback(() => {
    tauriInvoke<OutboxCountInfo[]>("outbox_count_all")
      .then((items) => {
        const map: Record<string, number> = {};
        for (const item of items) map[item.network_cid_short] = item.count;
        setOutboxCounts(map);
      })
      .catch(() => {});
  }, []);

  const refreshRccStatuses = useCallback(() => {
    tauriInvoke<RccStatusInfo[]>("rcc_status_list")
      .then((items) => {
        const map: Record<string, RccStatusInfo> = {};
        for (const item of items) map[item.network_cid_short] = item;
        setRccStatuses(map);
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    tauriInvoke<NetworkInfo[]>("network_list").then(setNetworks);
    tauriInvoke<PluginInfo[]>("plugin_list").then(setPlugins).catch(() => {});
    tauriInvoke<PairedDeviceInfo[]>("pair_list").then(setPairedDevices).catch(() => {});
    tauriInvoke<{ cid_short: string; cid_full: string; secret_b58: string }>("identity_show")
      .then(setIdentity).catch(() => {});
    refreshOutboxCounts();
    refreshRccStatuses();
  }, [refreshOutboxCounts, refreshRccStatuses]);

  const refreshNetwork = useCallback((cid: string) => {
    tauriInvoke<MemberInfo[]>("member_list", { networkCid: cid }).then((data) => {
      if (selectedRef.current?.cid_short === cid) setMembers(data);
    });
    tauriInvoke<PendingMemberInfo[]>("member_pending_list", { networkCid: cid }).then((data) => {
      if (selectedRef.current?.cid_short === cid) setPending(data);
    });
    tauriInvoke<NetworkInfo[]>("network_list").then((nets) => {
      setNetworks(nets);
      const updated = nets.find((n) => n.cid_short === cid);
      if (updated && selectedRef.current?.cid_short === cid) setSelected(updated);
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

  const refreshAgendaEvents = useCallback((cid: string) => {
    tauriInvoke<AgendaEventInfo[]>("agenda_list", { networkCidShort: cid }).then(setAgendaEvents).catch(() => {});
  }, []);

  const refreshDocuments = useCallback((cid: string) => {
    tauriInvoke<DocumentInfo[]>("document_list", { networkCidShort: cid }).then(setDocuments).catch(() => {});
  }, []);

  const refreshPairedDevices = useCallback(() => {
    tauriInvoke<PairedDeviceInfo[]>("pair_list").then(setPairedDevices).catch(() => {});
  }, []);

  const refreshAp = useCallback((cid: string) => {
    tauriInvoke<ApFollowerInfo[]>("ap_list_followers", { networkCid: cid }).then(setApFollowers).catch(() => {});
    tauriInvoke<ApPostInfo[]>("ap_list_posts", { networkCid: cid }).then(setApPosts).catch(() => {});
  }, []);

  const handleApEnable = useCallback(async () => {
    if (!selected) return;
    setApEnabling(true);
    setApError(null);
    try {
      await tauriInvoke("ap_enable", { networkCid: selected.cid_short });
      refreshNetwork(selected.cid_short);
      refreshAp(selected.cid_short);
    } catch (e) {
      setApError(String(e));
    } finally {
      setApEnabling(false);
    }
  }, [selected, refreshNetwork, refreshAp]);

  const handleApDisable = useCallback(async () => {
    if (!selected) return;
    setApEnabling(true);
    setApError(null);
    try {
      await tauriInvoke("ap_disable", { networkCid: selected.cid_short });
      refreshNetwork(selected.cid_short);
      refreshAp(selected.cid_short);
    } catch (e) {
      setApError(String(e));
    } finally {
      setApEnabling(false);
    }
  }, [selected, refreshNetwork, refreshAp]);

  const handleApPost = useCallback(async () => {
    if (!selected || !apContent.trim()) return;
    setApPosting(true);
    setApError(null);
    try {
      await tauriInvoke<ApPostResult>("ap_post", { networkCid: selected.cid_short, content: apContent.trim() });
      setApContent("");
      refreshAp(selected.cid_short);
    } catch (e) {
      setApError(String(e));
    } finally {
      setApPosting(false);
    }
  }, [selected, apContent, refreshAp]);

  const refreshActivity = useCallback((cid: string) => {
    tauriInvoke<ActivityEventInfo[]>("activity_list", { networkCidShort: cid }).then(setActivityEvents).catch(() => {});
    tauriInvoke<NotificationInfo[]>("notification_list", { networkCidShort: cid }).then((notifs) => {
      setNotifications(notifs);
      setUnreadCount(notifs.filter((n) => !n.read).length);
    }).catch(() => {});
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
    refreshAgendaEvents(selected.cid_short);
    refreshDocuments(selected.cid_short);
    refreshActivity(selected.cid_short);
    refreshAp(selected.cid_short);
    // Charger la config hub pour ce réseau
    tauriInvoke<{ hub_url: string; enabled: boolean; last_sync_ts: number } | null>("hub_config_get", {
      networkCid: selected.cid_short,
    }).then((cfg) => {
      setHubConfig(cfg);
      setHubUrl(cfg?.hub_url ?? "");
      setHubMsg(null);
    }).catch(() => {});
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
    setAgendaEvents([]);
    setShowAgendaForm(false);
    setDocuments([]);
    setShowDocForm(false);
    setExpandedDocId(null);
    setActivityEvents([]);
    setNotifications([]);
    setUnreadCount(0);
  }, [selected?.cid_short]);

  // Charger les paramètres du nœud une seule fois.
  useEffect(() => {
    tauriInvoke<NodeSettingsInfo>("node_settings_get")
      .then((s) => {
        setNodeSettings(s);
        setNodeTcpPort(s.tcp_port === 0 ? "" : String(s.tcp_port));
        setNodeWsPort(s.ws_port === 0 ? "" : String(s.ws_port));
        setNodeExternalAddr(s.external_addr);
      })
      .catch(() => {});
  }, []);

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

    // Load MCP status once on mount
    tauriInvoke<McpStatus>("mcp_status").then(setMcpStatus).catch(() => {});

    tauriInvoke<FraudAlertInfo[]>("get_active_alerts").then((a) => {
      if (mounted) setActiveAlerts(a);
    }).catch(() => {});

    let unlistenSync: UnlistenFn | null = null;
    let unlistenOutbox: UnlistenFn | null = null;
    let unlistenRcc: UnlistenFn | null = null;
    let unlistenAlert: UnlistenFn | null = null;

    listen<string>("civium://sync-completed", (event) => {
      const cid = event.payload;
      tauriInvoke<NetworkInfo[]>("network_list").then((nets) => {
        if (mounted) setNetworks(nets);
      });
      if (selectedRef.current?.cid_short === cid) {
        refreshNetwork(cid);
        refreshMessages(cid);
      }
      refreshOutboxCounts();
    }).then((fn) => {
      unlistenSync = fn;
    });

    listen<string>("civium://outbox-cleared", () => {
      refreshOutboxCounts();
    }).then((fn) => {
      unlistenOutbox = fn;
    });

    listen("civium://rcc-status-changed", () => {
      refreshRccStatuses();
    }).then((fn) => {
      unlistenRcc = fn;
    });

    listen<FraudAlertInfo>("civium://fraud-alert", (event) => {
      if (mounted) setActiveAlerts((prev) => [...prev, event.payload]);
    }).then((fn) => {
      unlistenAlert = fn;
    });

    let unlistenRoot: UnlistenFn | null = null;
    listen<string>("civium://root-connected", (event) => {
      if (mounted) setRootConnected(event.payload);
    }).then((fn) => {
      unlistenRoot = fn;
    });

    return () => {
      mounted = false;
      clearInterval(interval);
      unlistenSync?.();
      unlistenOutbox?.();
      unlistenRcc?.();
      unlistenAlert?.();
      unlistenRoot?.();
    };
  }, [refreshNetwork, refreshMessages, refreshOutboxCounts, refreshRccStatuses]);

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
      refreshActivity(selected.cid_short);
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
      refreshActivity(selected.cid_short);
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
      refreshActivity(selected.cid_short);
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

  async function handleSetRole(memberCid: string, role: "admin" | "member") {
    if (!selected) return;
    setSettingRole(memberCid);
    try {
      await tauriInvoke("member_set_role", { networkCid: selected.cid_short, memberCid, role });
      refreshNetwork(selected.cid_short);
    } catch (e) {
      alert(String(e));
    } finally {
      setSettingRole(null);
    }
  }

  async function handleRemoveMember(memberCid: string, displayName: string) {
    if (!selected) return;
    if (!window.confirm(`Exclure ${displayName} du réseau ? Cette action est irréversible.`)) return;
    setRemovingMember(memberCid);
    try {
      await tauriInvoke("member_remove", { networkCid: selected.cid_short, memberCid });
      setExpandedMember(null);
      refreshNetwork(selected.cid_short);
    } catch (e) {
      alert(String(e));
    } finally {
      setRemovingMember(null);
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

  async function handleSendDm(toCidShort: string) {
    if (!selected || !dmBody[toCidShort]?.trim()) return;
    setSendingDm(toCidShort);
    const isE2E = dmE2EMode[toCidShort] ?? false;
    try {
      const cmd = isE2E ? "message_send_e2e" : "message_send_direct";
      const msg = await tauriInvoke<MessageDisplay>(cmd, {
        networkCid: selected.cid_short,
        toCidShort,
        body: dmBody[toCidShort].trim(),
      });
      setMessages((prev) => [...prev, msg]);
      setDmBody((prev) => ({ ...prev, [toCidShort]: "" }));
      refreshOutboxCounts();
    } catch (e) {
      alert(String(e));
    } finally {
      setSendingDm(null);
    }
  }

  async function handleTogglePlugin(pluginId: string, currentState: string) {
    setTogglingPlugin(pluginId);
    try {
      if (currentState === "enabled") {
        await tauriInvoke("plugin_disable", { pluginId });
      } else {
        await tauriInvoke("plugin_enable", { pluginId });
      }
      const updated = await tauriInvoke<PluginInfo[]>("plugin_list");
      setPlugins(updated);
    } catch (e) {
      alert(String(e));
    } finally {
      setTogglingPlugin(null);
    }
  }

  async function handleMarkAllRead() {
    if (!selected) return;
    const unread = notifications.filter((n) => !n.read);
    await Promise.all(
      unread.map((n) => tauriInvoke("notification_mark_read", { notifId: n.id }).catch(() => {}))
    );
    refreshActivity(selected.cid_short);
  }

  async function handleCreateEvent() {
    if (!selected || !agendaTitle.trim() || !agendaStart) return;
    setCreatingEvent(true);
    try {
      const startTs = Math.floor(new Date(agendaStart).getTime() / 1000);
      const endTs = agendaEnd ? Math.floor(new Date(agendaEnd).getTime() / 1000) : null;
      await tauriInvoke("agenda_create", {
        networkCidShort: selected.cid_short,
        title: agendaTitle.trim(),
        description: agendaDescription.trim(),
        startAt: startTs,
        endAt: endTs,
        location: agendaLocation.trim() || null,
      });
      setAgendaTitle("");
      setAgendaDescription("");
      setAgendaStart("");
      setAgendaEnd("");
      setAgendaLocation("");
      setShowAgendaForm(false);
      refreshAgendaEvents(selected.cid_short);
    } catch (e) {
      alert(String(e));
    } finally {
      setCreatingEvent(false);
    }
  }

  async function handleDeleteEvent(eventId: string) {
    if (!selected) return;
    try {
      await tauriInvoke("agenda_delete", {
        networkCidShort: selected.cid_short,
        eventId,
      });
      refreshAgendaEvents(selected.cid_short);
    } catch (e) {
      alert(String(e));
    }
  }

  async function handleCreateDocument() {
    if (!selected || !docTitle.trim() || !docBody.trim()) return;
    setCreatingDoc(true);
    try {
      await tauriInvoke("document_create", {
        networkCidShort: selected.cid_short,
        title: docTitle.trim(),
        body: docBody.trim(),
      });
      setDocTitle("");
      setDocBody("");
      setShowDocForm(false);
      refreshDocuments(selected.cid_short);
      refreshActivity(selected.cid_short);
    } catch (e) {
      alert(String(e));
    } finally {
      setCreatingDoc(false);
    }
  }

  async function handleDeleteDocument(docId: string) {
    if (!selected) return;
    try {
      await tauriInvoke("document_delete", {
        networkCidShort: selected.cid_short,
        docId,
      });
      setExpandedDocId(null);
      refreshDocuments(selected.cid_short);
    } catch (e) {
      alert(String(e));
    }
  }

  async function handleMcpStart() {
    try {
      const port = parseInt(mcpPort, 10) || 7523;
      const status = await tauriInvoke<McpStatus>("mcp_start", { port });
      setMcpStatus(status);
      setShowMcpToken(true);
    } catch (e) {
      alert(String(e));
    }
  }

  async function handleMcpStop() {
    try {
      await tauriInvoke("mcp_stop");
      setMcpStatus({ running: false, port: null, token: null, url: null });
      setShowMcpToken(false);
    } catch (e) {
      alert(String(e));
    }
  }

  async function handlePairInit() {
    if (!pairLabel.trim()) return;
    try {
      const session = await tauriInvoke<PairingInitInfo>("pair_init", { label: pairLabel.trim() });
      setPairingSession(session);
      setPairLabel("");
    } catch (e) {
      alert(String(e));
    }
  }

  async function handlePairComplete() {
    if (!pairLink.trim() || !pairCompleteLabel.trim()) return;
    try {
      const device = await tauriInvoke<PairedDeviceInfo>("pair_complete", {
        link: pairLink.trim(),
        label: pairCompleteLabel.trim(),
      });
      setPairedDevices((prev) => [...prev, device]);
      setPairLink("");
      setPairCompleteLabel("");
      setShowPairCompleteForm(false);
    } catch (e) {
      alert(String(e));
    }
  }

  async function handleHubSave() {
    if (!selected) return;
    setHubSaving(true);
    setHubMsg(null);
    try {
      await tauriInvoke("hub_config_set", {
        networkCid: selected.cid_short,
        hubUrl: hubUrl.trim(),
        enabled: true,
      });
      await tauriInvoke("hub_network_register", { networkCid: selected.cid_short });
      const cfg = await tauriInvoke<{ hub_url: string; enabled: boolean; last_sync_ts: number } | null>(
        "hub_config_get", { networkCid: selected.cid_short }
      );
      setHubConfig(cfg);
      setHubMsg({ ok: true, text: "Réseau enregistré sur le hub." });
    } catch (e) {
      setHubMsg({ ok: false, text: String(e) });
    } finally {
      setHubSaving(false);
    }
  }

  async function handleHubSync() {
    if (!selected) return;
    setHubSyncing(true);
    setHubMsg(null);
    try {
      const count = await tauriInvoke<number>("hub_sync", { networkCid: selected.cid_short });
      setHubMsg({ ok: true, text: `Synchronisation terminée — ${count} message(s) reçu(s).` });
    } catch (e) {
      setHubMsg({ ok: false, text: String(e) });
    } finally {
      setHubSyncing(false);
    }
  }

  async function handleNodeSettingsSave() {
    setNodeSaving(true);
    try {
      await tauriInvoke("node_settings_set", {
        tcpPort:      parseInt(nodeTcpPort) || 0,
        wsPort:       parseInt(nodeWsPort)  || 0,
        externalAddr: nodeExternalAddr.trim(),
      });
      const s = await tauriInvoke<NodeSettingsInfo>("node_settings_get");
      setNodeSettings(s);
      alert("Paramètres enregistrés. Redémarrez l'application pour les appliquer.");
    } catch (e) {
      alert(String(e));
    } finally {
      setNodeSaving(false);
    }
  }

  async function handleRccRegister() {
    if (!selected || !rccEmail.trim()) return;
    setRccRegistering(true);
    try {
      const info = await tauriInvoke<RccStatusInfo>("rcc_register", {
        networkCid: selected.cid_short,
        adminEmail: rccEmail.trim(),
      });
      setRccStatuses((prev) => ({ ...prev, [info.network_cid_short]: info }));
      setRccEmail("");
    } catch (e) {
      alert(String(e));
    } finally {
      setRccRegistering(false);
    }
  }

  async function handleRccForceRetry() {
    if (!selected) return;
    if (!confirm("Forcer le ré-enregistrement au RCC ? L'ancien enregistrement sera écrasé.")) return;
    setRccRegistering(true);
    try {
      const info = await tauriInvoke<RccStatusInfo>("rcc_force_retry", {
        networkCid: selected.cid_short,
      });
      setRccStatuses((prev) => ({ ...prev, [info.network_cid_short]: info }));
    } catch (e) {
      alert(String(e));
    } finally {
      setRccRegistering(false);
    }
  }

  async function handlePairRevoke(deviceId: string) {
    try {
      await tauriInvoke("pair_revoke", { deviceId });
      refreshPairedDevices();
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
      refreshActivity(selected.cid_short);
      refreshOutboxCounts();
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

  async function handleJoinNetwork() {
    if (!joinInviteLink.trim() || !joinDisplayName.trim()) return;
    setJoining(true);
    try {
      let net: NetworkInfo;
      if (joinPeerAddr.trim()) {
        net = await tauriInvoke<NetworkInfo>("network_join_p2p", {
          inviteLink: joinInviteLink.trim(),
          displayName: joinDisplayName.trim(),
          peerAddr: joinPeerAddr.trim(),
        });
      } else {
        net = await tauriInvoke<NetworkInfo>("network_join", {
          inviteLink: joinInviteLink.trim(),
          displayName: joinDisplayName.trim(),
        });
      }
      const nets = await tauriInvoke<NetworkInfo[]>("network_list");
      setNetworks(nets);
      selectedRef.current = net;
      setSelected(net);
      setActiveView('messages');
      setShowJoinForm(false);
      setJoinInviteLink("");
      setJoinPeerAddr("");
      setJoinDisplayName("");
    } catch (e) {
      alert(String(e));
    } finally {
      setJoining(false);
    }
  }

  async function handleCreateNetwork() {
    if (!createName.trim()) return;
    setCreating(true);
    try {
      const name = createName.trim();
      const net = await tauriInvoke<NetworkInfo>("network_create", { name, displayName: name, privacy: !createIsPublic });
      const createdCid = net.cid_short;
      // Auto-register to RCC if email provided
      if (createAdminEmail.trim()) {
        tauriInvoke("rcc_register", {
          networkCid: createdCid,
          adminEmail: createAdminEmail.trim(),
        }).catch(() => {});
      }
      const nets = await tauriInvoke<NetworkInfo[]>("network_list");
      setNetworks(nets);
      const created = nets.find((n) => n.cid_short === createdCid) ?? nets.find((n) => n.name === name);
      if (created) {
        selectedRef.current = created;
        setSelected(created);
        setActiveView('messages');
      }
      setShowCreateForm(false);
      setCreateName("");
      setCreateAdminEmail("");
    } catch (e) {
      alert(String(e));
    } finally {
      setCreating(false);
    }
  }

  const circleLabel = (c: number) =>
    ["Annuaire", "Connaissance", "Confiance", "Intime"][c] ?? `Cercle ${c}`;

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <aside className="w-64 bg-civium-900 text-white flex flex-col">
        {/* Mon nœud */}
        <div className="px-4 py-3 border-b border-civium-700">
          <p className="text-xs font-semibold text-civium-400 uppercase tracking-wider mb-2">Mon nœud</p>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className={`w-2 h-2 rounded-full flex-shrink-0 ${nodeStatus.running ? "bg-green-400" : "bg-gray-500"}`} />
              <span className="text-xs text-civium-100 font-mono truncate max-w-[110px]">
                {identity ? identity.cid_short : "…"}
              </span>
            </div>
            <button
              onClick={() => {
                const next = !showSettings;
                setShowSettings(next);
                setActiveView('messages');
                setShowCreateForm(false);
                setShowJoinForm(false);
              }}
              className={`text-xs px-2 py-1 rounded-lg transition-colors ${
                showSettings ? "bg-civium-600 text-white" : "text-civium-300 hover:bg-civium-700"
              }`}
              title="Paramètres du nœud"
            >
              ⚙
            </button>
          </div>
        </div>

        {/* Mes réseaux header */}
        <div className="px-4 pt-3 pb-1 flex items-center justify-between">
          <p className="text-xs font-semibold text-civium-400 uppercase tracking-wider">Mes réseaux</p>
          <div className="flex gap-1">
            <button
              onClick={() => { setShowJoinForm((v) => !v); setShowCreateForm(false); setShowSettings(false); }}
              className="text-civium-300 hover:text-white hover:bg-civium-700 rounded-lg w-6 h-6 flex items-center justify-center text-xs transition-colors"
              title="Rejoindre un réseau existant"
            >
              ↩
            </button>
            {networks.length === 0 && (
              <button
                onClick={() => { setShowCreateForm((v) => !v); setShowJoinForm(false); setShowSettings(false); setActiveView('messages'); }}
                className="text-civium-300 hover:text-white hover:bg-civium-700 rounded-lg w-6 h-6 flex items-center justify-center text-base transition-colors"
                title="Créer votre réseau"
              >
                +
              </button>
            )}
          </div>
        </div>

        {/* Network list + plugin sub-nav */}
        <nav className="flex-1 overflow-y-auto px-3 py-2 space-y-1">
          {networks.length === 0 && !showCreateForm && (
            <p className="text-xs text-civium-100 px-2 py-2">Aucun réseau. Cliquez sur + pour en créer un.</p>
          )}
          {networks.map((net) => {
            const isSelected = selected?.cid_short === net.cid_short;
            const navItem = (view: ActiveView, icon: string, label: string, badge?: number) => (
              <button
                key={view}
                onClick={() => { setActiveView(view); setShowSettings(false); setShowCreateForm(false); }}
                className={`w-full text-left pl-8 pr-3 py-1.5 rounded-lg text-xs transition-colors flex items-center gap-2 ${
                  isSelected && activeView === view && !showSettings && !showCreateForm
                    ? "bg-civium-500 text-white"
                    : "text-civium-200 hover:bg-civium-700"
                }`}
              >
                <span>{icon}</span>
                <span className="flex-1">{label}</span>
                {badge !== undefined && badge > 0 && (
                  <span className="text-xs bg-red-500 text-white rounded-full px-1.5 py-0.5 min-w-[1.2rem] text-center">{badge}</span>
                )}
              </button>
            );
            const enabledPluginIds = plugins.filter((p) => p.state === "enabled").map((p) => p.id);
            const hasAgenda = enabledPluginIds.some((id) => id.includes("agenda"));
            const hasDocuments = enabledPluginIds.some((id) => id.includes("document"));
            return (
              <div key={net.cid_short}>
                <button
                  onClick={() => {
                    selectedRef.current = net;
                    setSelected(net);
                    setActiveView('messages');
                    setShowSettings(false);
                    setShowCreateForm(false);
                    setShowJoinForm(false);
                  }}
                  className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                    isSelected && !showSettings && !showCreateForm
                      ? "bg-civium-700 text-white"
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
                    {net.is_public && (
                      <span className="text-xs bg-green-600 text-white px-1 py-0.5 rounded" title="Réseau public — visible dans les annuaires">🌐</span>
                    )}
                    <span className="ml-auto flex items-center gap-1">
                      {(outboxCounts[net.cid_short] ?? 0) > 0 && (
                        <span className="text-xs bg-amber-400 text-amber-900 rounded-full px-1.5 py-0.5 min-w-[1.2rem] text-center" title="Messages en attente">
                          ↑{outboxCounts[net.cid_short]}
                        </span>
                      )}
                      {rccStatuses[net.cid_short]?.status === "registered" && (
                        <span className="text-xs text-green-400" title="Enregistré au RCC">✓</span>
                      )}
                      {rccStatuses[net.cid_short]?.status === "pending" && (
                        <span className="text-xs text-amber-400" title="Enregistrement en cours…">↻</span>
                      )}
                    </span>
                  </div>
                  <div className="text-xs opacity-60">
                    {net.is_directory ? "Annuaire" : net.is_rrm ? "Registre malveillants" : `${net.member_count} membre(s)`}
                  </div>
                </button>
                {/* Plugin sub-navigation when this network is selected */}
                {isSelected && !showSettings && !showCreateForm && (
                  <div className="mt-0.5 space-y-0.5">
                    {navItem('messages', '💬', 'Messages', unreadCount > 0 ? unreadCount : undefined)}
                    {navItem('membres', '👥', 'Membres')}
                    {navItem('gouvernance', '🗳', 'Gouvernance')}
                    {navItem('activite', '📊', 'Activité')}
                    {navItem('notifications', '🔔', 'Notifications', unreadCount > 0 ? unreadCount : undefined)}
                    {hasAgenda && navItem('agenda', '📅', 'Agenda')}
                    {hasDocuments && navItem('documents', '📄', 'Documents')}
                    {net.is_directory && navItem('annuaire', '🔍', 'Annuaire')}
                    {net.is_rrm && navItem('rrm', '🚫', 'Réseaux signalés')}
                    {navItem('extensions', '🧩', 'Extensions')}
                  </div>
                )}
              </div>
            );
          })}
        </nav>

        {/* Statut réseau */}
        <div className="px-4 py-2 border-t border-civium-700">
          <span className="text-xs text-civium-500">{nodeStatus.running ? "Connecté" : "Hors ligne"}</span>
        </div>
      </aside>

      {/* Main */}
      <main className="flex-1 overflow-y-auto">
        {/* Bannière connexion réseau racine */}
        {rootConnected && (
          <div className="bg-civium-600 text-white px-6 py-2 flex items-center gap-2 text-sm">
            <span className="font-semibold">Connecté au réseau Civium</span>
            <span className="text-civium-200">— votre réseau ({rootConnected}) est maintenant dans l'annuaire public.</span>
            <button
              onClick={() => setRootConnected(null)}
              className="ml-auto text-civium-200 hover:text-white transition-colors text-xs"
            >
              ✕
            </button>
          </div>
        )}

        {/* Fraud alert banners */}
        {activeAlerts.length > 0 && (
          <div className="bg-red-600 text-white px-6 py-3 space-y-1">
            {activeAlerts.map((al, i) => (
              <div key={i} className="flex items-start gap-2 text-sm">
                <span className="font-bold uppercase shrink-0">[{al.alert_type}]</span>
                <span>{al.description}</span>
                {al.network_cids.length > 0 && (
                  <span className="ml-2 text-red-200 text-xs">
                    Réseaux : {al.network_cids.join(", ")}
                  </span>
                )}
              </div>
            ))}
          </div>
        )}

        {showSettings ? (
          /* ══════════════════════════════════════════════════════
             PANNEAU PARAMÈTRES
             ══════════════════════════════════════════════════════ */
          <div className="max-w-2xl mx-auto py-8 px-6 space-y-8">
            <div className="flex items-center justify-between">
              <h2 className="text-2xl font-bold text-gray-900">Paramètres</h2>
              <button
                className="text-xs text-gray-400 hover:text-gray-600"
                onClick={() => setShowSettings(false)}
              >
                ✕ Fermer
              </button>
            </div>

            {/* ── Identité ── */}
            <section>
              <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-1">Identité</h3>
              <p className="text-xs text-gray-400 mb-3">
                Ces informations vous permettent de vous connecter depuis le client web ou un autre appareil. Ne partagez jamais votre clé secrète.
              </p>
              <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-5 space-y-4">
                {identity ? (
                  <>
                    <div>
                      <p className="text-xs text-gray-500 mb-1">CID complet</p>
                      <div className="flex items-center gap-2">
                        <code className="flex-1 text-xs font-mono bg-gray-50 border border-gray-200 rounded px-2 py-1.5 break-all">{identity.cid_full}</code>
                        <button
                          className="text-xs text-civium-600 hover:text-civium-800 border border-civium-200 rounded px-2 py-1 shrink-0"
                          onClick={() => navigator.clipboard.writeText(identity!.cid_full)}
                        >Copier</button>
                      </div>
                    </div>
                    <div>
                      <p className="text-xs text-gray-500 mb-1">Clé secrète <span className="text-red-400">(confidentielle)</span></p>
                      <div className="flex items-center gap-2">
                        <code className="flex-1 text-xs font-mono bg-gray-50 border border-gray-200 rounded px-2 py-1.5 break-all">
                          {secretVisible ? identity.secret_b58 : "•".repeat(44)}
                        </code>
                        <button
                          className="text-xs border border-gray-200 rounded px-2 py-1 shrink-0 hover:bg-gray-50"
                          onClick={() => setSecretVisible((v) => !v)}
                        >{secretVisible ? "Masquer" : "Afficher"}</button>
                        {secretVisible && (
                          <button
                            className="text-xs text-civium-600 hover:text-civium-800 border border-civium-200 rounded px-2 py-1 shrink-0"
                            onClick={() => navigator.clipboard.writeText(identity!.secret_b58)}
                          >Copier</button>
                        )}
                      </div>
                    </div>
                  </>
                ) : (
                  <p className="text-xs text-gray-400">Chargement…</p>
                )}
              </div>
            </section>

            {/* ── Connexion internet ── */}
            <section>
              <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-1">Connexion internet</h3>
              <p className="text-xs text-gray-400 mb-3">
                Votre application communique directement avec les autres membres (pair-à-pair). Ces ports permettent d'être joignable depuis l'extérieur.
              </p>
              <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-5 space-y-4">
                <div className="flex items-center gap-2">
                  <span className={`w-2 h-2 rounded-full flex-shrink-0 ${nodeStatus.running ? "bg-green-400" : "bg-gray-400"}`} />
                  <span className="text-sm font-medium text-gray-700">{nodeStatus.running ? "Nœud en ligne" : "Nœud hors ligne"}</span>
                </div>
                {nodeStatus.running && nodeStatus.listen_addrs.length > 0 && (
                  <div>
                    <p className="text-xs text-gray-500 mb-1">Adresses d'accès (cliquer pour copier) :</p>
                    <div className="space-y-1">
                      {nodeStatus.listen_addrs.map((a) => (
                        <div key={a} className="text-xs font-mono bg-gray-50 border border-gray-200 rounded px-2 py-1 cursor-pointer hover:bg-gray-100 truncate" onClick={() => navigator.clipboard.writeText(a)} title="Cliquer pour copier">{a}</div>
                      ))}
                    </div>
                  </div>
                )}
                <div className="grid grid-cols-2 gap-3 pt-1 border-t border-gray-100">
                  <div>
                    <label className="block text-xs text-gray-500 mb-1">Port principal (ex: 4001)</label>
                    <input type="number" min="1024" max="65535" className="w-full text-sm border border-gray-200 rounded px-2 py-1.5" placeholder="0 = automatique" value={nodeTcpPort} onChange={(e) => setNodeTcpPort(e.target.value)} />
                  </div>
                  <div>
                    <label className="block text-xs text-gray-500 mb-1">Port web (ex: 4002)</label>
                    <input type="number" min="1024" max="65535" className="w-full text-sm border border-gray-200 rounded px-2 py-1.5" placeholder="0 = automatique" value={nodeWsPort} onChange={(e) => setNodeWsPort(e.target.value)} />
                  </div>
                </div>
                <div>
                  <label className="block text-xs text-gray-500 mb-1">Adresse publique (si vous avez une IP fixe)</label>
                  <input type="text" className="w-full text-sm font-mono border border-gray-200 rounded px-2 py-1.5" placeholder="Laisser vide si vous n'avez pas d'IP publique fixe" value={nodeExternalAddr} onChange={(e) => setNodeExternalAddr(e.target.value)} />
                </div>
                <button className="text-sm bg-civium-600 text-white px-3 py-1.5 rounded hover:bg-civium-700 disabled:opacity-50" disabled={nodeSaving} onClick={handleNodeSettingsSave}>
                  {nodeSaving ? "Enregistrement…" : "Enregistrer (redémarrage requis)"}
                </button>
              </div>
            </section>

            {/* ── Synchronisation serveur (Hub) ── */}
            {selected && (
              <section>
                <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-1">Synchronisation via serveur</h3>
                <p className="text-xs text-gray-400 mb-3">
                  Si vous avez un serveur web, vous pouvez l'utiliser comme relais pour que votre réseau reste accessible même quand votre ordinateur est éteint.
                </p>
                <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-5 space-y-3">
                  {hubMsg && (
                    <p className={`text-xs px-3 py-2 rounded border ${hubMsg.ok ? "bg-green-50 border-green-200 text-green-700" : "bg-red-50 border-red-200 text-red-700"}`}>{hubMsg.text}</p>
                  )}
                  <div className="flex gap-2">
                    <input type="url" className="flex-1 text-sm font-mono border border-gray-200 rounded px-2 py-1.5" placeholder="https://votre-serveur.com/civium" value={hubUrl} onChange={(e) => setHubUrl(e.target.value)} />
                    <button className="text-sm bg-civium-600 text-white px-3 py-1.5 rounded hover:bg-civium-700 disabled:opacity-50" disabled={hubSaving || !hubUrl.trim()} onClick={handleHubSave}>
                      {hubSaving ? "…" : hubConfig ? "Mettre à jour" : "Connecter"}
                    </button>
                  </div>
                  {hubConfig && (
                    <div className="flex items-center gap-3 text-xs text-gray-500">
                      <span>Dernière sync : {hubConfig.last_sync_ts ? new Date(hubConfig.last_sync_ts * 1000).toLocaleString("fr-FR") : "jamais"}</span>
                      <button className="ml-auto text-sm border border-civium-300 text-civium-700 px-3 py-1.5 rounded hover:bg-civium-50 disabled:opacity-50" disabled={hubSyncing} onClick={handleHubSync}>
                        {hubSyncing ? "Synchronisation…" : "Synchroniser maintenant"}
                      </button>
                    </div>
                  )}
                </div>
              </section>
            )}

            {/* ── Enregistrement légal (RCC) ── */}
            {selected && (
              <section>
                <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-1">Enregistrement légal</h3>
                <p className="text-xs text-gray-400 mb-3">
                  Tout réseau Civium doit être déclaré auprès du registre central. C'est obligatoire et gratuit — votre email reste confidentiel.
                </p>
                <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-5">
                  {(() => {
                    const rcc = rccStatuses[selected.cid_short];
                    if (rcc?.status === "registered") return (
                      <div className="space-y-2">
                        <div className="flex items-center gap-2 text-sm text-green-700 bg-green-50 border border-green-200 rounded px-3 py-2">
                          <span className="text-base">✓</span>
                          <span className="flex-1">Réseau déclaré — {rcc.admin_email}</span>
                          <button className="text-xs text-gray-400 hover:text-gray-600 underline" disabled={rccRegistering} onClick={handleRccForceRetry}>Ré-enregistrer</button>
                        </div>
                      </div>
                    );
                    if (rcc?.status === "pending") return (
                      <div className="flex flex-col gap-2 text-sm text-amber-700 bg-amber-50 border border-amber-200 rounded px-3 py-2">
                        <div className="flex items-center gap-2">
                          <span className="text-base animate-spin inline-block">↻</span>
                          <span className="flex-1">Déclaration en cours… ({rcc.attempts} tentative{rcc.attempts > 1 ? "s" : ""})</span>
                          <button className="text-xs text-amber-600 hover:text-amber-800 underline shrink-0" disabled={rccRegistering} onClick={handleRccForceRetry}>Ré-essayer</button>
                        </div>
                        <button className="text-xs text-left text-amber-500 hover:text-amber-700 underline"
                          onClick={async () => {
                            if (!selected) return;
                            if (!confirm("Le serveur a déjà confirmé la déclaration ? Cela va juste mettre à jour l'état local.")) return;
                            try {
                              const info = await tauriInvoke<RccStatusInfo>("rcc_mark_registered", { networkCid: selected.cid_short });
                              setRccStatuses((prev) => ({ ...prev, [info.network_cid_short]: info }));
                            } catch (e) { alert(String(e)); }
                          }}>
                          Le serveur a déjà confirmé → marquer comme déclaré
                        </button>
                      </div>
                    );
                    if (rcc?.status === "failed") return (
                      <div className="space-y-3">
                        <p className="text-xs text-red-600">Déclaration échouée après {rcc.attempts} tentatives.</p>
                        <button className="text-xs text-red-500 hover:text-red-700 underline" disabled={rccRegistering} onClick={handleRccForceRetry}>Ré-essayer</button>
                      </div>
                    );
                    return (
                      <div className="space-y-3">
                        <p className="text-xs text-gray-500">Entrez votre adresse email pour déclarer ce réseau. Elle servira uniquement à vous contacter en cas d'alerte de sécurité.</p>
                        <div className="flex gap-2">
                          <input type="email" className="flex-1 text-sm border border-gray-200 rounded px-2 py-1.5" placeholder="votre@email.com" value={rccEmail} onChange={(e) => setRccEmail(e.target.value)} onKeyDown={(e) => { if (e.key === "Enter") handleRccRegister(); }} />
                          <button className="text-sm bg-civium-600 text-white px-3 py-1.5 rounded hover:bg-civium-700 disabled:opacity-50" disabled={rccRegistering || !rccEmail.trim()} onClick={handleRccRegister}>
                            {rccRegistering ? "…" : "Déclarer"}
                          </button>
                        </div>
                      </div>
                    );
                  })()}
                </div>
              </section>
            )}

            {/* ── Fédération (ActivityPub) ── */}
            {selected && (
              <section>
                <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-1">Fédération avec d'autres réseaux</h3>
                <p className="text-xs text-gray-400 mb-3">
                  Activez cette option pour que votre réseau soit visible et suivi depuis Mastodon, PeerTube et d'autres plateformes compatibles.
                </p>
                <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-5 space-y-3">
                  {apError && <p className="text-xs text-red-600">{apError}</p>}
                  {selected.ap_enabled ? (
                    <div className="space-y-3">
                      {selected.ap_actor_url && (
                        <div className="bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-xs font-mono break-all text-gray-700 cursor-pointer hover:bg-gray-100" onClick={() => navigator.clipboard.writeText(selected.ap_actor_url!)} title="Cliquer pour copier">
                          {selected.ap_actor_url} <span className="text-gray-400 font-sans">(copier)</span>
                        </div>
                      )}
                      <p className="text-xs text-gray-500">{apFollowers.length} abonné{apFollowers.length !== 1 ? "s" : ""}</p>
                      <button onClick={handleApDisable} disabled={apEnabling} className="text-xs text-red-600 hover:text-red-700 disabled:opacity-50">
                        {apEnabling ? "…" : "Désactiver la fédération"}
                      </button>
                    </div>
                  ) : (
                    <button onClick={handleApEnable} disabled={apEnabling} className="text-sm px-4 py-2 bg-civium-600 text-white rounded-lg hover:bg-civium-700 disabled:opacity-50">
                      {apEnabling ? "Activation…" : "Activer la fédération"}
                    </button>
                  )}
                </div>
              </section>
            )}

            {/* ── Assistant IA (MCP) ── */}
            <section>
              <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-1">Assistant IA (Claude, etc.)</h3>
              <p className="text-xs text-gray-400 mb-3">
                Permet à un assistant IA d'accéder en lecture seule à vos réseaux pour vous aider. Les données restent chiffrées et sous votre contrôle.
              </p>
              <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-5 space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium text-gray-700">État</span>
                  <span className={`text-xs px-2 py-0.5 rounded-full ${mcpStatus.running ? "bg-green-100 text-green-700" : "bg-gray-100 text-gray-500"}`}>
                    {mcpStatus.running ? "Actif" : "Inactif"}
                  </span>
                </div>
                {!mcpStatus.running ? (
                  <div className="flex items-center gap-3">
                    <div className="flex items-center gap-2">
                      <label className="text-xs text-gray-500">Port :</label>
                      <input type="number" min={1024} max={65535} value={mcpPort} onChange={(e) => setMcpPort(e.target.value)} className="w-24 text-sm border border-gray-200 rounded px-2 py-1 font-mono" />
                    </div>
                    <button onClick={handleMcpStart} className="text-sm bg-indigo-500 text-white rounded-lg px-4 py-1.5 hover:bg-indigo-600">Démarrer</button>
                  </div>
                ) : (
                  <div className="space-y-3">
                    <div className="bg-gray-50 rounded-lg p-3 space-y-2">
                      <div className="flex items-center justify-between">
                        <span className="text-xs text-gray-500">Adresse :</span>
                        <code className="text-xs font-mono text-indigo-700">{mcpStatus.url}</code>
                      </div>
                      <div className="flex items-center justify-between gap-2">
                        <span className="text-xs text-gray-500">Clé d'accès :</span>
                        <div className="flex items-center gap-2">
                          {showMcpToken ? <code className="text-xs font-mono text-gray-700 break-all">{mcpStatus.token}</code> : <span className="text-xs text-gray-400 font-mono">{"•".repeat(16)}</span>}
                          <button onClick={() => setShowMcpToken((v) => !v)} className="text-xs text-indigo-400 hover:text-indigo-600">{showMcpToken ? "Masquer" : "Afficher"}</button>
                        </div>
                      </div>
                    </div>
                    <button onClick={handleMcpStop} className="text-sm border border-red-200 text-red-600 rounded-lg px-4 py-1.5 hover:bg-red-50">Arrêter</button>
                  </div>
                )}
              </div>
            </section>

            {/* ── Appareils jumelés ── */}
            <section>
              <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-1">Mes appareils</h3>
              <p className="text-xs text-gray-400 mb-3">
                Utilisez votre compte Civium sur plusieurs appareils (téléphone, second ordinateur…).
              </p>
              <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-5 space-y-4">
                {pairedDevices.length > 0 && (
                  <div className="divide-y divide-gray-100 border border-gray-100 rounded-xl">
                    {pairedDevices.map((d) => (
                      <div key={d.id} className="flex items-center px-4 py-3 gap-3">
                        <div className="flex-1 min-w-0">
                          <div className="text-sm font-medium text-gray-800">{d.label}</div>
                          <div className="text-xs text-gray-400">Ajouté le {new Date(d.paired_at * 1000).toLocaleDateString("fr-FR")}{d.revoked && <span className="ml-2 text-red-500">— révoqué</span>}</div>
                        </div>
                        {!d.revoked && <button onClick={() => handlePairRevoke(d.id)} className="text-xs border border-red-200 text-red-600 px-3 py-1.5 rounded-lg hover:bg-red-50">Retirer</button>}
                      </div>
                    ))}
                  </div>
                )}
                {pairingSession ? (
                  <div className="bg-indigo-50 rounded-xl p-4 space-y-2">
                    <p className="text-xs font-medium text-indigo-800">Lien de connexion (valable 10 min) :</p>
                    <code className="block text-xs font-mono text-indigo-700 break-all bg-white p-2 rounded border border-indigo-100">{pairingSession.link}</code>
                    <p className="text-xs text-indigo-600">Copiez ce lien sur le second appareil.</p>
                    <button onClick={() => setPairingSession(null)} className="text-xs text-indigo-400 hover:text-indigo-600">Fermer</button>
                  </div>
                ) : (
                  <div className="flex items-center gap-3">
                    <input type="text" placeholder="Nom du nouvel appareil" value={pairLabel} onChange={(e) => setPairLabel(e.target.value)} className="flex-1 text-sm border border-gray-200 rounded-lg px-3 py-1.5" />
                    <button onClick={handlePairInit} disabled={!pairLabel.trim()} className="text-sm bg-civium-600 text-white rounded-lg px-4 py-1.5 hover:bg-civium-700 disabled:opacity-50">Ajouter un appareil</button>
                  </div>
                )}
                <div className="border-t border-gray-100 pt-3">
                  <button onClick={() => setShowPairCompleteForm((v) => !v)} className="text-xs text-gray-500 hover:text-gray-700">{showPairCompleteForm ? "▲ Masquer" : "▼ J'ai reçu un lien de jumelage"}</button>
                  {showPairCompleteForm && (
                    <div className="mt-3 space-y-2">
                      <input type="text" placeholder="Lien civium://pair/…" value={pairLink} onChange={(e) => setPairLink(e.target.value)} className="w-full text-sm border border-gray-200 rounded-lg px-3 py-1.5 font-mono" />
                      <div className="flex items-center gap-3">
                        <input type="text" placeholder="Nom de cet appareil" value={pairCompleteLabel} onChange={(e) => setPairCompleteLabel(e.target.value)} className="flex-1 text-sm border border-gray-200 rounded-lg px-3 py-1.5" />
                        <button onClick={handlePairComplete} disabled={!pairLink.trim() || !pairCompleteLabel.trim()} className="text-sm bg-green-600 text-white rounded-lg px-4 py-1.5 hover:bg-green-700 disabled:opacity-50">Valider</button>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </section>

            {/* ── Plugins ── */}
            <section>
              <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-1">Extensions (Plugins)</h3>
              <p className="text-xs text-gray-400 mb-3">
                Les extensions ajoutent des fonctionnalités à vos réseaux. Celles marquées "système" sont indispensables au fonctionnement.
              </p>
              <div className="bg-white rounded-2xl shadow-sm border border-gray-100 divide-y divide-gray-100">
                {plugins.length === 0 && <p className="px-4 py-6 text-sm text-gray-400 text-center">Aucune extension installée.</p>}
                {plugins.map((p) => (
                  <div key={p.id} className="px-4 py-4 flex items-start gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className="text-sm font-semibold text-gray-800">{p.name}</span>
                        {p.is_system && <span className="text-xs px-1.5 py-0.5 bg-gray-100 text-gray-500 rounded">système</span>}
                        {p.certification === "certified" && <span className="text-xs px-1.5 py-0.5 bg-indigo-100 text-indigo-700 rounded">Certifié</span>}
                        <span className={`text-xs px-1.5 py-0.5 rounded ${p.state === "enabled" ? "bg-green-100 text-green-700" : "bg-gray-100 text-gray-500"}`}>{p.state === "enabled" ? "actif" : "inactif"}</span>
                      </div>
                      <p className="text-xs text-gray-500 mt-0.5">{p.description}</p>
                    </div>
                    {!p.is_system && (
                      <button className={`flex-shrink-0 text-xs px-3 py-1.5 rounded-lg border transition-colors disabled:opacity-50 ${p.state === "enabled" ? "border-gray-200 text-gray-600 hover:bg-gray-50" : "border-civium-200 text-civium-700 bg-civium-50 hover:bg-civium-100"}`} disabled={togglingPlugin === p.id} onClick={() => handleTogglePlugin(p.id, p.state)}>
                        {togglingPlugin === p.id ? "…" : p.state === "enabled" ? "Désactiver" : "Activer"}
                      </button>
                    )}
                  </div>
                ))}
              </div>
            </section>

            {/* ── Zone de danger ── */}
            {selected && selected.member_count <= 1 && (
              <section className="border border-red-200 rounded-xl p-4">
                <h3 className="text-sm font-semibold text-red-600 uppercase tracking-wide mb-3">Zone de danger</h3>
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-800">Supprimer ce réseau</p>
                    <p className="text-xs text-gray-400 mt-0.5">Action irréversible. Toutes les données locales seront effacées.</p>
                  </div>
                  <button
                    onClick={async () => {
                      if (!confirm(`Supprimer définitivement le réseau « ${selected.name} » ? Cette action est irréversible.`)) return;
                      try {
                        await tauriInvoke("network_delete", { networkCid: selected.cid_short });
                        const nets = await tauriInvoke<NetworkInfo[]>("network_list");
                        setNetworks(nets);
                        setSelected(nets[0] ?? null);
                        setShowSettings(false);
                      } catch (e) {
                        alert(String(e));
                      }
                    }}
                    className="text-xs border border-red-300 text-red-600 rounded-lg px-3 py-1.5 hover:bg-red-50 transition-colors shrink-0"
                  >
                    Supprimer
                  </button>
                </div>
              </section>
            )}
          </div>

        ) : showJoinForm ? (
          /* ══ PANNEAU REJOINDRE UN RÉSEAU ══ */
          <div className="max-w-lg mx-auto py-12 px-6">
            <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-8 space-y-6">
              <div className="flex items-center justify-between">
                <h2 className="text-xl font-bold text-gray-900">Rejoindre un réseau</h2>
                <button
                  className="text-gray-400 hover:text-gray-600 text-sm"
                  onClick={() => { setShowJoinForm(false); setJoinInviteLink(""); setJoinPeerAddr(""); setJoinDisplayName(""); }}
                >✕</button>
              </div>

              <p className="text-sm text-gray-500">
                Vous avez reçu une invitation d'un administrateur Civium. Collez les informations ci-dessous.
              </p>

              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Lien d'invitation</label>
                  <textarea
                    autoFocus
                    rows={3}
                    className="w-full border border-gray-200 rounded-lg px-3 py-2 text-xs font-mono focus:outline-none focus:ring-2 focus:ring-civium-400 resize-none placeholder:font-sans placeholder:text-gray-400"
                    placeholder="civium-invite:…"
                    value={joinInviteLink}
                    onChange={(e) => setJoinInviteLink(e.target.value)}
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Adresse de connexion de l'admin
                    <span className="ml-1 text-xs font-normal text-gray-400">(fournie avec l'invitation)</span>
                  </label>
                  <input
                    type="text"
                    className="w-full border border-gray-200 rounded-lg px-3 py-2 text-xs font-mono focus:outline-none focus:ring-2 focus:ring-civium-400 placeholder:font-sans placeholder:text-gray-400"
                    placeholder="/ip4/1.2.3.4/tcp/4001/p2p/12D3…"
                    value={joinPeerAddr}
                    onChange={(e) => setJoinPeerAddr(e.target.value)}
                  />
                  <p className="text-xs text-gray-400 mt-1">
                    Laissez vide seulement si l'admin vous a donné accès direct à sa base de données.
                  </p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Votre nom dans ce réseau</label>
                  <input
                    type="text"
                    className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-civium-400"
                    placeholder="Ex : Marie, Pierre…"
                    value={joinDisplayName}
                    onChange={(e) => setJoinDisplayName(e.target.value)}
                    onKeyDown={(e) => { if (e.key === "Enter") handleJoinNetwork(); }}
                  />
                </div>

                <button
                  className="w-full py-2.5 bg-civium-600 text-white rounded-xl font-semibold text-sm hover:bg-civium-700 disabled:opacity-50 transition-colors"
                  disabled={joining || !joinInviteLink.trim() || !joinDisplayName.trim()}
                  onClick={handleJoinNetwork}
                >
                  {joining
                    ? (joinPeerAddr.trim() ? "Connexion en cours…" : "Jonction…")
                    : "Rejoindre le réseau"}
                </button>

                {joining && joinPeerAddr.trim() && (
                  <p className="text-xs text-gray-400 text-center">
                    Connexion au nœud de l'admin… (jusqu'à 30 secondes)
                  </p>
                )}
              </div>
            </div>
          </div>

        ) : showCreateForm ? (
          /* ══ PANNEAU CRÉATION DE RÉSEAU ══ */
          <div className="max-w-lg mx-auto py-12 px-6">
            <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-8 space-y-6">
              <div className="flex items-center justify-between">
                <h2 className="text-xl font-bold text-gray-900">Créer un réseau</h2>
                <button className="text-gray-400 hover:text-gray-600 text-sm" onClick={() => setShowCreateForm(false)}>✕</button>
              </div>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Nom du réseau</label>
                  <input
                    type="text"
                    autoFocus
                    className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-civium-400"
                    placeholder="Ex : Famille Martin, Équipe projet…"
                    value={createName}
                    onChange={(e) => setCreateName(e.target.value)}
                    onKeyDown={(e) => { if (e.key === "Enter") handleCreateNetwork(); }}
                  />
                  <p className="text-xs text-gray-400 mt-1">
                    Un espace souverain pour votre groupe. Vous invitez les membres et définissez les règles.
                  </p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Votre email <span className="text-xs font-normal text-gray-400">(pour les alertes de sécurité)</span>
                  </label>
                  <input
                    type="email"
                    className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-civium-400"
                    placeholder="votre@email.com"
                    value={createAdminEmail}
                    onChange={(e) => setCreateAdminEmail(e.target.value)}
                  />
                  <p className="text-xs text-gray-400 mt-1">
                    Pour déclarer votre réseau au Registre Central Civium (RCC — registre légal obligatoire). Il ne sera jamais partagé publiquement.
                  </p>
                </div>
                {/* Visibilité */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">Visibilité</label>
                  <div className="flex gap-3">
                    <button
                      type="button"
                      onClick={() => setCreateIsPublic(false)}
                      className={`flex-1 py-2 px-3 rounded-lg border text-sm font-medium transition-colors ${
                        !createIsPublic
                          ? "bg-civium-50 border-civium-400 text-civium-700"
                          : "bg-white border-gray-200 text-gray-500 hover:bg-gray-50"
                      }`}
                    >
                      🔒 Privé
                    </button>
                    <button
                      type="button"
                      onClick={() => setCreateIsPublic(true)}
                      className={`flex-1 py-2 px-3 rounded-lg border text-sm font-medium transition-colors ${
                        createIsPublic
                          ? "bg-green-50 border-green-400 text-green-700"
                          : "bg-white border-gray-200 text-gray-500 hover:bg-gray-50"
                      }`}
                    >
                      🌐 Public
                    </button>
                  </div>
                  <p className="text-xs text-gray-400 mt-1">
                    {createIsPublic
                      ? "Votre réseau sera visible dans les annuaires Civium. Tout le monde peut demander à le rejoindre."
                      : "Votre réseau est sur invitation uniquement. Seules les personnes que vous invitez peuvent le rejoindre."}
                  </p>
                </div>
                <button
                  className="w-full py-2.5 bg-civium-600 text-white rounded-xl font-semibold text-sm hover:bg-civium-700 disabled:opacity-50 transition-colors"
                  disabled={creating || !createName.trim() || !createAdminEmail.trim()}
                  onClick={handleCreateNetwork}
                >
                  {creating ? "Création et déclaration…" : "Créer le réseau"}
                </button>
              </div>
            </div>
          </div>
        ) : !selected ? (
          <div className="flex flex-col items-center justify-center h-full gap-4 text-gray-400">
            {networks.length === 0 ? (
              <>
                <p>Créez votre réseau pour commencer.</p>
                <button
                  className="text-sm px-4 py-2 bg-civium-600 text-white rounded-lg hover:bg-civium-700 transition-colors"
                  onClick={() => setShowCreateForm(true)}
                >
                  + Créer mon réseau
                </button>
                <button
                  className="text-sm px-4 py-2 bg-white border border-gray-200 text-gray-600 rounded-lg hover:bg-gray-50 transition-colors"
                  onClick={() => setShowJoinForm(true)}
                >
                  Rejoindre un réseau existant
                </button>
              </>
            ) : (
              <p>Sélectionnez un réseau dans la barre latérale.</p>
            )}
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

            {/* Pending members — always visible as notification */}
            {activeView === 'membres' && pending.length > 0 && (
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
            {activeView === 'membres' && <section>
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
                      <div className="border-b border-gray-100 bg-gray-50">
                        {/* DM section */}
                        <div className="px-4 pt-2 pb-3">
                          <div className="flex items-center justify-between mb-2">
                            <p className="text-xs font-medium text-gray-500">Message direct</p>
                            <button
                              className={`text-xs px-2 py-0.5 rounded-full border transition-colors ${
                                dmE2EMode[m.cid_short]
                                  ? "bg-purple-50 border-purple-400 text-purple-700 hover:bg-purple-100"
                                  : "bg-gray-50 border-gray-300 text-gray-500 hover:bg-gray-100"
                              }`}
                              onClick={(e) => { e.stopPropagation(); setDmE2EMode((prev) => ({ ...prev, [m.cid_short]: !prev[m.cid_short] })); }}
                              title="Chiffrement de bout en bout (Cercle 3 — Intime)"
                            >
                              {dmE2EMode[m.cid_short] ? "🔒 E2E (Intime)" : "🔓 Direct"}
                            </button>
                          </div>
                          {messages
                            .filter((msg) => msg.is_direct && (msg.author_cid_short === m.cid_short || msg.to_cid_short === m.cid_short))
                            .slice(-5)
                            .map((msg) => (
                              <div key={msg.id} className={`text-xs mb-1 ${msg.author_cid_short === m.cid_short ? "text-gray-600" : "text-civium-700"}`}>
                                <span className="font-medium">{msg.author_name}</span>
                                {msg.is_e2e && <span className="ml-1 text-purple-500" title="Chiffrement E2E — Intime">🔒</span>}
                                <span className="text-gray-400"> {formatTime(msg.sent_at)} — </span>
                                {msg.body}
                              </div>
                            ))}
                          <div className="flex gap-2 mt-2" onClick={(e) => e.stopPropagation()}>
                            <input
                              type="text"
                              className={`flex-1 text-xs border rounded px-2 py-1 bg-white ${dmE2EMode[m.cid_short] ? "border-purple-300" : "border-gray-200"}`}
                              placeholder={dmE2EMode[m.cid_short] ? `Message E2E à ${m.display_name}…` : `Message à ${m.display_name}…`}
                              value={dmBody[m.cid_short] ?? ""}
                              onChange={(e) => setDmBody((prev) => ({ ...prev, [m.cid_short]: e.target.value }))}
                              onKeyDown={(e) => { if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleSendDm(m.cid_short); } }}
                            />
                            <button
                              className={`text-xs text-white px-2 py-1 rounded disabled:opacity-50 ${dmE2EMode[m.cid_short] ? "bg-purple-600 hover:bg-purple-700" : "bg-civium-600 hover:bg-civium-700"}`}
                              disabled={sendingDm === m.cid_short || !dmBody[m.cid_short]?.trim()}
                              onClick={(e) => { e.stopPropagation(); handleSendDm(m.cid_short); }}
                            >
                              {sendingDm === m.cid_short ? "…" : "Envoyer"}
                            </button>
                          </div>
                        </div>
                        {/* Admin section */}
                        {(() => {
                          const myRecord = members.find((x) => x.cid_short === identity?.cid_short);
                          const iAmAdmin = myRecord?.role === "admin";
                          const isMe = m.cid_short === identity?.cid_short;
                          return (
                        <div className="px-4 pb-3 space-y-2 border-t border-gray-100">
                          <div className="flex flex-wrap items-center gap-2 pt-2">
                            <span className="text-xs text-gray-500">Admin :</span>
                            {iAmAdmin && !isMe && (
                              <button
                                className={`text-xs px-2 py-0.5 rounded-full border transition-colors ${
                                  m.role === "admin"
                                    ? "bg-amber-50 border-amber-300 text-amber-700 hover:bg-amber-100"
                                    : "bg-gray-50 border-gray-300 text-gray-600 hover:bg-gray-100"
                                }`}
                                disabled={settingRole === m.cid_short}
                                onClick={(e) => { e.stopPropagation(); handleSetRole(m.cid_short, m.role === "admin" ? "member" : "admin"); }}
                              >
                                {settingRole === m.cid_short ? "…" : m.role === "admin" ? "Rétrograder membre" : "Promouvoir admin"}
                              </button>
                            )}
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
                            {iAmAdmin && !isMe && (
                              <button
                                className="text-xs px-2 py-0.5 rounded-full border border-red-200 bg-red-50 text-red-600 hover:bg-red-100 transition-colors"
                                disabled={removingMember === m.cid_short}
                                onClick={(e) => { e.stopPropagation(); handleRemoveMember(m.cid_short, m.display_name); }}
                              >
                                {removingMember === m.cid_short ? "…" : "Exclure du réseau"}
                              </button>
                            )}
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
                                  className="flex-1 text-xs border border-gray-200 rounded px-2 py-1 bg-white"
                                  placeholder="CID short du tuteur"
                                  value={newGuardianCid}
                                  onChange={(e) => setNewGuardianCid(e.target.value)}
                                />
                                <button
                                  className="text-xs bg-civium-600 text-white px-2 py-1 rounded hover:bg-civium-700 disabled:opacity-50"
                                  disabled={savingGuardian || !newGuardianCid.trim()}
                                  onClick={() => handleAddGuardian(m.cid_short)}
                                >
                                  {savingGuardian ? "…" : "Ajouter tuteur"}
                                </button>
                              </div>
                            </div>
                          )}
                        </div>
                          );
                        })()}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </section>}

            {/* Invite link */}
            {activeView === 'membres' && inviteLink && (
              <section>
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide mb-2">
                  Inviter quelqu'un
                </h3>
                <div className="bg-white border border-gray-200 rounded-xl p-4 space-y-4">

                  {/* Étape 1 — lien */}
                  <div>
                    <p className="text-xs font-semibold text-gray-700 mb-1">
                      Étape 1 — Envoyez ce lien d'invitation
                    </p>
                    <p className="text-xs text-gray-400 mb-2">
                      Par email, SMS ou messagerie. Ce lien identifie votre réseau et autorise la jonction.
                    </p>
                    <div className="flex gap-2">
                      <div className="flex-1 bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-xs font-mono break-all text-gray-600 select-all">
                        {inviteLink}
                      </div>
                      <button
                        className="flex-shrink-0 text-xs px-3 py-2 bg-civium-600 text-white rounded-lg hover:bg-civium-700 transition-colors"
                        onClick={() => navigator.clipboard.writeText(inviteLink)}
                      >
                        Copier
                      </button>
                    </div>
                  </div>

                  {/* Étape 2 — adresse P2P */}
                  <div>
                    <p className="text-xs font-semibold text-gray-700 mb-1">
                      Étape 2 — Envoyez aussi votre adresse de connexion
                    </p>
                    <p className="text-xs text-gray-400 mb-2">
                      L'invité en a besoin pour se connecter directement à votre nœud. Envoyez-la avec le lien ci-dessus.
                    </p>
                    {nodeStatus?.listen_addrs && nodeStatus.listen_addrs.length > 0 ? (
                      <div className="space-y-1">
                        {nodeStatus.listen_addrs.map((addr) => (
                          <div key={addr} className="flex gap-2">
                            <div className="flex-1 bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-xs font-mono break-all text-gray-600 select-all">
                              {addr}
                            </div>
                            <button
                              className="flex-shrink-0 text-xs px-3 py-2 bg-white border border-gray-200 text-gray-600 rounded-lg hover:bg-gray-50 transition-colors"
                              onClick={() => navigator.clipboard.writeText(addr)}
                            >
                              Copier
                            </button>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <div className="bg-amber-50 border border-amber-200 rounded-lg px-3 py-2 text-xs text-amber-700">
                        Nœud P2P non démarré — votre adresse de connexion n'est pas disponible. L'invité ne pourra pas se connecter tant que votre nœud est éteint.
                      </div>
                    )}
                  </div>

                  {/* Étape 3 — envoyer par email */}
                  <div>
                    <p className="text-xs font-semibold text-gray-700 mb-1">
                      Étape 3 — Envoyer par email (optionnel)
                    </p>
                    <p className="text-xs text-gray-400 mb-2">
                      Saisissez l'adresse email de l'invité pour préparer un email avec toutes les informations.
                    </p>
                    <div className="flex gap-2">
                      <input
                        type="email"
                        placeholder="prenom@exemple.fr"
                        value={inviteEmail}
                        onChange={(e) => setInviteEmail(e.target.value)}
                        className="flex-1 bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-xs text-gray-700 placeholder-gray-400 focus:outline-none focus:border-civium-400"
                      />
                      <button
                        disabled={!inviteEmail.trim()}
                        onClick={() => {
                          const addrs = nodeStatus?.listen_addrs ?? [];
                          const addrBlock = addrs.length > 0
                            ? addrs.map((a) => `  • ${a}`).join("\n")
                            : "  (nœud non démarré — démarrez votre nœud avant d'envoyer)";
                          const body = [
                            `Bonjour,`,
                            ``,
                            `Je vous invite à rejoindre mon réseau sur Civium.`,
                            ``,
                            `1. Téléchargez l'application Civium : https://civium.app`,
                            `2. Au démarrage, choisissez "Rejoindre un réseau"`,
                            `3. Collez ce lien d'invitation :`,
                            `   ${inviteLink}`,
                            `4. Collez mon adresse de connexion :`,
                            addrBlock,
                            `5. Choisissez votre nom et confirmez`,
                            ``,
                            `J'approuverai votre demande dès que je la recevrai.`,
                          ].join("\n");
                          const mailto = `mailto:${encodeURIComponent(inviteEmail.trim())}?subject=${encodeURIComponent("Invitation à rejoindre mon réseau Civium")}&body=${encodeURIComponent(body)}`;
                          window.open(mailto, "_blank");
                        }}
                        className="flex-shrink-0 text-xs px-3 py-2 bg-civium-600 text-white rounded-lg hover:bg-civium-700 disabled:opacity-40 transition-colors"
                      >
                        Envoyer par email
                      </button>
                    </div>
                  </div>

                  {/* Étape 4 — instructions pour l'invité */}
                  <div className="bg-gray-50 border border-gray-100 rounded-lg px-3 py-3 text-xs text-gray-500 space-y-1">
                    <p className="font-semibold text-gray-600">Ce que doit faire l'invité :</p>
                    <ol className="list-decimal list-inside space-y-1">
                      <li>Télécharger et ouvrir l'appli Civium</li>
                      <li>Au démarrage, choisir <strong>Rejoindre un réseau</strong></li>
                      <li>Coller le lien d'invitation (étape 1)</li>
                      <li>Coller votre adresse de connexion (étape 2)</li>
                      <li>Choisir son nom et confirmer</li>
                    </ol>
                    <p className="pt-1">Vous recevrez une demande d'admission à approuver ici.</p>
                  </div>
                </div>
              </section>
            )}

            {/* Garde-fou majoritaire */}
            {activeView === 'gouvernance' && adminActions.length > 0 && (
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
            {activeView === 'gouvernance' && <section>
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
            </section>}

            {/* Thread messages */}
            {activeView === 'messages' && <section>
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                  Fil de discussion
                  {messages.length > 0 && (
                    <span className="ml-1 font-normal text-gray-400 normal-case">
                      ({messages.filter((m) => !m.is_direct).length})
                    </span>
                  )}
                </h3>
                {(outboxCounts[selected.cid_short] ?? 0) > 0 && (
                  <span className="text-xs bg-amber-50 text-amber-700 border border-amber-200 rounded-full px-2.5 py-0.5 flex items-center gap-1">
                    <span>↑</span>
                    {outboxCounts[selected.cid_short]} message{outboxCounts[selected.cid_short] > 1 ? "s" : ""} en attente de synchronisation
                  </span>
                )}
              </div>

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
            </section>}

            {/* ── Activité section ── */}
            {activeView === 'activite' && <section>
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide flex items-center gap-2">
                  Fil d'activité
                  {unreadCount > 0 && (
                    <span className="bg-red-500 text-white text-xs rounded-full px-1.5 py-0.5">{unreadCount} non lu{unreadCount > 1 ? "es" : "e"}</span>
                  )}
                </h3>
                {unreadCount > 0 && (
                  <button
                    onClick={handleMarkAllRead}
                    className="text-xs text-indigo-500 hover:text-indigo-700"
                  >
                    Tout marquer lu
                  </button>
                )}
              </div>

              {activityEvents.length === 0 ? (
                <p className="text-xs text-gray-400 italic">Aucune activité enregistrée.</p>
              ) : (
                <div className="space-y-1.5">
                  {activityEvents.map((ev) => {
                    const notif = notifications.find((n) => n.source_event_id === ev.id);
                    const isUnread = notif && !notif.read;
                    return (
                      <div
                        key={ev.id}
                        className={`flex items-start gap-2 px-2 py-1.5 rounded text-xs ${isUnread ? "bg-indigo-50 border border-indigo-100" : "bg-gray-50"}`}
                        onClick={() => notif && !notif.read && tauriInvoke("notification_mark_read", { notifId: notif.id }).then(() => refreshActivity(selected!.cid_short)).catch(() => {})}
                        style={{ cursor: isUnread ? "pointer" : "default" }}
                      >
                        <span className="text-gray-400 font-mono flex-shrink-0 mt-0.5">{new Date(ev.occurred_at * 1000).toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" })}</span>
                        <span className="text-gray-600 flex-1">{ev.summary}</span>
                        {isUnread && <span className="w-2 h-2 rounded-full bg-red-400 flex-shrink-0 mt-1" />}
                      </div>
                    );
                  })}
                </div>
              )}
            </section>}

            {/* ── Agenda section ── */}
            {activeView === 'agenda' && <section>
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                  Agenda ({agendaEvents.length})
                </h3>
                <button
                  onClick={() => setShowAgendaForm((v) => !v)}
                  className="text-xs text-indigo-500 hover:text-indigo-700"
                >
                  {showAgendaForm ? "Annuler" : "+ Événement"}
                </button>
              </div>

              {showAgendaForm && (
                <div className="mb-4 p-3 bg-gray-50 rounded-lg border border-gray-200 space-y-2">
                  <input
                    className="w-full text-sm border border-gray-200 rounded px-2 py-1"
                    placeholder="Titre *"
                    value={agendaTitle}
                    onChange={(e) => setAgendaTitle(e.target.value)}
                  />
                  <textarea
                    className="w-full text-sm border border-gray-200 rounded px-2 py-1 resize-none"
                    placeholder="Description"
                    rows={2}
                    value={agendaDescription}
                    onChange={(e) => setAgendaDescription(e.target.value)}
                  />
                  <div className="flex gap-2">
                    <div className="flex-1">
                      <label className="text-xs text-gray-500">Début *</label>
                      <input
                        type="datetime-local"
                        className="w-full text-sm border border-gray-200 rounded px-2 py-1"
                        value={agendaStart}
                        onChange={(e) => setAgendaStart(e.target.value)}
                      />
                    </div>
                    <div className="flex-1">
                      <label className="text-xs text-gray-500">Fin</label>
                      <input
                        type="datetime-local"
                        className="w-full text-sm border border-gray-200 rounded px-2 py-1"
                        value={agendaEnd}
                        onChange={(e) => setAgendaEnd(e.target.value)}
                      />
                    </div>
                  </div>
                  <input
                    className="w-full text-sm border border-gray-200 rounded px-2 py-1"
                    placeholder="Lieu (optionnel)"
                    value={agendaLocation}
                    onChange={(e) => setAgendaLocation(e.target.value)}
                  />
                  <button
                    onClick={handleCreateEvent}
                    disabled={creatingEvent || !agendaTitle.trim() || !agendaStart}
                    className="w-full text-sm bg-indigo-500 text-white rounded px-3 py-1.5 hover:bg-indigo-600 disabled:opacity-50"
                  >
                    {creatingEvent ? "Enregistrement…" : "Créer l'événement"}
                  </button>
                </div>
              )}

              {agendaEvents.length === 0 ? (
                <p className="text-xs text-gray-400 italic">Aucun événement.</p>
              ) : (
                <div className="space-y-2">
                  {agendaEvents.map((ev) => (
                    <div
                      key={ev.id}
                      className="flex items-start justify-between gap-2 p-2 bg-gray-50 rounded border border-gray-100"
                    >
                      <div className="min-w-0 flex-1">
                        <div className="text-sm font-medium text-gray-800">{ev.title}</div>
                        <div className="text-xs text-gray-500">
                          {new Date(ev.start_at * 1000).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" })}
                          {ev.end_at && (
                            <> → {new Date(ev.end_at * 1000).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" })}</>
                          )}
                        </div>
                        {ev.location && (
                          <div className="text-xs text-gray-400">{ev.location}</div>
                        )}
                        {ev.description && (
                          <p className="text-xs text-gray-500 mt-0.5 line-clamp-2">{ev.description}</p>
                        )}
                      </div>
                      <button
                        onClick={() => handleDeleteEvent(ev.id)}
                        className="text-xs text-gray-300 hover:text-red-400 transition-colors flex-shrink-0"
                        title="Supprimer"
                      >
                        ✕
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </section>}

            {/* ── Documents section ── */}
            {activeView === 'documents' && <section>
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                  Documents ({documents.length})
                </h3>
                <button
                  onClick={() => setShowDocForm((v) => !v)}
                  className="text-xs text-indigo-500 hover:text-indigo-700"
                >
                  {showDocForm ? "Annuler" : "+ Document"}
                </button>
              </div>

              {showDocForm && (
                <div className="mb-4 p-3 bg-gray-50 rounded-lg border border-gray-200 space-y-2">
                  <input
                    className="w-full text-sm border border-gray-200 rounded px-2 py-1"
                    placeholder="Titre *"
                    value={docTitle}
                    onChange={(e) => setDocTitle(e.target.value)}
                  />
                  <textarea
                    className="w-full text-sm border border-gray-200 rounded px-2 py-1 resize-none"
                    placeholder="Contenu *"
                    rows={5}
                    value={docBody}
                    onChange={(e) => setDocBody(e.target.value)}
                  />
                  <button
                    onClick={handleCreateDocument}
                    disabled={creatingDoc || !docTitle.trim() || !docBody.trim()}
                    className="w-full text-sm bg-indigo-500 text-white rounded px-3 py-1.5 hover:bg-indigo-600 disabled:opacity-50"
                  >
                    {creatingDoc ? "Enregistrement…" : "Créer le document"}
                  </button>
                </div>
              )}

              {documents.length === 0 ? (
                <p className="text-xs text-gray-400 italic">Aucun document.</p>
              ) : (
                <div className="space-y-2">
                  {documents.map((doc) => (
                    <div
                      key={doc.id}
                      className="p-2 bg-gray-50 rounded border border-gray-100"
                    >
                      <div className="flex items-center justify-between gap-2">
                        <button
                          onClick={() => setExpandedDocId(expandedDocId === doc.id ? null : doc.id)}
                          className="text-sm font-medium text-gray-800 text-left hover:text-indigo-600 truncate"
                        >
                          {doc.title}
                        </button>
                        <div className="flex items-center gap-2 flex-shrink-0">
                          <span className="text-xs text-gray-400">v{doc.version}</span>
                          <button
                            onClick={() => handleDeleteDocument(doc.id)}
                            className="text-xs text-gray-300 hover:text-red-400 transition-colors"
                            title="Supprimer"
                          >
                            ✕
                          </button>
                        </div>
                      </div>
                      {expandedDocId === doc.id && (
                        <div className="mt-2">
                          <pre className="text-xs text-gray-700 whitespace-pre-wrap bg-white border border-gray-100 rounded p-2 max-h-48 overflow-y-auto">
                            {doc.body}
                          </pre>
                          <div className="text-xs text-gray-400 mt-1">
                            Par {doc.created_by} · {new Date(doc.created_at * 1000).toLocaleDateString("fr-FR")}
                          </div>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </section>}

            {/* ── RRM section (RRM networks only) ── */}
            {activeView === 'rrm' && selected.is_rrm && (
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
            {activeView === 'annuaire' && !selected.is_directory && !selected.is_rrm && (trustedRrms.length > 0 || showTrustForm) && (
              <section>
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                    Sources de détection RRM approuvées ({trustedRrms.length})
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
                  <div className="bg-orange-50 border border-orange-200 rounded-xl p-4 mb-3 space-y-3">
                    {(() => {
                      const available = networks.filter(
                        (n) => n.is_rrm && !trustedRrms.some((t) => t.rrm_cid_short === n.cid_short)
                      );
                      return available.length > 0 ? (
                        <div className="space-y-1.5">
                          <p className="text-xs text-orange-700 font-medium">Registres connus :</p>
                          {available.map((n) => (
                            <button
                              key={n.cid_short}
                              onClick={() => { setTrustRrmCid(n.cid_short); setTrustRrmName(n.name); }}
                              className={`w-full text-left px-3 py-2 rounded-lg border text-sm transition-colors ${
                                trustRrmCid === n.cid_short
                                  ? "border-orange-500 bg-orange-100"
                                  : "border-orange-200 bg-white hover:bg-orange-50"
                              }`}
                            >
                              <div className="font-medium text-orange-900">{n.name}</div>
                              <div className="text-xs font-mono text-orange-400 mt-0.5">{n.cid_short}</div>
                            </button>
                          ))}
                        </div>
                      ) : null;
                    })()}
                    <details>
                      <summary className="text-xs text-orange-600 cursor-pointer select-none hover:text-orange-800">
                        Saisir manuellement (registre externe…)
                      </summary>
                      <div className="mt-2 space-y-2">
                        <input
                          value={trustRrmCid}
                          onChange={(e) => setTrustRrmCid(e.target.value)}
                          placeholder="CID court (ex : civ1AbCd1234)"
                          className="w-full text-sm border border-orange-200 rounded-lg px-3 py-1.5
                                     focus:outline-none focus:ring-2 focus:ring-orange-400
                                     font-mono placeholder:font-sans placeholder:text-gray-400 bg-white"
                        />
                        <input
                          value={trustRrmName}
                          onChange={(e) => setTrustRrmName(e.target.value)}
                          placeholder="Nom du registre"
                          className="w-full text-sm border border-orange-200 rounded-lg px-3 py-1.5
                                     focus:outline-none focus:ring-2 focus:ring-orange-400
                                     placeholder:text-gray-400 bg-white"
                        />
                      </div>
                    </details>
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
            {activeView === 'annuaire' && !selected.is_directory && !selected.is_rrm && trustedRrms.length === 0 && !showTrustForm && (
              <button
                onClick={() => setShowTrustForm(true)}
                className="text-xs text-gray-400 hover:text-orange-600 transition-colors block"
              >
                + Approuver un registre de surveillance
              </button>
            )}

            {/* ── Annuaire section (directory networks only) ── */}
            {activeView === 'annuaire' && selected.is_directory && (
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
                        {(() => {
                          const available = networks.filter(
                            (n) =>
                              n.is_directory &&
                              n.cid_short !== selected.cid_short &&
                              !federations.some((f) => f.peer_cid_short === n.cid_short)
                          );
                          return available.length > 0 ? (
                            <div className="space-y-1.5">
                              <p className="text-xs text-blue-700 font-medium">Annuaires connus :</p>
                              {available.map((n) => (
                                <button
                                  key={n.cid_short}
                                  onClick={() => { setFedPeerCid(n.cid_short); setFedPeerName(n.name); }}
                                  className={`w-full text-left px-3 py-2 rounded-lg border text-sm transition-colors ${
                                    fedPeerCid === n.cid_short
                                      ? "border-blue-500 bg-blue-100"
                                      : "border-blue-200 bg-white hover:bg-blue-50"
                                  }`}
                                >
                                  <div className="font-medium text-blue-900">{n.name}</div>
                                  <div className="text-xs font-mono text-blue-400 mt-0.5">{n.cid_short}</div>
                                </button>
                              ))}
                            </div>
                          ) : null;
                        })()}
                        <details>
                          <summary className="text-xs text-blue-600 cursor-pointer select-none hover:text-blue-800">
                            Saisir manuellement (annuaire externe…)
                          </summary>
                          <div className="mt-2 space-y-2">
                            <input
                              value={fedPeerCid}
                              onChange={(e) => setFedPeerCid(e.target.value)}
                              placeholder="CID court (ex : civ1AbCd1234)"
                              className="w-full text-sm border border-blue-200 rounded-lg px-3 py-1.5
                                         focus:outline-none focus:ring-2 focus:ring-blue-400
                                         font-mono placeholder:font-sans placeholder:text-gray-400 bg-white"
                            />
                            <input
                              value={fedPeerName}
                              onChange={(e) => setFedPeerName(e.target.value)}
                              placeholder="Nom de l'annuaire"
                              className="w-full text-sm border border-blue-200 rounded-lg px-3 py-1.5
                                         focus:outline-none focus:ring-2 focus:ring-blue-400
                                         placeholder:text-gray-400 bg-white"
                            />
                          </div>
                        </details>
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

            {/* ── Notifications section ── */}
            {activeView === 'notifications' && (
              <section>
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide flex items-center gap-2">
                    Notifications
                    {unreadCount > 0 && (
                      <span className="bg-red-500 text-white text-xs rounded-full px-1.5 py-0.5">{unreadCount} non lue{unreadCount > 1 ? "s" : ""}</span>
                    )}
                  </h3>
                  {unreadCount > 0 && (
                    <button onClick={handleMarkAllRead} className="text-xs text-indigo-500 hover:text-indigo-700">
                      Tout marquer lu
                    </button>
                  )}
                </div>
                {notifications.length === 0 ? (
                  <p className="text-xs text-gray-400 italic">Aucune notification.</p>
                ) : (
                  <div className="space-y-1.5">
                    {notifications.slice(0, 30).map((n) => {
                      const ev = activityEvents.find((a) => a.id === n.source_event_id);
                      const kindIcon: Record<string, string> = {
                        member_joined:    "👤",
                        message_posted:   "💬",
                        proposal_created: "🗳",
                        vote_cast:        "✅",
                        agenda_event_created: "📅",
                        document_created: "📄",
                        connection_requested: "🔗",
                        connection_accepted:  "🤝",
                      };
                      const icon = ev ? (kindIcon[ev.kind] ?? "🔔") : "🔔";
                      const label = ev ? ev.summary : "Événement inconnu";
                      return (
                        <div key={n.id} className={`flex items-start gap-3 px-3 py-2 rounded-xl border ${n.read ? "border-gray-100 bg-white" : "border-indigo-100 bg-indigo-50"}`}>
                          <span className="text-lg mt-0.5 flex-shrink-0">{icon}</span>
                          <div className="flex-1 min-w-0">
                            <p className="text-sm text-gray-800">{label}</p>
                            <p className="text-xs text-gray-400 mt-0.5">{new Date(n.created_at * 1000).toLocaleString("fr-FR")}</p>
                          </div>
                          {!n.read && (
                            <button
                              onClick={() => tauriInvoke("notification_mark_read", { notifId: n.id }).then(() => {
                                setNotifications((prev) => prev.map((x) => x.id === n.id ? { ...x, read: true } : x));
                                setUnreadCount((c) => Math.max(0, c - 1));
                              })}
                              className="text-xs text-indigo-400 hover:text-indigo-600 flex-shrink-0"
                            >
                              Lu
                            </button>
                          )}
                        </div>
                      );
                    })}
                  </div>
                )}
              </section>
            )}

            {/* ── Extensions section ── */}
            {activeView === 'extensions' && (
              <section>
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide mb-3">Extensions (Plugins)</h3>
                <div className="bg-white rounded-2xl shadow-sm border border-gray-100 divide-y divide-gray-100">
                  {plugins.length === 0 && (
                    <p className="px-4 py-6 text-sm text-gray-400 text-center">Aucun plugin installé.</p>
                  )}
                  {plugins.map((p) => (
                    <div key={p.id} className="px-4 py-4 flex items-start gap-4">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 flex-wrap">
                          <span className="text-sm font-semibold text-gray-800">{p.name}</span>
                          <span className="text-xs text-gray-400">v{p.version}</span>
                          {p.is_system && <span className="text-xs px-1.5 py-0.5 bg-gray-100 text-gray-500 rounded">système</span>}
                          <span className={`text-xs px-1.5 py-0.5 rounded ${p.state === "enabled" ? "bg-green-100 text-green-700" : "bg-gray-100 text-gray-500"}`}>
                            {p.state === "enabled" ? "actif" : "inactif"}
                          </span>
                        </div>
                        <p className="text-xs text-gray-500 mt-0.5">{p.description}</p>
                      </div>
                      {!p.is_system && (
                        <button
                          className={`flex-shrink-0 text-xs px-3 py-1.5 rounded-lg border transition-colors disabled:opacity-50 ${
                            p.state === "enabled" ? "border-gray-200 text-gray-600 hover:bg-gray-50" : "border-civium-200 text-civium-700 bg-civium-50 hover:bg-civium-100"
                          }`}
                          disabled={togglingPlugin === p.id}
                          onClick={() => handleTogglePlugin(p.id, p.state)}
                        >
                          {togglingPlugin === p.id ? "…" : p.state === "enabled" ? "Désactiver" : "Activer"}
                        </button>
                      )}
                    </div>
                  ))}
                </div>
              </section>
            )}
          </div>
        )}
      </main>
    </div>
  );
}
