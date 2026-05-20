import { useState, useEffect, useCallback, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { writeFile as fsWriteFile } from "@tauri-apps/plugin-fs";
import { tempDir } from "@tauri-apps/api/path";
import { check as checkUpdate } from "@tauri-apps/plugin-updater";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import ReactMarkdown from "react-markdown";
import rehypeSanitize from "rehype-sanitize";
import { tauriInvoke } from "../tauri";
import type {
  NetworkInfo,
  MemberInfo,
  PendingMemberInfo,
  NodeStatus,
  MessageDisplay,
  MessageListPage,
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
  InvitationInfo,
  ConnectionInfo,
} from "../types";

function formatTime(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString("fr-FR", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function BackupExportWidget({ cidShort, onToast }: { cidShort: string; onToast: (msg: string, kind?: "error" | "ok") => void }) {
  const [pwd, setPwd] = useState("");
  const [pwd2, setPwd2] = useState("");
  const [busy, setBusy] = useState(false);

  async function doExport() {
    if (pwd.length < 8) { onToast("Le mot de passe doit faire au moins 8 caractères."); return; }
    if (pwd !== pwd2) { onToast("Les mots de passe ne correspondent pas."); return; }
    setBusy(true);
    try {
      const b64 = await tauriInvoke<string>("identity_backup_export", { password: pwd });
      const blob = new Blob([b64], { type: "application/octet-stream" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `civium-backup-${cidShort}.civium-backup`;
      a.click();
      URL.revokeObjectURL(url);
      setPwd(""); setPwd2("");
      onToast("Sauvegarde exportée avec succès.", "ok");
    } catch (e) { onToast(String(e)); }
    finally { setBusy(false); }
  }

  return (
    <div className="space-y-2">
      <div className="flex gap-2">
        <input type="password" placeholder="Mot de passe de protection"
          className="flex-1 text-xs border border-gray-200 rounded px-2 py-1.5 focus:outline-none focus:ring-2 focus:ring-civium-400"
          value={pwd} onChange={e => setPwd(e.target.value)} />
        <input type="password" placeholder="Confirmer"
          className="flex-1 text-xs border border-gray-200 rounded px-2 py-1.5 focus:outline-none focus:ring-2 focus:ring-civium-400"
          value={pwd2} onChange={e => setPwd2(e.target.value)} />
      </div>
      <button
        className="text-xs px-3 py-1.5 bg-amber-50 border border-amber-200 text-amber-800 rounded-lg hover:bg-amber-100 transition-colors disabled:opacity-50"
        onClick={doExport} disabled={busy || !pwd || !pwd2}
      >{busy ? "Chiffrement…" : "Télécharger la sauvegarde chiffrée (.civium-backup)"}</button>
      <p className="text-xs text-gray-400">Stockez ce fichier hors de cet appareil. Chiffré avec Argon2id + ChaCha20-Poly1305.</p>
    </div>
  );
}

export default function Dashboard() {
  const [networks, setNetworks] = useState<NetworkInfo[]>([]);
  // Toast notifications
  const [toasts, setToasts] = useState<{ id: number; msg: string; kind: "error" | "ok" }[]>([]);
  const [updateAvailable, setUpdateAvailable] = useState<{ version: string } | null>(null);
  const toastId = useRef(0);
  function showToast(msg: string, kind: "error" | "ok" = "error") {
    const id = ++toastId.current;
    setToasts((t) => [...t, { id, msg, kind }]);
    setTimeout(() => setToasts((t) => t.filter((x) => x.id !== id)), 5000);
  }

  const [selected, setSelected] = useState<NetworkInfo | null>(null);
  const [members, setMembers] = useState<MemberInfo[]>([]);
  const [pending, setPending] = useState<PendingMemberInfo[]>([]);
  const [messages, setMessages] = useState<MessageDisplay[]>([]);
  const [hasMoreMessages, setHasMoreMessages] = useState(false);
  const [oldestRowid, setOldestRowid] = useState<number | null>(null);
  const [loadingOlderMessages, setLoadingOlderMessages] = useState(false);
  const [msgBody, setMsgBody] = useState("");
  const [sending, setSending] = useState(false);
  const [inviteLink, setInviteLink] = useState<string | null>(null);
  const [loadingInvite, setLoadingInvite] = useState(false);
  const [invitations, setInvitations] = useState<InvitationInfo[]>([]);
  const [revokingNonce, setRevokingNonce] = useState<string | null>(null);
  const [mutedMembers, setMutedMembers] = useState<Set<string>>(new Set());
  const [connections, setConnections] = useState<ConnectionInfo[]>([]);
  const [acceptingConn, setAcceptingConn] = useState<string | null>(null);
  const [actingConn, setActingConn] = useState<string | null>(null);
  const [admitting, setAdmitting] = useState<string | null>(null);
  const [admitCircle, setAdmitCircle] = useState<Record<string, number>>({}); // cid → circle
  const [changingCircle, setChangingCircle] = useState<string | null>(null);
  const [deletingMessage, setDeletingMessage] = useState<string | null>(null);
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

  // Delegation (avancé — masqué par défaut)
  const [showDelegationPanel, setShowDelegationPanel] = useState(false);

  // File attachment state
  const [sendingFile, setSendingFile] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [fileDataCache, setFileDataCache] = useState<Record<string, string>>({});
  const [loadingFile, setLoadingFile] = useState<string | null>(null);
  const [showWebcam, setShowWebcam] = useState(false);
  const [webcamStream, setWebcamStream] = useState<MediaStream | null>(null);
  const [capturedPhoto, setCapturedPhoto] = useState<string | null>(null);
  const [webcamMode, setWebcamMode] = useState<"photo" | "video">("photo");
  const [isRecording, setIsRecording] = useState(false);
  const [recordingSeconds, setRecordingSeconds] = useState(0);
  const [recordedBlob, setRecordedBlob] = useState<Blob | null>(null);
  const [recordedUrl, setRecordedUrl] = useState<string | null>(null);
  const webcamVideoRef = useRef<HTMLVideoElement>(null);
  const webcamCanvasRef = useRef<HTMLCanvasElement>(null);
  const webcamStreamRef = useRef<MediaStream | null>(null);
  const webcamWantedRef = useRef(false);
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const recordedChunksRef = useRef<BlobPart[]>([]);
  const recordingTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Lightbox
  const [lightboxSrc, setLightboxSrc] = useState<string | null>(null);

  // Garde-fou state
  const [adminActions, setAdminActions] = useState<AdminActionInfo[]>([]);
  const [contesting, setContesting] = useState<string | null>(null);
  const [now, setNow] = useState(() => Math.floor(Date.now() / 1000));

  // Pagination for long lists (50 items per page)
  const PAGE_SIZE = 50;
  const [membersPage, setMembersPage] = useState(0);
  const [agendaPage, setAgendaPage] = useState(0);
  const [docsPage, setDocsPage] = useState(0);
  const [dirPage, setDirPage] = useState(0);

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
  const [rccRegistering, setRccRegistering] = useState(false);

  // Profile email (contact pour RCC)
  const [profileEmail, setProfileEmail] = useState("");
  const [profileEmailEdit, setProfileEmailEdit] = useState("");
  const [profileEmailEditing, setProfileEmailEditing] = useState(false);
  const [profileEmailSaving, setProfileEmailSaving] = useState(false);

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
  const [lastBackup, setLastBackup] = useState<string | null>(null);
  const [backingUp, setBackingUp] = useState(false);

  // Active view within selected network
  type ActiveView = 'messages' | 'membres' | 'gouvernance' | 'agenda' | 'documents' | 'activite' | 'notifications' | 'annuaire' | 'rrm' | 'extensions' | 'connexions';
  const [activeView, setActiveView] = useState<ActiveView>('messages');

  // Create network form
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [createName, setCreateName] = useState("");
  const [createIsPublic, setCreateIsPublic] = useState(true);
  const [createNetworkType, setCreateNetworkType] = useState<'standard' | 'annuaire' | 'rrm'>('standard');
  const [creating, setCreating] = useState(false);

  // Email invite
  const [inviteEmail, setInviteEmail] = useState("");

  // Join network form
  const [showJoinForm, setShowJoinForm] = useState(false);
  const [joinInviteLink, setJoinInviteLink] = useState("");
  const [joinPeerAddr, setJoinPeerAddr] = useState("");
  const [joinDisplayName, setJoinDisplayName] = useState("");
  const [joining, setJoining] = useState(false);
  const [joinTab, setJoinTab] = useState<"invite" | "directory">("directory");
  const [publicNetworks, setPublicNetworks] = useState<Array<{ network_cid: string; network_name: string; is_main?: boolean }>>([]);
  const [loadingDirectory, setLoadingDirectory] = useState(false);
  const [joiningPublic, setJoiningPublic] = useState<string | null>(null);

  // Fraud alerts
  const [activeAlerts, setActiveAlerts] = useState<FraudAlertInfo[]>([]);
  const [rootConnected, setRootConnected] = useState<string | null>(null);

  // DB error state
  const [dbError, setDbError] = useState<string | null>(null);
  const [dbRestored, setDbRestored] = useState(false);

  // Node watchdog state
  const [nodeCrashed, setNodeCrashed] = useState(false);

  // Backup reminder banner
  const [backupDismissed, setBackupDismissed] = useState<boolean>(
    () => localStorage.getItem("civium.backup_warned") === "1"
  );
  function dismissBackupWarning() {
    localStorage.setItem("civium.backup_warned", "1");
    setBackupDismissed(true);
  }

  // Global activity feed (home screen)
  const [globalFeed, setGlobalFeed] = useState<ActivityEventInfo[]>([]);

  // Clipboard copy feedback: stores the key of the last copied item for 2 s.
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const copyToClipboard = (text: string, key: string) => {
    navigator.clipboard.writeText(text);
    setCopiedKey(key);
    setTimeout(() => setCopiedKey(null), 2000);
  };

  // Keep refs so event listeners always read the latest value.
  const selectedRef = useRef<NetworkInfo | null>(null);
  const networksRef = useRef<NetworkInfo[]>([]);
  useEffect(() => {
    selectedRef.current = selected;
  }, [selected]);
  useEffect(() => {
    networksRef.current = networks;
  }, [networks]);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when messages change.
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Stop webcam whenever the modal closes (or component unmounts)
  useEffect(() => {
    if (showWebcam) return; // only act on close
    const video = webcamVideoRef.current;
    if (video) {
      video.pause();
      video.srcObject = null;
      video.load();
    }
    webcamStreamRef.current?.getTracks().forEach((t) => t.stop());
    webcamStreamRef.current = null;
  }, [showWebcam]);

  // Safety: also stop on unmount
  useEffect(() => {
    return () => {
      webcamStreamRef.current?.getTracks().forEach((t) => t.stop());
    };
  }, []);

  // Auto-load all file messages when messages change
  useEffect(() => {
    if (!selected) return;
    for (const msg of messages) {
      if (msg.is_file && !fileDataCache[msg.id]) {
        handleGetFileData(selected.cid_short, msg.id);
      }
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [messages, selected?.cid_short]);

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

  const refreshGlobalFeed = useCallback(() => {
    tauriInvoke<ActivityEventInfo[]>("activity_list_all").then(setGlobalFeed).catch(() => {});
  }, []);

  useEffect(() => {
    tauriInvoke<NetworkInfo[]>("network_list").then(setNetworks);
    tauriInvoke<PluginInfo[]>("plugin_list").then(setPlugins).catch(() => {});
    tauriInvoke<PairedDeviceInfo[]>("pair_list").then(setPairedDevices).catch(() => {});
    tauriInvoke<{ cid_short: string; cid_full: string; secret_b58: string }>("identity_show")
      .then(setIdentity).catch(() => {});
    tauriInvoke<string>("profile_email_get")
      .then((e) => { setProfileEmail(e); setProfileEmailEdit(e); }).catch(() => {});
    refreshOutboxCounts();
    refreshRccStatuses();
    refreshGlobalFeed();
  }, [refreshOutboxCounts, refreshRccStatuses, refreshGlobalFeed]);

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
    tauriInvoke<MessageListPage>("message_list_paged", { networkCid: cid, limit: 50 }).then((page) => {
      setMessages(page.messages);
      setHasMoreMessages(page.has_more);
      setOldestRowid(page.oldest_rowid);
    });
  }, []);

  const loadOlderMessages = useCallback(async (cid: string) => {
    if (!oldestRowid || loadingOlderMessages) return;
    setLoadingOlderMessages(true);
    try {
      const page = await tauriInvoke<MessageListPage>("message_list_paged", {
        networkCid: cid, limit: 50, beforeRowid: oldestRowid,
      });
      setMessages((prev) => [...page.messages, ...prev]);
      setHasMoreMessages(page.has_more);
      setOldestRowid(page.oldest_rowid);
    } finally {
      setLoadingOlderMessages(false);
    }
  }, [oldestRowid, loadingOlderMessages]);

  const refreshProposals = useCallback((cid: string) => {
    tauriInvoke<ProposalInfo[]>("proposal_list", { networkCid: cid }).then((props) => {
      setProposals(props);
      props.forEach((p) => {
        tauriInvoke<VoteResultInfo>("vote_results", { networkCid: cid, proposalId: p.id })
          .then((r) => setVoteResults((prev) => ({ ...prev, [p.id]: r })))
          .catch(() => {});
      });
    });
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
    // Load essentials: network info, members, and messages (default tab).
    refreshNetwork(selected.cid_short);
    refreshMessages(selected.cid_short);
    // Other tabs load lazily via the activeView effect below.
    // Charger la config hub pour ce réseau
    tauriInvoke<{ hub_url: string; enabled: boolean; last_sync_ts: number } | null>("hub_config_get", {
      networkCid: selected.cid_short,
    }).then((cfg) => {
      setHubConfig(cfg);
      setHubUrl(cfg?.hub_url ?? "");
      setHubMsg(null);
    }).catch(() => {});
    setInviteLink(null);
    setInvitations([]);
    tauriInvoke<InvitationInfo[]>("invitation_list", { networkCid: selected.cid_short })
      .then(setInvitations).catch(() => {});
    tauriInvoke<string[]>("member_muted_list", { networkCid: selected.cid_short })
      .then((cids) => setMutedMembers(new Set(cids))).catch(() => {});
    setMessages([]);
    setHasMoreMessages(false);
    setOldestRowid(null);
    setProposals([]);
    setVoteResults({});
    setShowProposalForm(false);
    setAdminActions([]);
    setMyDelegations([]);
    setDirEntries([]);
    setDirSearchResults(null);
    setMembersPage(0);
    setAgendaPage(0);
    setDocsPage(0);
    setDirPage(0);
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

  // Lazy-load tab-specific data when the active view changes.
  useEffect(() => {
    if (!selected) return;
    const cid = selected.cid_short;
    switch (activeView) {
      case 'gouvernance':
        refreshProposals(cid);
        refreshAdminActions(cid);
        refreshDelegations(cid);
        break;
      case 'agenda':
        refreshAgendaEvents(cid);
        break;
      case 'documents':
        refreshDocuments(cid);
        break;
      case 'activite':
      case 'notifications':
        refreshActivity(cid);
        break;
      case 'annuaire':
        refreshDirEntries(cid);
        refreshFederations(cid);
        break;
      case 'rrm':
        refreshRrmEntries(cid);
        refreshTrustedRrms(cid);
        break;
      case 'connexions':
        tauriInvoke<ConnectionInfo[]>("connection_list", { networkCid: cid })
          .then(setConnections).catch(() => {});
        break;
      case 'extensions':
        refreshAp(cid);
        break;
    }
  }, [selected?.cid_short, activeView,
    refreshProposals, refreshAdminActions, refreshDelegations,
    refreshAgendaEvents, refreshDocuments, refreshActivity,
    refreshDirEntries, refreshFederations,
    refreshRrmEntries, refreshTrustedRrms, refreshAp]);

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
          if (mounted) {
            setNodeStatus(s);
            // Clear the crash banner once the node is back online.
            if (s.running) setNodeCrashed(false);
          }
        })
        .catch(() => {});
    };
    pollStatus();
    const interval = setInterval(pollStatus, 5000);

    // Load MCP status once on mount
    tauriInvoke<McpStatus>("mcp_status").then(setMcpStatus).catch(() => {});

    // Check for updates once on mount
    checkUpdate().then((update) => {
      if (update?.available && mounted) setUpdateAvailable({ version: update.version });
    }).catch(() => {});

    tauriInvoke<FraudAlertInfo[]>("get_active_alerts").then((a) => {
      if (mounted) setActiveAlerts(a);
    }).catch(() => {});

    // Poll hub for fraud alerts immediately then every 30 min
    const pollAlerts = () => {
      tauriInvoke<FraudAlertInfo[]>("poll_hub_alerts").catch(() => {});
    };
    pollAlerts();
    const alertInterval = setInterval(pollAlerts, 30 * 60 * 1000);

    let unlistenSync: UnlistenFn | null = null;
    let unlistenHubSync: UnlistenFn | null = null;
    let unlistenOutbox: UnlistenFn | null = null;
    let unlistenRcc: UnlistenFn | null = null;
    let unlistenAlert: UnlistenFn | null = null;

    listen<string>("civium://sync-completed", async (event) => {
      const cid = event.payload;
      tauriInvoke<NetworkInfo[]>("network_list").then((nets) => {
        if (mounted) setNetworks(nets);
      });
      if (selectedRef.current?.cid_short === cid) {
        refreshNetwork(cid);
        refreshMessages(cid);
      } else {
        // Notify user of new messages in a background network
        let perm = await isPermissionGranted();
        if (!perm) { const res = await requestPermission(); perm = res === "granted"; }
        if (perm) {
          const net = networksRef.current?.find((n) => n.cid_short === cid);
          if (net) sendNotification({ title: "Nouveau message", body: `Nouveau(x) message(s) dans « ${net.name} »` });
        }
      }
      refreshOutboxCounts();
      refreshGlobalFeed();
    }).then((fn) => {
      unlistenSync = fn;
    });

    listen<string>("civium://hub-sync-completed", (event) => {
      const cid = event.payload;
      if (selectedRef.current?.cid_short === cid) {
        refreshMessages(cid);
      }
      refreshGlobalFeed();
    }).then((fn) => {
      unlistenHubSync = fn;
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

    listen<FraudAlertInfo>("civium://fraud-alert", async (event) => {
      if (mounted) {
        setActiveAlerts((prev) => [...prev, event.payload]);
        let perm = await isPermissionGranted();
        if (!perm) { const res = await requestPermission(); perm = res === "granted"; }
        if (perm) sendNotification({ title: "Alerte Civium", body: event.payload.description });
      }
    }).then((fn) => {
      unlistenAlert = fn;
    });

    let unlistenRoot: UnlistenFn | null = null;
    listen<string>("civium://root-connected", (event) => {
      if (mounted) setRootConnected(event.payload);
    }).then((fn) => {
      unlistenRoot = fn;
    });

    let unlistenDbError: UnlistenFn | null = null;
    listen<string>("civium://db-error", (event) => {
      if (mounted) setDbError(event.payload);
    }).then((fn) => {
      unlistenDbError = fn;
    });

    let unlistenNodeCrashed: UnlistenFn | null = null;
    listen("civium://node-crashed", () => {
      if (mounted) setNodeCrashed(true);
    }).then((fn) => {
      unlistenNodeCrashed = fn;
    });

    let unlistenDbRestored: UnlistenFn | null = null;
    listen("civium://db-restored", () => {
      if (mounted) setDbRestored(true);
    }).then((fn) => {
      unlistenDbRestored = fn;
    });

    // Deep link handler: civium://join/<b58> ou civium://pair/<b58>
    function handleDeepLinkEvent(e: Event) {
      if (!mounted) return;
      const { action, param } = (e as CustomEvent<{ action: string; param: string }>).detail;
      if (action === "join" && param) {
        setJoinInviteLink(param);
        setShowJoinForm(true);
      } else if (action === "pair" && param) {
        setPairLink(param);
        setShowPairCompleteForm(true);
      }
    }
    window.addEventListener("civium:deep-link", handleDeepLinkEvent);

    return () => {
      mounted = false;
      clearInterval(interval);
      clearInterval(alertInterval);
      unlistenSync?.();
      unlistenHubSync?.();
      unlistenOutbox?.();
      unlistenRcc?.();
      unlistenAlert?.();
      unlistenRoot?.();
      unlistenDbError?.();
      unlistenDbRestored?.();
      unlistenNodeCrashed?.();
      window.removeEventListener("civium:deep-link", handleDeepLinkEvent);
    };
  }, [refreshNetwork, refreshMessages, refreshOutboxCounts, refreshRccStatuses, refreshGlobalFeed]);

  async function generateInvite() {
    if (!selected) return;
    setLoadingInvite(true);
    try {
      const link = await tauriInvoke<string>("network_invite", {
        networkCid: selected.cid_short,
        expiresIn: 0,
      });
      setInviteLink(link);
      // Recharger la liste des invitations
      tauriInvoke<InvitationInfo[]>("invitation_list", { networkCid: selected.cid_short })
        .then(setInvitations).catch(() => {});
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
      showToast(String(e));
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
      showToast(String(e));
    }
  }

  async function handleChangeCircle(memberCid: string, circle: number) {
    if (!selected) return;
    setChangingCircle(memberCid);
    try {
      await tauriInvoke("member_change_circle", {
        networkCid: selected.cid_short,
        memberCid,
        circle,
      });
      refreshNetwork(selected.cid_short);
    } catch (e) {
      showToast(String(e));
    } finally {
      setChangingCircle(null);
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      if (opts.length < 2) { showToast("Au moins 2 options requises."); return; }
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      setDirPage(0);
    } catch (e) {
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
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
      showToast(String(e));
    }
  }

  async function handleMcpStart() {
    try {
      const port = parseInt(mcpPort, 10) || 7523;
      const status = await tauriInvoke<McpStatus>("mcp_start", { port });
      setMcpStatus(status);
      setShowMcpToken(true);
    } catch (e) {
      showToast(String(e));
    }
  }

  async function handleMcpStop() {
    try {
      await tauriInvoke("mcp_stop");
      setMcpStatus({ running: false, port: null, token: null, url: null });
      setShowMcpToken(false);
    } catch (e) {
      showToast(String(e));
    }
  }

  async function handlePairInit() {
    if (!pairLabel.trim()) return;
    try {
      const session = await tauriInvoke<PairingInitInfo>("pair_init", { label: pairLabel.trim() });
      setPairingSession(session);
      setPairLabel("");
    } catch (e) {
      showToast(String(e));
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
      showToast(String(e));
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
      showToast("Paramètres enregistrés. Redémarrez l'application pour les appliquer.", "ok");
    } catch (e) {
      showToast(String(e));
    } finally {
      setNodeSaving(false);
    }
  }

  async function handleRccRegister() {
    if (!selected || !profileEmail.trim()) return;
    setRccRegistering(true);
    try {
      const info = await tauriInvoke<RccStatusInfo>("rcc_register", {
        networkCid: selected.cid_short,
        adminEmail: profileEmail.trim(),
      });
      setRccStatuses((prev) => ({ ...prev, [info.network_cid_short]: info }));
    } catch (e) {
      showToast(String(e));
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
      showToast(String(e));
    } finally {
      setRccRegistering(false);
    }
  }

  async function handlePairRevoke(deviceId: string) {
    try {
      await tauriInvoke("pair_revoke", { deviceId });
      refreshPairedDevices();
    } catch (e) {
      showToast(String(e));
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
      showToast(String(e));
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

  async function handleFileSelect(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file || !selected) return;
    // Audio/video always go through the temp-file path to avoid large IPC payloads.
    // Other files use IPC only if ≤ 500 KB.
    const isMedia = file.type.startsWith("video/") || file.type.startsWith("audio/");
    const MAX_IPC = 524_288;      // 500 Ko — seuil IPC pour les non-média
    const MAX_TOTAL = 524_288_000; // 500 Mo — limite absolue
    if (file.size > MAX_TOTAL) {
      showToast("Fichier trop volumineux (max 500 Mo)");
      return;
    }
    setSendingFile(true);
    try {
      let msg: MessageDisplay;
      if (!isMedia && file.size <= MAX_IPC) {
        // Petits fichiers (≤ 5 Mo) : envoi direct en base64 via IPC
        const buffer = await file.arrayBuffer();
        const bytes = new Uint8Array(buffer);
        // Chunked encoding to avoid slow character-by-character concatenation
        const CHUNK = 0x8000;
        let binary = "";
        for (let i = 0; i < bytes.length; i += CHUNK) {
          binary += String.fromCharCode(...bytes.subarray(i, i + CHUNK));
        }
        msg = await tauriInvoke<MessageDisplay>("message_send_file", {
          networkCid: selected.cid_short,
          filename: file.name,
          mimeType: file.type || "application/octet-stream",
          dataBase64: btoa(binary),
        });
      } else {
        // Gros fichiers (> 5 Mo) : écriture sur disque, envoi via chemin
        const buffer = await file.arrayBuffer();
        const bytes = new Uint8Array(buffer);
        const tmpDir = await tempDir();
        const tmpPath = `${tmpDir}civium_upload_${Date.now()}_${file.name}`;
        await fsWriteFile(tmpPath, bytes);
        msg = await tauriInvoke<MessageDisplay>("message_send_file_path", {
          networkCid: selected.cid_short,
          tempPath: tmpPath,
          filename: file.name,
          mimeType: file.type || "application/octet-stream",
        });
      }
      setMessages((prev) => [...prev, msg]);
      refreshActivity(selected.cid_short);
    } catch (err) {
      showToast("Erreur lors de l'envoi du fichier : " + String(err));
    } finally {
      setSendingFile(false);
      if (fileInputRef.current) fileInputRef.current.value = "";
    }
  }

  async function handleGetFileData(networkCid: string, messageId: string): Promise<void> {
    if (fileDataCache[messageId] || loadingFile === messageId) return;
    setLoadingFile(messageId);
    try {
      const res = await tauriInvoke<{ filename: string; mime_type: string; data_b64: string }>(
        "message_get_file", { networkCid, messageId }
      );
      setFileDataCache((prev) => ({ ...prev, [messageId]: res.data_b64 }));
    } catch (e) {
      showToast("Erreur chargement fichier : " + String(e));
    } finally {
      setLoadingFile(null);
    }
  }

  function handleDownloadFile(msg: MessageDisplay) {
    const data = fileDataCache[msg.id];
    if (!data) { return; }
    const binary = atob(data);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    const blob = new Blob([bytes], { type: msg.mime_type || "application/octet-stream" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url; a.download = msg.filename || "fichier";
    document.body.appendChild(a); a.click();
    document.body.removeChild(a); URL.revokeObjectURL(url);
  }

  async function openWebcam() {
    webcamWantedRef.current = true;
    setShowWebcam(true);
    setCapturedPhoto(null);
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ video: true });
      // Race: user closed modal while getUserMedia was pending → stop immediately
      if (!webcamWantedRef.current) {
        stream.getTracks().forEach((t) => t.stop());
        return;
      }
      webcamStreamRef.current = stream;
      setWebcamStream(stream);
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          // Guard: user may have closed the modal between getUserMedia and this rAF
          if (!webcamWantedRef.current) {
            stream.getTracks().forEach((t) => t.stop());
            webcamStreamRef.current = null;
            return;
          }
          if (webcamVideoRef.current) {
            webcamVideoRef.current.srcObject = stream;
            webcamVideoRef.current.play().catch(() => {});
          }
        });
      });
    } catch {
      webcamWantedRef.current = false;
      setShowWebcam(false);
      showToast("Impossible d'accéder à la caméra. Vérifiez les permissions.");
    }
  }

  function stopRecording() {
    if (recordingTimerRef.current) {
      clearInterval(recordingTimerRef.current);
      recordingTimerRef.current = null;
    }
    if (mediaRecorderRef.current && mediaRecorderRef.current.state !== "inactive") {
      mediaRecorderRef.current.stop();
    }
    setIsRecording(false);
  }

  function closeWebcam() {
    webcamWantedRef.current = false;
    stopRecording();
    mediaRecorderRef.current = null;
    recordedChunksRef.current = [];
    const stream = webcamStreamRef.current;
    if (stream) {
      stream.getTracks().forEach((t) => t.stop());
      webcamStreamRef.current = null;
    }
    const video = webcamVideoRef.current;
    if (video) {
      video.pause();
      video.srcObject = null;
    }
    if (recordedUrl) { URL.revokeObjectURL(recordedUrl); }
    setShowWebcam(false);
    setWebcamStream(null);
    setCapturedPhoto(null);
    setRecordedBlob(null);
    setRecordedUrl(null);
    setRecordingSeconds(0);
  }

  function capturePhoto() {
    if (!webcamVideoRef.current || !webcamCanvasRef.current) return;
    const video = webcamVideoRef.current;
    const canvas = webcamCanvasRef.current;
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    canvas.getContext("2d")?.drawImage(video, 0, 0);
    webcamWantedRef.current = false;
    video.srcObject = null;
    webcamStreamRef.current?.getTracks().forEach((t) => t.stop());
    webcamStreamRef.current = null;
    setWebcamStream(null);
    setCapturedPhoto(canvas.toDataURL("image/jpeg", 0.9));
  }

  async function sendCapturedPhoto() {
    if (!capturedPhoto || !selected) return;
    const base64 = capturedPhoto.split(",")[1];
    setSendingFile(true);
    try {
      const msg = await tauriInvoke<MessageDisplay>("message_send_file", {
        networkCid: selected.cid_short,
        filename: `photo_${Date.now()}.jpg`,
        mimeType: "image/jpeg",
        dataBase64: base64,
      });
      setMessages((prev) => [...prev, msg]);
      closeWebcam();
      refreshActivity(selected.cid_short);
      refreshGlobalFeed();
    } catch (e) {
      showToast("Erreur envoi photo : " + String(e));
    } finally {
      setSendingFile(false);
    }
  }

  function startRecording() {
    const stream = webcamStreamRef.current;
    if (!stream) return;
    recordedChunksRef.current = [];
    const mimeType = MediaRecorder.isTypeSupported("video/webm;codecs=vp9")
      ? "video/webm;codecs=vp9"
      : "video/webm";
    const recorder = new MediaRecorder(stream, { mimeType });
    mediaRecorderRef.current = recorder;
    recorder.ondataavailable = (e) => {
      if (e.data.size > 0) recordedChunksRef.current.push(e.data);
    };
    recorder.onstop = () => {
      const blob = new Blob(recordedChunksRef.current, { type: "video/webm" });
      const url = URL.createObjectURL(blob);
      // Stop camera tracks first
      webcamWantedRef.current = false;
      webcamStreamRef.current?.getTracks().forEach((t) => t.stop());
      webcamStreamRef.current = null;
      if (webcamVideoRef.current) {
        webcamVideoRef.current.srcObject = null;
      }
      setWebcamStream(null);
      // Let React re-render with recordedBlob/recordedUrl — the <video src={recordedUrl}> handles playback
      setRecordedBlob(blob);
      setRecordedUrl(url);
    };
    recorder.start(200); // collect data every 200ms
    setIsRecording(true);
    setRecordingSeconds(0);
    recordingTimerRef.current = setInterval(() => {
      setRecordingSeconds((s) => s + 1);
    }, 1000);
  }

  async function sendRecordedVideo() {
    if (!recordedBlob || !selected) return;
    const MAX_TOTAL = 524_288_000; // 500 Mo
    const MAX_IPC   = 5_242_880;  // 5 Mo — above this use the temp-file path
    if (recordedBlob.size > MAX_TOTAL) {
      showToast("Vidéo trop volumineuse (max 500 Mo).");
      return;
    }
    setSendingFile(true);
    try {
      const filename = `video_${Date.now()}.webm`;
      let msg: MessageDisplay;
      if (recordedBlob.size <= MAX_IPC) {
        // Small recording: encode to base64 in-process (native FileReader)
        const base64 = await new Promise<string>((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => resolve((reader.result as string).split(",")[1]);
          reader.onerror = reject;
          reader.readAsDataURL(recordedBlob!);
        });
        msg = await tauriInvoke<MessageDisplay>("message_send_file", {
          networkCid: selected.cid_short,
          filename,
          mimeType: "video/webm",
          dataBase64: base64,
        });
      } else {
        // Large recording: write to disk, let Rust read it (avoids large IPC payload)
        const bytes = new Uint8Array(await recordedBlob.arrayBuffer());
        const tmpDir = await tempDir();
        const tmpPath = `${tmpDir}civium_upload_${Date.now()}_${filename}`;
        await fsWriteFile(tmpPath, bytes);
        msg = await tauriInvoke<MessageDisplay>("message_send_file_path", {
          networkCid: selected.cid_short,
          tempPath: tmpPath,
          filename,
          mimeType: "video/webm",
        });
      }
      setMessages((prev) => [...prev, msg]);
      closeWebcam();
      refreshActivity(selected.cid_short);
      refreshGlobalFeed();
    } catch (e) {
      showToast("Erreur envoi vidéo : " + String(e));
    } finally {
      setSendingFile(false);
    }
  }

  async function loadPublicNetworks() {
    setLoadingDirectory(true);
    try {
      const nets = await tauriInvoke<Array<{ network_cid: string; network_name: string; is_main?: boolean }>>("hub_public_networks");
      setPublicNetworks(nets);
    } catch {
      setPublicNetworks([]);
    } finally {
      setLoadingDirectory(false);
    }
  }

  async function handleJoinPublicNetwork(networkCid: string, networkName: string) {
    setJoiningPublic(networkCid);
    try {
      const net = await tauriInvoke<NetworkInfo>("hub_join_public_network", { networkCid, networkName });
      const nets = await tauriInvoke<NetworkInfo[]>("network_list");
      setNetworks(nets);
      const joined = nets.find(n => n.cid_short === net.cid_short);
      if (joined) { selectedRef.current = joined; setSelected(joined); }
      setShowJoinForm(false);
      setActiveView("messages");
    } catch (e) {
      showToast(String(e));
    } finally {
      setJoiningPublic(null);
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
      showToast(String(e));
    } finally {
      setJoining(false);
    }
  }

  async function handleCreateNetwork() {
    if (!createName.trim()) return;
    setCreating(true);
    try {
      const name = createName.trim();
      let net: NetworkInfo;
      if (createNetworkType === 'annuaire') {
        net = await tauriInvoke<NetworkInfo>("directory_create", { name, displayName: name });
      } else if (createNetworkType === 'rrm') {
        net = await tauriInvoke<NetworkInfo>("rrm_create", { name, displayName: name });
      } else {
        net = await tauriInvoke<NetworkInfo>("network_create", { name, displayName: name, privacy: !createIsPublic });
      }
      const createdCid = net.cid_short;
      // Auto-register to RCC avec l'email de profil (réseaux standard uniquement)
      if (createNetworkType === 'standard' && profileEmail.trim()) {
        tauriInvoke("rcc_register", {
          networkCid: createdCid,
          adminEmail: profileEmail.trim(),
        }).catch(() => {});
      }
      const nets = await tauriInvoke<NetworkInfo[]>("network_list");
      setNetworks(nets);
      const created = nets.find((n) => n.cid_short === createdCid) ?? nets.find((n) => n.name === name);
      if (created) {
        selectedRef.current = created;
        setSelected(created);
        setActiveView(createNetworkType === 'annuaire' ? 'annuaire' : createNetworkType === 'rrm' ? 'rrm' : 'messages');
      }
      setShowCreateForm(false);
      setCreateName("");
      setCreateNetworkType('standard');
    } catch (e) {
      showToast(String(e));
    } finally {
      setCreating(false);
    }
  }

  const circleLabel = (c: number) =>
    ["Annuaire", "Connaissance", "Confiance", "Intime"][c] ?? `Cercle ${c}`;

  const enabledPluginIds = plugins.filter((p) => p.state === "enabled").map((p) => p.id);
  const hasAgenda = enabledPluginIds.some((id) => id.includes("agenda"));
  const hasDocuments = enabledPluginIds.some((id) => id.includes("document"));

  const globalNavItem = (view: ActiveView, icon: string, label: string, badge?: number) => {
    const isActive = activeView === view && !showSettings && !showCreateForm && !showJoinForm;
    return (
      <button
        key={view}
        role="tab"
        aria-selected={isActive}
        aria-controls={`panel-${view}`}
        onClick={() => { setActiveView(view); setShowSettings(false); setShowCreateForm(false); setShowJoinForm(false); }}
        className={`w-full text-left px-3 py-1.5 rounded-lg text-xs transition-colors flex items-center gap-2 ${
          isActive ? "bg-civium-500 text-white" : "text-civium-200 hover:bg-civium-700"
        }`}
      >
        <span aria-hidden="true">{icon}</span>
        <span className="flex-1">{label}</span>
        {badge !== undefined && badge > 0 && (
          <span
            aria-label={`${badge} non lus`}
            className="text-xs bg-red-500 text-white rounded-full px-1.5 py-0.5 min-w-[1.2rem] text-center"
          >{badge}</span>
        )}
      </button>
    );
  };

  // DB corruption screen — shown before everything else
  if (dbError) {
    return (
      <main className="min-h-screen flex items-center justify-center bg-red-50 p-6" role="alert" aria-live="assertive">
        <div className="bg-white rounded-2xl shadow-lg max-w-lg w-full p-8 space-y-6">
          <div className="text-center">
            <div className="text-5xl mb-4">⚠️</div>
            <h1 className="text-2xl font-bold text-red-700">Base de données corrompue</h1>
            {dbRestored ? (
              <p className="text-sm text-green-700 mt-2 font-medium">
                Une sauvegarde a été restaurée automatiquement. Veuillez relancer l'application.
              </p>
            ) : (
              <p className="text-sm text-gray-600 mt-2">
                Civium n'a pas pu ouvrir la base de données locale et aucune sauvegarde n'a pu être restaurée automatiquement.
              </p>
            )}
          </div>
          <div className="bg-red-50 border border-red-200 rounded-lg p-4">
            <p className="text-xs font-mono text-red-700 break-all">{dbError}</p>
          </div>
          <div className="space-y-3 text-sm text-gray-600">
            <p className="font-semibold">Options :</p>
            <ul className="list-disc pl-5 space-y-1">
              <li>Relancez l'application — si une sauvegarde a été restaurée, elle sera utilisée au prochain démarrage.</li>
              <li>Copiez manuellement un fichier <code className="font-mono text-xs bg-gray-100 px-1 rounded">civium-*.db</code> depuis le dossier <code className="font-mono text-xs bg-gray-100 px-1 rounded">.backups/</code> vers <code className="font-mono text-xs bg-gray-100 px-1 rounded">civium.db</code>.</li>
              <li>Si aucune sauvegarde n'existe, vous devrez réinitialiser l'application (perte de données). Supprimez <code className="font-mono text-xs bg-gray-100 px-1 rounded">civium.db</code> pour redémarrer depuis zéro.</li>
            </ul>
          </div>
          <button
            onClick={() => setDbError(null)}
            className="w-full py-3 bg-gray-200 hover:bg-gray-300 rounded-xl text-sm font-medium transition-colors"
          >
            Réessayer (ignorer et continuer)
          </button>
        </div>
      </main>
    );
  }

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Toast notifications */}
      <div
        aria-live="polite"
        aria-atomic="false"
        className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm"
      >
        {toasts.map((t) => (
          <div
            key={t.id}
            role="alert"
            className={`flex items-start gap-3 px-4 py-3 rounded-xl shadow-lg text-sm
              ${t.kind === "error"
                ? "bg-red-700 text-white"
                : "bg-green-600 text-white"
              }`}
          >
            <span className="flex-1">{t.msg}</span>
            <button
              onClick={() => setToasts((prev) => prev.filter((x) => x.id !== t.id))}
              aria-label="Fermer"
              className="shrink-0 opacity-70 hover:opacity-100 transition-opacity"
            >✕</button>
          </div>
        ))}
      </div>

      {/* Sidebar */}
      <aside aria-label="Navigation" className="w-64 bg-civium-900 text-white flex flex-col">
        {/* Mon nœud */}
        <div className="px-4 py-3 border-b border-civium-700">
          <p className="text-xs font-semibold text-civium-400 uppercase tracking-wider mb-2">Mon nœud</p>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span
                aria-label={nodeStatus.running ? "Nœud en ligne" : "Nœud hors ligne"}
                className={`w-2 h-2 rounded-full flex-shrink-0 ${nodeStatus.running ? "bg-green-400" : "bg-gray-500"}`}
              />
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
                if (next) {
                  tauriInvoke<string | null>("db_backup_last").then((ts) => setLastBackup(ts));
                }
              }}
              className={`text-xs px-2 py-1 rounded-lg transition-colors ${
                showSettings ? "bg-civium-600 text-white" : "text-gray-400 hover:bg-civium-700"
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
              className={`text-xs px-2 py-1 rounded-lg transition-colors ${
                showJoinForm ? "bg-civium-600 text-white" : "text-gray-400 hover:bg-civium-700"
              }`}
              title="Rejoindre un réseau existant"
            >
              ←
            </button>
            {networks.length === 0 && (
              <button
                onClick={() => { setShowCreateForm((v) => !v); setShowJoinForm(false); setShowSettings(false); setActiveView('messages'); }}
                className={`text-xs px-2 py-1 rounded-lg transition-colors ${showCreateForm ? "bg-civium-600 text-white" : "text-gray-400 hover:bg-civium-700"}`}
                title="Créer votre réseau"
              >
                +
              </button>
            )}
          </div>
        </div>

        {/* Network list + plugin sub-nav */}
        <nav aria-label="Mes réseaux" className="flex-1 overflow-y-auto px-3 py-2 space-y-1">
          {networks.length === 0 && !showCreateForm && (
            <p className="text-xs text-civium-100 px-2 py-2">Aucun réseau. Cliquez sur + pour en créer un.</p>
          )}
          {networks.filter(n => !n.parent_cid).map((net) => {
            const isSelected = selected?.cid_short === net.cid_short;
            const children = networks.filter(n => n.parent_cid === net.cid_full);
            return (
              <div key={net.cid_short}>
                <button
                  onClick={() => {
                    selectedRef.current = net;
                    setSelected(net);
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
                {/* Sous-réseaux */}
                {children.length > 0 && (
                  <div className="ml-3 pl-2 border-l border-civium-700 mt-0.5 space-y-0.5">
                    {children.map((child) => {
                      const isChildSelected = selected?.cid_short === child.cid_short;
                      return (
                        <button
                          key={child.cid_short}
                          onClick={() => {
                            selectedRef.current = child;
                            setSelected(child);
                            setShowSettings(false);
                            setShowCreateForm(false);
                            setShowJoinForm(false);
                          }}
                          className={`w-full text-left px-2 py-1.5 rounded-lg text-xs transition-colors ${
                            isChildSelected && !showSettings && !showCreateForm
                              ? "bg-civium-700 text-white"
                              : "text-civium-200 hover:bg-civium-700"
                          }`}
                        >
                          <span className="opacity-50 mr-1">↳</span>
                          <span className="font-medium">{child.name}</span>
                        </button>
                      );
                    })}
                  </div>
                )}
              </div>
            );
          })}
        </nav>

        {/* Navigation globale */}
        <div role="tablist" aria-label="Sections du réseau" className="px-3 py-2 border-t border-civium-700 space-y-0.5">
          {selected && (
            <p className="text-xs text-civium-500 px-1 pb-1 truncate" title={selected.name}>
              ◈ {selected.name}
            </p>
          )}
          {globalNavItem('messages', '💬', 'Messages', unreadCount > 0 ? unreadCount : undefined)}
          {globalNavItem('membres', '👥', 'Membres')}
          {globalNavItem('gouvernance', '🗳', 'Gouvernance')}
          {hasAgenda && globalNavItem('agenda', '📅', 'Agenda')}
          {hasDocuments && globalNavItem('documents', '📄', 'Documents')}
          {globalNavItem('activite', '📊', 'Activité')}
          {globalNavItem('notifications', '🔔', 'Notifications', unreadCount > 0 ? unreadCount : undefined)}
          {selected?.is_directory && globalNavItem('annuaire', '🔍', 'Annuaire')}
          {selected?.is_rrm && globalNavItem('rrm', '🚫', 'Réseaux signalés')}
          {globalNavItem('connexions', '🔗', 'Connexions')}
          {globalNavItem('extensions', '🧩', 'Extensions')}
        </div>

        {/* Statut réseau */}
        <div className="px-4 py-2 border-t border-civium-700">
          <span className="text-xs text-civium-500">{nodeStatus.running ? "Connecté" : "Hors ligne"}</span>
        </div>
      </aside>

      {/* Main */}
      <main id={`panel-${activeView}`} role="tabpanel" aria-label={activeView} className="flex-1 overflow-y-auto">
        {/* Bannière connexion réseau racine */}
        {rootConnected && (
          <div className="bg-civium-600 text-white px-6 py-2 flex items-center gap-2 text-sm">
            <span className="font-semibold">Connecté au réseau Civium</span>
            <span className="text-civium-200">— votre réseau ({rootConnected}) est maintenant dans l'annuaire public.</span>
            <button
              aria-label="Fermer"
              onClick={() => setRootConnected(null)}
              className="ml-auto text-civium-200 hover:text-white transition-colors text-xs"
            >
              ✕
            </button>
          </div>
        )}

        {/* Email manquant — requis pour le RCC */}
        {!profileEmail && (
          <div className="bg-amber-50 border-b border-amber-200 px-6 py-2 flex items-center justify-between text-sm">
            <span className="text-amber-800">Renseignez votre email de contact pour enregistrer vos réseaux au RCC (registre légal obligatoire).</span>
            <button
              className="ml-4 text-xs underline text-amber-700 hover:text-amber-900 shrink-0"
              onClick={() => setShowSettings(true)}
            >
              Paramètres
            </button>
          </div>
        )}

        {/* Node crash banner */}
        {nodeCrashed && !nodeStatus.running && (
          <div role="alert" className="bg-orange-600 text-white px-6 py-2 flex items-center justify-between text-sm">
            <span>Le nœud P2P s'est arrêté — redémarrage automatique en cours…</span>
            <button
              aria-label="Fermer"
              onClick={() => setNodeCrashed(false)}
              className="text-orange-200 hover:text-white transition-colors"
            >
              ✕
            </button>
          </div>
        )}

        {/* Fraud alert banners */}
        {updateAvailable && (
          <div className="bg-civium-600 text-white px-6 py-2 flex items-center justify-between text-sm">
            <span>Nouvelle version disponible : <strong>{updateAvailable.version}</strong></span>
            <button
              aria-label="Reporter la mise à jour"
              onClick={() => setUpdateAvailable(null)}
              className="text-civium-200 hover:text-white transition-colors"
            >
              ✕ Plus tard
            </button>
          </div>
        )}

        {activeAlerts.length > 0 && (
          <div className="bg-red-700 text-white px-6 py-2 space-y-1.5">
            {activeAlerts.map((al, i) => (
              <div key={i} className="flex items-start gap-2 text-sm">
                <span className="font-bold uppercase shrink-0">[{al.alert_type}]</span>
                <span className="flex-1">{al.description}</span>
                {al.network_cids.length > 0 && (
                  <span className="text-red-200 text-xs shrink-0">
                    Réseaux : {al.network_cids.join(", ")}
                  </span>
                )}
                <button
                  onClick={() => {
                    tauriInvoke("alert_dismiss", { alertType: al.alert_type, emittedAt: al.emitted_at }).catch(() => {});
                    setActiveAlerts((prev) => prev.filter((_, j) => j !== i));
                  }}
                  title="Masquer cette alerte définitivement"
                  className="shrink-0 text-red-200 hover:text-white transition-colors leading-none"
                >
                  ✕
                </button>
              </div>
            ))}
          </div>
        )}

        {!backupDismissed && (
          <div className="bg-amber-50 border-b border-amber-200 px-6 py-3 flex items-start gap-3">
            <span className="text-amber-600 text-lg leading-none shrink-0">⚠</span>
            <div className="flex-1 text-sm text-amber-900">
              <span className="font-semibold">Sauvegardez votre identité.</span>{" "}
              Sans fichier de sauvegarde, la perte de cet appareil entraîne la perte définitive de votre identité Civium.{" "}
              <button
                className="underline hover:no-underline"
                onClick={() => {
                  setShowSettings(true);
                  setSelected(null);
                }}
              >
                Créer une sauvegarde chiffrée →
              </button>
            </div>
            <button
              onClick={dismissBackupWarning}
              title="Ne plus afficher"
              className="shrink-0 text-amber-500 hover:text-amber-800 transition-colors text-xs"
            >
              ✕ Compris
            </button>
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

              {/* Backup warning */}
              <div className="flex items-start gap-3 bg-amber-50 border border-amber-200 rounded-xl px-4 py-3 mb-3">
                <span className="text-amber-500 text-lg shrink-0">⚠️</span>
                <div className="text-xs text-amber-800">
                  <p className="font-semibold mb-0.5">Sauvegardez votre clé secrète</p>
                  <p>Sans sauvegarde, la perte de cet appareil entraîne la perte définitive de votre identité Civium. Exportez votre clé dans un fichier et conservez-le en lieu sûr.</p>
                </div>
              </div>

              <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-5 space-y-4">
                {identity ? (
                  <>
                    <div>
                      <p className="text-xs text-gray-500 mb-1">CID complet</p>
                      <div className="flex items-center gap-2">
                        <code className="flex-1 text-xs font-mono bg-gray-50 border border-gray-200 rounded px-2 py-1.5 break-all">{identity.cid_full}</code>
                        <button
                          className="text-xs text-civium-600 hover:text-civium-800 border border-civium-200 rounded px-2 py-1 shrink-0 transition-colors"
                          onClick={() => copyToClipboard(identity!.cid_full, "cid")}
                        >{copiedKey === "cid" ? "✓ Copié" : "Copier"}</button>
                      </div>
                    </div>
                    {/* Email de contact */}
                    <div>
                      <p className="text-xs text-gray-500 mb-1">Email de contact <span className="text-gray-400">(pour les alertes RCC)</span></p>
                      {profileEmailEditing ? (
                        <div className="flex items-center gap-2">
                          <input
                            type="email"
                            className="flex-1 text-sm border border-gray-200 rounded px-2 py-1.5 focus:outline-none focus:ring-2 focus:ring-civium-400"
                            value={profileEmailEdit}
                            onChange={(e) => setProfileEmailEdit(e.target.value)}
                            onKeyDown={async (e) => {
                              if (e.key === "Enter") {
                                setProfileEmailSaving(true);
                                try {
                                  await tauriInvoke("profile_email_set", { email: profileEmailEdit.trim() });
                                  setProfileEmail(profileEmailEdit.trim());
                                  setProfileEmailEditing(false);
                                } catch (err) { showToast(String(err)); }
                                finally { setProfileEmailSaving(false); }
                              }
                              if (e.key === "Escape") { setProfileEmailEdit(profileEmail); setProfileEmailEditing(false); }
                            }}
                            autoFocus
                          />
                          <button
                            className="text-xs bg-civium-600 text-white border border-civium-600 rounded px-2 py-1 shrink-0 disabled:opacity-50"
                            disabled={profileEmailSaving || !profileEmailEdit.trim()}
                            onClick={async () => {
                              setProfileEmailSaving(true);
                              try {
                                await tauriInvoke("profile_email_set", { email: profileEmailEdit.trim() });
                                setProfileEmail(profileEmailEdit.trim());
                                setProfileEmailEditing(false);
                              } catch (err) { showToast(String(err)); }
                              finally { setProfileEmailSaving(false); }
                            }}
                          >{profileEmailSaving ? "…" : "Enregistrer"}</button>
                          <button className="text-xs border border-gray-200 rounded px-2 py-1 shrink-0 hover:bg-gray-50"
                            onClick={() => { setProfileEmailEdit(profileEmail); setProfileEmailEditing(false); }}>Annuler</button>
                        </div>
                      ) : (
                        <div className="flex items-center gap-2">
                          {profileEmail ? (
                            <span className="flex-1 text-sm text-gray-700 bg-gray-50 border border-gray-200 rounded px-2 py-1.5">{profileEmail}</span>
                          ) : (
                            <span className="flex-1 text-sm text-red-400 bg-red-50 border border-red-200 rounded px-2 py-1.5">Non renseigné — requis pour le RCC</span>
                          )}
                          <button
                            className="text-xs border border-gray-200 rounded px-2 py-1 shrink-0 hover:bg-gray-50"
                            onClick={() => { setProfileEmailEdit(profileEmail); setProfileEmailEditing(true); }}
                          >{profileEmail ? "Modifier" : "Renseigner"}</button>
                        </div>
                      )}
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
                            className="text-xs text-civium-600 hover:text-civium-800 border border-civium-200 rounded px-2 py-1 shrink-0 transition-colors"
                            onClick={() => copyToClipboard(identity!.secret_b58, "secret")}
                          >{copiedKey === "secret" ? "✓ Copié" : "Copier"}</button>
                        )}
                      </div>
                    </div>
                    {/* Export backup chiffré */}
                    <div className="pt-2 border-t border-gray-100">
                      <p className="text-xs text-gray-500 mb-2">Exporter la clé en fichier de sauvegarde chiffré</p>
                      <BackupExportWidget cidShort={identity!.cid_short} onToast={showToast} />
                    </div>
                    {/* Export all data */}
                    <div className="pt-2 border-t border-gray-100">
                      <p className="text-xs text-gray-500 mb-2">Exporter toutes mes données (messages, réseaux, etc.)</p>
                      <button
                        className="text-xs px-3 py-1.5 bg-gray-50 border border-gray-200 text-gray-700
                                   rounded-lg hover:bg-gray-100 transition-colors"
                        onClick={async () => {
                          try {
                            const json = await tauriInvoke<string>("export_data");
                            const blob = new Blob([json], { type: "application/json" });
                            const url = URL.createObjectURL(blob);
                            const a = document.createElement("a");
                            a.href = url;
                            a.download = `civium-export-${identity?.cid_short ?? "data"}.json`;
                            a.click();
                            URL.revokeObjectURL(url);
                          } catch (e) { showToast(String(e)); }
                        }}
                      >
                        Télécharger mes données (.json)
                      </button>
                    </div>
                    {/* Download logs */}
                    <div className="pt-2 border-t border-gray-100">
                      <p className="text-xs text-gray-500 mb-2">Journaux applicatifs (utile pour les rapports de bug)</p>
                      <button
                        className="text-xs px-3 py-1.5 bg-gray-50 border border-gray-200 text-gray-700
                                   rounded-lg hover:bg-gray-100 transition-colors"
                        onClick={async () => {
                          try {
                            const content = await tauriInvoke<string>("logs_get");
                            if (!content) { showToast("Aucun journal disponible pour aujourd'hui.", "ok"); return; }
                            const blob = new Blob([content], { type: "text/plain" });
                            const url = URL.createObjectURL(blob);
                            const a = document.createElement("a");
                            a.href = url;
                            const date = new Date().toISOString().slice(0, 10);
                            a.download = `civium-${date}.log`;
                            a.click();
                            URL.revokeObjectURL(url);
                          } catch (e) { showToast(String(e)); }
                        }}
                      >
                        Télécharger les journaux (.log)
                      </button>
                    </div>
                  </>
                ) : (
                  <p className="text-xs text-gray-400">Chargement…</p>
                )}
              </div>
            </section>

            {/* ── Sauvegarde automatique ── */}
            <section>
              <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-1">Sauvegarde de la base de données</h3>
              <p className="text-xs text-gray-400 mb-3">
                Une copie de <code>civium.db</code> est créée au démarrage et toutes les 6 heures. Les 7 dernières sauvegardes sont conservées.
              </p>
              <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-4 flex items-center justify-between gap-4">
                <div>
                  <p className="text-xs text-gray-500">Dernière sauvegarde :</p>
                  <p className="text-sm font-medium text-gray-800 mt-0.5">
                    {lastBackup
                      ? new Date(lastBackup).toLocaleString("fr-FR")
                      : "Aucune sauvegarde disponible"}
                  </p>
                </div>
                <button
                  onClick={async () => {
                    setBackingUp(true);
                    try {
                      await tauriInvoke("db_backup_now");
                      const ts = await tauriInvoke<string | null>("db_backup_last");
                      setLastBackup(ts);
                      showToast("Sauvegarde créée avec succès.", "ok");
                    } catch (e) { showToast(String(e)); }
                    finally { setBackingUp(false); }
                  }}
                  disabled={backingUp}
                  className="text-xs px-3 py-1.5 bg-civium-600 text-white rounded-lg hover:bg-civium-700 disabled:opacity-50 transition-colors shrink-0"
                >
                  {backingUp ? "Sauvegarde…" : "Sauvegarder maintenant"}
                </button>
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
                        <div key={a} className="text-xs font-mono bg-gray-50 border border-gray-200 rounded px-2 py-1 cursor-pointer hover:bg-gray-100 truncate transition-colors" onClick={() => copyToClipboard(a, `addr-${a}`)} title="Cliquer pour copier">
                          {copiedKey === `addr-${a}` ? "✓ Copié !" : a}
                        </div>
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
                            } catch (e) { showToast(String(e)); }
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
                    return profileEmail.trim() ? (
                      <div className="space-y-2">
                        <p className="text-xs text-gray-500">Ce réseau n'est pas encore déclaré. La déclaration utilisera votre email de contact : <span className="font-medium text-gray-700">{profileEmail}</span></p>
                        <button className="text-sm bg-civium-600 text-white px-3 py-1.5 rounded hover:bg-civium-700 disabled:opacity-50" disabled={rccRegistering} onClick={handleRccRegister}>
                          {rccRegistering ? "Déclaration…" : "Déclarer ce réseau"}
                        </button>
                      </div>
                    ) : (
                      <p className="text-xs text-gray-500">
                        Renseignez votre email de contact dans <button className="underline text-civium-600 hover:text-civium-800" onClick={() => setShowSettings(true)}>Paramètres → Identité</button> pour pouvoir déclarer ce réseau.
                      </p>
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
                        <div className="bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-xs font-mono break-all text-gray-700 cursor-pointer hover:bg-gray-100 transition-colors" onClick={() => copyToClipboard(selected.ap_actor_url!, "ap_url")} title="Cliquer pour copier">
                          {copiedKey === "ap_url" ? <span className="font-sans text-green-600">✓ Copié !</span> : <>{selected.ap_actor_url} <span className="text-gray-400 font-sans">(copier)</span></>}
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

            {/* ── Zone de danger ── */}
            <section className="border border-red-200 rounded-xl p-4 space-y-3">
              <h3 className="text-sm font-semibold text-red-600 uppercase tracking-wide">Zone de danger</h3>

              {/* Leave network */}
              {selected && (
                <div className="flex items-center justify-between gap-4 py-2 border-b border-red-100">
                  <div>
                    <p className="text-sm font-medium text-gray-800">Quitter « {selected.name} »</p>
                    <p className="text-xs text-gray-400 mt-0.5">Vous quittez ce réseau. Vos messages restent visibles pour les autres membres.</p>
                  </div>
                  <button
                    onClick={async () => {
                      if (!confirm(`Quitter le réseau « ${selected.name} » ?`)) return;
                      try {
                        await tauriInvoke("network_leave", { networkCid: selected.cid_short });
                        const nets = await tauriInvoke<NetworkInfo[]>("network_list");
                        setNetworks(nets);
                        setSelected(nets[0] ?? null);
                        setShowSettings(false);
                      } catch (e) { showToast(String(e)); }
                    }}
                    className="text-xs border border-red-300 text-red-600 rounded-lg px-3 py-1.5 hover:bg-red-50 transition-colors shrink-0"
                  >
                    Quitter
                  </button>
                </div>
              )}

              {/* Delete network (admin only, when empty) */}
              {selected && selected.member_count <= 1 && (
                <div className="flex items-center justify-between gap-4 py-2 border-b border-red-100">
                  <div>
                    <p className="text-sm font-medium text-gray-800">Supprimer « {selected.name} »</p>
                    <p className="text-xs text-gray-400 mt-0.5">Supprime le réseau et toutes ses données locales. Irréversible.</p>
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
                      } catch (e) { showToast(String(e)); }
                    }}
                    className="text-xs border border-red-300 text-red-600 rounded-lg px-3 py-1.5 hover:bg-red-50 transition-colors shrink-0"
                  >
                    Supprimer
                  </button>
                </div>
              )}

              {/* Wipe all data */}
              <div className="flex items-center justify-between gap-4 py-2">
                <div>
                  <p className="text-sm font-medium text-gray-800">Effacer toutes mes données</p>
                  <p className="text-xs text-gray-400 mt-0.5">Supprime votre identité, tous les réseaux, messages, et clés de cet appareil. Irréversible sans sauvegarde.</p>
                </div>
                <button
                  onClick={async () => {
                    if (!confirm("Effacer TOUTES vos données Civium ? Votre identité et tous vos réseaux seront supprimés. Cette action est irréversible sans fichier de sauvegarde.")) return;
                    if (!confirm("Dernière confirmation : supprimer définitivement toutes vos données ?")) return;
                    try {
                      await tauriInvoke("wipe_all_data");
                      window.location.reload();
                    } catch (e) { showToast(String(e)); }
                  }}
                  className="text-xs border border-red-500 bg-red-50 text-red-700 rounded-lg px-3 py-1.5 hover:bg-red-100 transition-colors shrink-0 font-semibold"
                >
                  Tout effacer
                </button>
              </div>
            </section>
          </div>

        ) : showJoinForm ? (
          /* ══ PANNEAU REJOINDRE UN RÉSEAU ══ */
          <div className="max-w-lg mx-auto py-12 px-6">
            <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-8 space-y-5">
              <div className="flex items-center justify-between">
                <h2 className="text-xl font-bold text-gray-900">Rejoindre un réseau</h2>
                <button
                  className="text-gray-400 hover:text-gray-600 text-sm"
                  onClick={() => { setShowJoinForm(false); setJoinInviteLink(""); setJoinPeerAddr(""); setJoinDisplayName(""); }}
                >✕</button>
              </div>

              {/* Onglets */}
              <div className="flex gap-1 bg-gray-100 rounded-lg p-1">
                <button
                  className={`flex-1 py-1.5 rounded-md text-sm font-medium transition-colors ${joinTab === "directory" ? "bg-white text-gray-900 shadow-sm" : "text-gray-500 hover:text-gray-700"}`}
                  onClick={() => { setJoinTab("directory"); if (publicNetworks.length === 0) loadPublicNetworks(); }}
                >
                  Annuaire Civium
                </button>
                <button
                  className={`flex-1 py-1.5 rounded-md text-sm font-medium transition-colors ${joinTab === "invite" ? "bg-white text-gray-900 shadow-sm" : "text-gray-500 hover:text-gray-700"}`}
                  onClick={() => setJoinTab("invite")}
                >
                  Lien d'invitation
                </button>
              </div>

              {/* Onglet Annuaire */}
              {joinTab === "directory" && (
                <div className="space-y-3">
                  <p className="text-sm text-gray-500">
                    Réseaux publics hébergés sur le serveur Civium. Rejoignez-en un directement, sans invitation.
                  </p>
                  {loadingDirectory ? (
                    <p className="text-sm text-gray-400 text-center py-4">Chargement…</p>
                  ) : publicNetworks.length === 0 ? (
                    <div className="text-center py-4 space-y-2">
                      <p className="text-sm text-gray-400">Aucun réseau public disponible.</p>
                      <button
                        className="text-xs text-civium-600 hover:underline"
                        onClick={loadPublicNetworks}
                      >Réessayer</button>
                    </div>
                  ) : (
                    <div className="space-y-2">
                      {publicNetworks.map((net) => {
                        const alreadyJoined = networks.some(n => n.cid_full === net.network_cid || n.cid_short === net.network_cid.slice(0, 12));
                        return (
                          <div key={net.network_cid} className="flex items-center gap-3 bg-gray-50 border border-gray-200 rounded-xl px-4 py-3">
                            <div className="flex-1 min-w-0">
                              <div className="text-sm font-semibold text-gray-800 flex items-center gap-1.5">
                                {net.network_name}
                                {net.is_main && <span className="text-xs bg-civium-100 text-civium-700 px-1.5 py-0.5 rounded-full">Principal</span>}
                                <span className="text-xs bg-green-100 text-green-700 px-1.5 py-0.5 rounded-full">🌐 Public</span>
                              </div>
                              <div className="text-xs text-gray-400 font-mono truncate">{net.network_cid.slice(0, 20)}…</div>
                            </div>
                            {alreadyJoined ? (
                              <span className="text-xs text-green-600 font-medium shrink-0">Déjà membre</span>
                            ) : (
                              <button
                                className="shrink-0 text-xs px-3 py-1.5 bg-civium-600 text-white rounded-lg hover:bg-civium-700 disabled:opacity-50 transition-colors"
                                disabled={joiningPublic === net.network_cid}
                                onClick={() => handleJoinPublicNetwork(net.network_cid, net.network_name)}
                              >
                                {joiningPublic === net.network_cid ? "…" : "Rejoindre"}
                              </button>
                            )}
                          </div>
                        );
                      })}
                      <button className="text-xs text-gray-400 hover:text-gray-600 w-full text-center pt-1" onClick={loadPublicNetworks}>
                        Actualiser la liste
                      </button>
                    </div>
                  )}
                </div>
              )}

              {/* Onglet Invitation */}
              {joinTab === "invite" && (
                <div className="space-y-4">
                  <p className="text-sm text-gray-500">
                    Vous avez reçu un lien d'invitation d'un administrateur. Collez-le ci-dessous.
                  </p>
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
                      Adresse P2P de l'admin
                      <span className="ml-1 text-xs font-normal text-gray-400">(optionnel)</span>
                    </label>
                    <input
                      type="text"
                      className="w-full border border-gray-200 rounded-lg px-3 py-2 text-xs font-mono focus:outline-none focus:ring-2 focus:ring-civium-400 placeholder:font-sans placeholder:text-gray-400"
                      placeholder="/ip4/1.2.3.4/tcp/4001/p2p/12D3…"
                      value={joinPeerAddr}
                      onChange={(e) => setJoinPeerAddr(e.target.value)}
                    />
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
                    {joining ? (joinPeerAddr.trim() ? "Connexion…" : "Jonction…") : "Rejoindre le réseau"}
                  </button>
                  {joining && joinPeerAddr.trim() && (
                    <p className="text-xs text-gray-400 text-center">
                      Connexion au nœud de l'admin… (jusqu'à 30 secondes)
                    </p>
                  )}
                </div>
              )}
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
                {/* Type de réseau */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">Type de réseau</label>
                  <div className="grid grid-cols-3 gap-2">
                    {([
                      { id: 'standard', icon: '👥', label: 'Réseau', desc: 'Groupe de personnes : famille, équipe, association…' },
                      { id: 'annuaire', icon: '🔍', label: 'Annuaire', desc: 'Répertoire public de réseaux, membres ou services.' },
                      { id: 'rrm',      icon: '🚫', label: 'RRM (Registre des Réseaux Malveillants)', desc: 'Liste de réseaux signalés comme malveillants.' },
                    ] as const).map(({ id, icon, label, desc }) => (
                      <button
                        key={id}
                        type="button"
                        onClick={() => setCreateNetworkType(id)}
                        title={desc}
                        className={`py-2 px-2 rounded-lg border text-xs font-medium transition-colors text-center ${
                          createNetworkType === id
                            ? "bg-civium-50 border-civium-400 text-civium-700"
                            : "bg-white border-gray-200 text-gray-500 hover:bg-gray-50"
                        }`}
                      >
                        <span className="block text-lg">{icon}</span>
                        {label}
                      </button>
                    ))}
                  </div>
                  <p className="text-xs text-gray-400 mt-1">
                    {createNetworkType === 'annuaire'
                      ? "Un annuaire (Annuaire de réseau Civium) permet de référencer et de rendre découvrables des réseaux, membres ou services."
                      : createNetworkType === 'rrm'
                      ? "Un RRM (Registre des Réseaux Malveillants) permet de signaler des réseaux au comportement prouvé malveillant. Utilisé par les autres réseaux pour filtrer."
                      : "Un réseau standard est un espace de groupe privé ou public : famille, équipe, association…"}
                  </p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Nom du réseau</label>
                  <input
                    type="text"
                    autoFocus
                    className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-civium-400"
                    placeholder={createNetworkType === 'annuaire' ? "Ex : Annuaire des associations de Lyon…" : createNetworkType === 'rrm' ? "Ex : RRM Global Civium…" : "Ex : Famille Martin, Équipe projet…"}
                    value={createName}
                    onChange={(e) => setCreateName(e.target.value)}
                    onKeyDown={(e) => { if (e.key === "Enter") handleCreateNetwork(); }}
                  />
                </div>
                {createNetworkType === 'standard' && (
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">Visibilité</label>
                    <div className="flex gap-3">
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
                    </div>
                    <p className="text-xs text-gray-400 mt-1">
                      {createIsPublic
                        ? "Votre réseau sera visible dans les annuaires Civium. Tout le monde peut demander à le rejoindre."
                        : "Votre réseau est sur invitation uniquement. Seules les personnes que vous invitez peuvent le rejoindre."}
                    </p>
                  </div>
                )}
                <button
                  className="w-full py-2.5 bg-civium-600 text-white rounded-xl font-semibold text-sm hover:bg-civium-700 disabled:opacity-50 transition-colors"
                  disabled={creating || !createName.trim()}
                  onClick={handleCreateNetwork}
                >
                  {creating ? "Création en cours…" : `Créer ${createNetworkType === 'annuaire' ? "l'annuaire" : createNetworkType === 'rrm' ? 'le RRM' : 'le réseau'}`}
                </button>
              </div>
            </div>
          </div>
        ) : (
          <div className="max-w-2xl mx-auto py-8 px-6 space-y-6">
            {/* Welcome / select prompt (non-Extensions views only) */}
            {activeView !== 'extensions' && !selected && networks.length === 0 && (
              <div className="flex flex-col items-center justify-center h-80 text-center">
                <p className="text-gray-600 font-medium mb-2">Bienvenue sur Civium</p>
                <p className="text-sm text-gray-400 leading-relaxed max-w-sm mb-5">
                  Votre identité ({identity?.cid_short ?? "…"}) est distincte de vos réseaux.
                  Un réseau est un espace de groupe que vous créez ou rejoignez.
                </p>
                <div className="flex flex-col gap-2 w-full max-w-xs">
                  <button
                    className="text-sm px-4 py-2 bg-civium-600 text-white rounded-lg hover:bg-civium-700 transition-colors"
                    onClick={() => setShowCreateForm(true)}
                  >
                    + Créer un nouveau réseau
                  </button>
                  <button
                    className="text-sm px-4 py-2 bg-white border border-gray-200 text-gray-600 rounded-lg hover:bg-gray-50 transition-colors"
                    onClick={() => { setShowJoinForm(true); setJoinTab("directory"); loadPublicNetworks(); }}
                  >
                    Rejoindre un réseau existant
                  </button>
                </div>
              </div>
            )}

            {/* Global activity feed — shown when networks exist but none selected */}
            {activeView !== 'extensions' && !selected && networks.length > 0 && (
              <div className="space-y-4">
                {/* Header */}
                <div className="bg-gradient-to-br from-civium-50 to-indigo-50 border border-civium-100 rounded-2xl px-6 py-5">
                  <h2 className="text-xl font-bold text-civium-700 mb-0.5">Fil d'actualité</h2>
                  <p className="text-sm text-civium-500">
                    {networks.length} réseau{networks.length > 1 ? "x" : ""} — sélectionnez-en un dans la barre latérale
                  </p>
                </div>

                {globalFeed.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-16 text-gray-400">
                    <span className="text-4xl mb-3">📭</span>
                    <p className="text-sm font-medium">Aucune activité récente.</p>
                    <p className="text-xs mt-1 text-gray-300">Les événements de vos réseaux s'afficheront ici.</p>
                  </div>
                ) : (
                  <ul className="space-y-2">
                    {globalFeed.map((e) => {
                      const netName = networks.find((n) => n.cid_short === e.network_cid_short)?.name ?? e.network_cid_short;
                      return (
                        <li key={e.id} className="flex items-start gap-3 bg-white border border-gray-100 rounded-xl px-4 py-3 text-sm shadow-sm hover:shadow-md transition-shadow">
                          <span className="w-8 h-8 rounded-full bg-civium-100 flex-shrink-0 flex items-center justify-center text-civium-700 text-xs font-bold mt-0.5">
                            {netName[0]?.toUpperCase() ?? "?"}
                          </span>
                          <div className="flex-1 min-w-0">
                            <span className="text-xs font-medium text-civium-600">{netName}</span>
                            <p className="text-gray-700 mt-0.5 leading-snug">{e.summary}</p>
                          </div>
                          <span className="text-gray-300 text-xs shrink-0 mt-0.5">
                            {new Date(e.occurred_at * 1000).toLocaleString('fr-FR', { day:'2-digit', month:'2-digit', hour:'2-digit', minute:'2-digit' })}
                          </span>
                        </li>
                      );
                    })}
                  </ul>
                )}
              </div>
            )}

            {/* Header: réseau sélectionné */}
            {activeView !== 'extensions' && selected && (
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
            )}

            {/* Pending members — always visible as notification */}
            {selected && activeView === 'membres' && pending.length > 0 && (
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
                      <div className="flex items-center gap-2">
                        <select
                          value={admitCircle[p.cid_short] ?? 1}
                          onChange={(e) => setAdmitCircle((prev) => ({ ...prev, [p.cid_short]: Number(e.target.value) }))}
                          className="text-xs border border-gray-200 rounded px-1.5 py-1 focus:outline-none"
                          title="Cercle de confiance"
                        >
                          <option value={0}>0 — Annuaire</option>
                          <option value={1}>1 — Connaissance</option>
                          <option value={2}>2 — Confiance</option>
                          <option value={3}>3 — Intime</option>
                        </select>
                        <button
                          onClick={() => admitMember(p.cid_short, admitCircle[p.cid_short] ?? 1)}
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
            {selected && activeView === 'membres' && <section>
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
                {members.slice(membersPage * PAGE_SIZE, (membersPage + 1) * PAGE_SIZE).map((m) => (
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
                        {mutedMembers.has(m.cid_short) && (
                          <span className="text-xs px-2 py-0.5 bg-orange-100 text-orange-700 rounded-full" title="Membre en sourdine — ses messages sont masqués">
                            sourdine
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
                          {/* Circle selector */}
                          {iAmAdmin && (
                            <div className="flex items-center gap-2 pt-2">
                              <span className="text-xs text-gray-500">Cercle :</span>
                              <select
                                value={m.circle}
                                onChange={(e) => { handleChangeCircle(m.cid_short, Number(e.target.value)); }}
                                disabled={changingCircle === m.cid_short}
                                className="text-xs border border-gray-200 rounded px-1.5 py-0.5 focus:outline-none disabled:opacity-50"
                                onClick={(e) => e.stopPropagation()}
                              >
                                <option value={0}>0 — Annuaire</option>
                                <option value={1}>1 — Connaissance</option>
                                <option value={2}>2 — Confiance</option>
                                <option value={3}>3 — Intime</option>
                              </select>
                              {changingCircle === m.cid_short && <span className="text-xs text-gray-400">…</span>}
                            </div>
                          )}
                          <div className="flex flex-wrap items-center gap-2 pt-2">
                            <span className="text-xs text-gray-500">Admin :</span>
                            {/* Mute / unmute — local only, visible to self only */}
                            {!isMe && (
                              <button
                                className={`text-xs px-2 py-0.5 rounded-full border transition-colors ${
                                  mutedMembers.has(m.cid_short)
                                    ? "bg-orange-50 border-orange-300 text-orange-700 hover:bg-orange-100"
                                    : "bg-gray-50 border-gray-300 text-gray-600 hover:bg-gray-100"
                                }`}
                                onClick={async (e) => {
                                  e.stopPropagation();
                                  const isMuted = mutedMembers.has(m.cid_short);
                                  try {
                                    if (isMuted) {
                                      await tauriInvoke("member_unmute", { networkCid: selected!.cid_short, memberCid: m.cid_short });
                                      setMutedMembers((prev) => { const s = new Set(prev); s.delete(m.cid_short); return s; });
                                    } else {
                                      await tauriInvoke("member_mute", { networkCid: selected!.cid_short, memberCid: m.cid_short });
                                      setMutedMembers((prev) => new Set([...prev, m.cid_short]));
                                    }
                                  } catch (err) { showToast(String(err)); }
                                }}
                              >
                                {mutedMembers.has(m.cid_short) ? "🔕 Rétablir" : "🔔 Mettre en sourdine"}
                              </button>
                            )}
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
              {members.length > PAGE_SIZE && (
                <div className="flex items-center justify-between mt-2 text-xs text-gray-500">
                  <button
                    disabled={membersPage === 0}
                    onClick={() => setMembersPage((p) => p - 1)}
                    className="px-3 py-1 rounded border border-gray-200 bg-white hover:bg-gray-50 disabled:opacity-40 transition-colors"
                  >← Précédent</button>
                  <span>Page {membersPage + 1} / {Math.ceil(members.length / PAGE_SIZE)}</span>
                  <button
                    disabled={(membersPage + 1) * PAGE_SIZE >= members.length}
                    onClick={() => setMembersPage((p) => p + 1)}
                    className="px-3 py-1 rounded border border-gray-200 bg-white hover:bg-gray-50 disabled:opacity-40 transition-colors"
                  >Suivant →</button>
                </div>
              )}
            </section>}

            {/* Invite link */}
            {selected && activeView === 'membres' && inviteLink && (
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
                        onClick={() => copyToClipboard(inviteLink, "invite")}
                      >
                        {copiedKey === "invite" ? "✓ Copié !" : "Copier"}
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
                              onClick={() => copyToClipboard(addr, `peer-${addr}`)}
                            >
                              {copiedKey === `peer-${addr}` ? "✓ Copié !" : "Copier"}
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
                          const networkName = selected?.name ?? "Civium";
                          const networkCid  = selected?.cid_full ?? "";
                          const webLink = networkCid
                            ? `https://www.rouaix.com/civium/app?join=${encodeURIComponent(networkCid)}&jname=${encodeURIComponent(networkName)}`
                            : "";
                          const body = [
                            `Bonjour,`,
                            ``,
                            `Je vous invite à rejoindre mon réseau "${networkName}" sur Civium.`,
                            ``,
                            `— Option A : depuis votre navigateur (sans installation)`,
                            webLink ? `  ${webLink}` : `  (CID du réseau : ${networkCid})`,
                            ``,
                            `— Option B : depuis l'application desktop`,
                            `  1. Téléchargez l'application Civium : https://civium.app`,
                            `  2. Au démarrage, choisissez "Rejoindre un réseau"`,
                            `  3. Collez ce lien d'invitation :`,
                            `     ${inviteLink}`,
                            `  4. Collez mon adresse de connexion :`,
                            addrBlock,
                            `  5. Choisissez votre nom et confirmez`,
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

            {/* Liste des invitations */}
            {selected && activeView === 'membres' && invitations.length > 0 && (
              <section>
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide mb-2">
                  Invitations ({invitations.filter((i) => !i.revoked && !i.is_expired).length} actives)
                </h3>
                <div className="bg-white border border-gray-200 rounded-xl divide-y divide-gray-100">
                  {invitations.map((inv) => {
                    const active = !inv.revoked && !inv.is_expired;
                    return (
                      <div key={inv.nonce_b58} className="flex items-center gap-3 px-4 py-3">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-0.5">
                            <span className={`text-xs px-1.5 py-0.5 rounded-full font-medium ${
                              inv.revoked ? "bg-red-100 text-red-600" :
                              inv.is_expired ? "bg-gray-100 text-gray-500" :
                              "bg-green-100 text-green-700"
                            }`}>
                              {inv.revoked ? "Révoquée" : inv.is_expired ? "Expirée" : "Active"}
                            </span>
                            <span className="text-xs text-gray-400 font-mono truncate">
                              …{inv.nonce_b58.slice(-8)}
                            </span>
                          </div>
                          <div className="text-xs text-gray-500">
                            Créée le {new Date(inv.created_at * 1000).toLocaleDateString("fr-FR")}
                            {inv.expires_at > 0 && ` · Expire le ${new Date(inv.expires_at * 1000).toLocaleDateString("fr-FR")}`}
                          </div>
                        </div>
                        <div className="flex items-center gap-2 shrink-0">
                          {active && (
                            <button
                              className="text-xs px-2 py-1 border border-gray-200 rounded hover:bg-gray-50 transition-colors"
                              onClick={() => copyToClipboard(inv.link, `inv-${inv.nonce_b58}`)}
                              title="Copier le lien"
                            >
                              {copiedKey === `inv-${inv.nonce_b58}` ? "✓ Copié !" : "Copier"}
                            </button>
                          )}
                          {active && (
                            <button
                              className="text-xs px-2 py-1 border border-red-200 text-red-600 rounded hover:bg-red-50 disabled:opacity-50 transition-colors"
                              disabled={revokingNonce === inv.nonce_b58}
                              onClick={async () => {
                                if (!confirm("Révoquer ce lien d'invitation ? Les personnes qui l'ont déjà reçu ne pourront plus l'utiliser.")) return;
                                setRevokingNonce(inv.nonce_b58);
                                try {
                                  await tauriInvoke("invitation_revoke", {
                                    networkCid: selected.cid_short,
                                    nonceB58: inv.nonce_b58,
                                  });
                                  setInvitations((prev) => prev.map((i) =>
                                    i.nonce_b58 === inv.nonce_b58 ? { ...i, revoked: true } : i
                                  ));
                                } catch (e) { showToast(String(e)); }
                                finally { setRevokingNonce(null); }
                              }}
                            >
                              {revokingNonce === inv.nonce_b58 ? "…" : "Révoquer"}
                            </button>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </section>
            )}

            {/* Garde-fou majoritaire */}
            {selected && activeView === 'gouvernance' && adminActions.length > 0 && (
              <section>
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide mb-3">
                  Décisions en contestation
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
            {selected && activeView === 'gouvernance' && <section>
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                  Votes & Propositions
                  {proposals.filter(p => p.status === "open").length > 0 && (
                    <span className="ml-2 text-xs font-normal text-green-600 normal-case">
                      {proposals.filter(p => p.status === "open").length} ouverte{proposals.filter(p => p.status === "open").length > 1 ? "s" : ""}
                    </span>
                  )}
                </h3>
                <button
                  onClick={() => setShowProposalForm((v) => !v)}
                  className="text-xs px-3 py-1.5 bg-civium-600 text-white rounded-lg
                             hover:bg-civium-700 transition-colors"
                >
                  {showProposalForm ? "Annuler" : "+ Nouvelle proposition"}
                </button>
              </div>

              {/* Delegation panel (collapsed by default) */}
              {showDelegationPanel && (() => {
                const globalDel = myDelegations.find((d) => d.proposal_id === null);
                return (
                  <div className="bg-blue-50 border border-blue-100 rounded-xl px-4 py-3 mb-3">
                    <p className="text-xs font-medium text-blue-700 mb-2">Délégation réseau (toutes propositions)</p>
                    {globalDel ? (
                      <div className="flex items-center gap-2 text-xs text-blue-600">
                        <span>Active → <span className="font-mono">{globalDel.delegate_cid_short}</span></span>
                        <button
                          onClick={() => handleRevokeDelegation(null)}
                          className="text-gray-400 hover:text-red-500 transition-colors"
                        >
                          Révoquer
                        </button>
                      </div>
                    ) : (
                      <div className="flex items-center gap-2">
                        <input
                          type="text"
                          value={delegatingTo["global"] ?? ""}
                          onChange={(e) => setDelegatingTo((p) => ({ ...p, global: e.target.value }))}
                          placeholder="CID court du délégué…"
                          className="border border-blue-200 bg-white rounded px-2 py-1 text-xs
                                     focus:outline-none focus:ring-1 focus:ring-blue-300 w-52"
                        />
                        <button
                          onClick={() => handleDelegate(null, delegatingTo["global"] ?? "")}
                          disabled={savingDelegation === "global" || !delegatingTo["global"]?.trim()}
                          className="text-xs px-2 py-1 bg-blue-600 text-white
                                     rounded hover:bg-blue-700 disabled:opacity-50 transition-colors"
                        >
                          {savingDelegation === "global" ? "…" : "Déléguer"}
                        </button>
                      </div>
                    )}
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
                            : prop.status === "cancelled"
                            ? "bg-red-100 text-red-600"
                            : "bg-gray-100 text-gray-500"
                        }`}>
                          {prop.status === "open" ? "Ouverte" : prop.status === "cancelled" ? "Annulée" : "Fermée"}
                        </span>
                      </div>

                      {/* Per-proposal delegation (shown only in delegation panel) */}
                      {showDelegationPanel && prop.status === "open" && (() => {
                        const propDel = myDelegations.find((d) => d.proposal_id === prop.id);
                        const globalDel = myDelegations.find((d) => d.proposal_id === null);
                        const activeDel = propDel ?? globalDel;
                        const key = prop.id;
                        return (
                          <div className="flex items-center gap-2 text-xs bg-blue-50 rounded-lg px-3 py-2">
                            <span className="text-blue-500 font-medium">Délégation :</span>
                            {activeDel ? (
                              <>
                                <span className="text-blue-600">
                                  → <span className="font-mono">{activeDel.delegate_cid_short}</span>
                                  {activeDel.proposal_id === null && " (réseau)"}
                                </span>
                                <button
                                  onClick={() => handleRevokeDelegation(propDel ? prop.id : null)}
                                  className="text-gray-400 hover:text-red-500 transition-colors ml-1"
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
                                  placeholder="CID court…"
                                  className="border border-blue-200 bg-white rounded px-2 py-1 text-xs
                                             focus:outline-none focus:ring-1 focus:ring-blue-300 w-36"
                                />
                                <button
                                  onClick={() => handleDelegate(prop.id, delegatingTo[key] ?? "")}
                                  disabled={savingDelegation === key || !delegatingTo[key]?.trim()}
                                  className="px-2 py-1 bg-blue-600 text-white text-xs
                                             rounded hover:bg-blue-700 disabled:opacity-50 transition-colors"
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
                              className="text-sm px-5 py-2.5 bg-civium-600 text-white rounded-lg
                                         hover:bg-civium-700 disabled:opacity-50 transition-colors font-medium"
                            >
                              {voting === prop.id ? "…" : opt}
                            </button>
                          ))}
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

              {/* Delegation — advanced option at the bottom */}
              <div className="mt-4 pt-3 border-t border-gray-100">
                <button
                  onClick={() => setShowDelegationPanel((v) => !v)}
                  className="text-xs text-gray-400 hover:text-civium-600 transition-colors"
                >
                  {showDelegationPanel ? "▲ Masquer les délégations de vote" : "▼ Gérer mes délégations de vote"}
                </button>
              </div>
            </section>}

            {/* Thread messages */}
            {selected && activeView === 'messages' && <section>
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
                              h-[calc(100vh-320px)] min-h-64 overflow-y-auto p-4 space-y-4">
                {/* Pagination — load older messages */}
                {hasMoreMessages && (
                  <div className="text-center pb-2">
                    <button
                      onClick={() => loadOlderMessages(selected.cid_short)}
                      disabled={loadingOlderMessages}
                      className="text-xs px-3 py-1.5 bg-gray-100 hover:bg-gray-200
                                 text-gray-600 rounded-lg transition-colors disabled:opacity-50"
                    >
                      {loadingOlderMessages ? "Chargement…" : "↑ Charger les messages précédents"}
                    </button>
                  </div>
                )}
                {messages.filter((m) => !m.is_direct && !mutedMembers.has(m.author_cid_short)).length === 0 ? (
                  <p className="text-sm text-gray-400 text-center py-4">
                    Aucun message. Soyez le premier à écrire !
                  </p>
                ) : (
                  messages
                    .filter((m) => !m.is_direct && !mutedMembers.has(m.author_cid_short))
                    .map((msg) => {
                      const iAmAdmin = members.find((x) => x.cid_short === identity?.cid_short)?.role === "admin";
                      return (
                      <div key={msg.id} className="flex gap-3 group">
                        <div className="w-7 h-7 rounded-full bg-civium-100 flex-shrink-0 flex
                                        items-center justify-center text-civium-700 text-xs font-semibold">
                          {msg.author_name[0]?.toUpperCase()}
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-baseline gap-2 flex-wrap">
                            <span className="text-sm font-medium text-gray-900">{msg.author_name}</span>
                            <span className="text-xs text-civium-400 font-medium">{msg.network_name}</span>
                            <span className="text-xs text-gray-400">{formatTime(msg.sent_at)}</span>
                            <button
                              onClick={async () => {
                                const reason = prompt("Raison du signalement (facultatif) :") ?? "";
                                try {
                                  await tauriInvoke("message_report", { networkCid: selected!.cid_short, messageId: msg.id, reason });
                                  showToast("Message signalé aux administrateurs.", "ok");
                                } catch (e) { showToast(String(e)); }
                              }}
                              className="opacity-0 group-hover:opacity-100 text-xs text-orange-400 hover:text-orange-600 transition-opacity ml-auto"
                              title="Signaler ce message"
                              aria-label="Signaler ce message"
                            >
                              ⚑
                            </button>
                            {(iAmAdmin || msg.author_cid_short === identity?.cid_short) && (
                              <button
                                onClick={async () => {
                                  if (!confirm("Supprimer ce message ?")) return;
                                  setDeletingMessage(msg.id);
                                  try {
                                    await tauriInvoke("message_delete", { networkCid: selected!.cid_short, messageId: msg.id });
                                    setMessages((prev) => prev.filter((m) => m.id !== msg.id));
                                  } catch (e) { showToast(String(e)); }
                                  finally { setDeletingMessage(null); }
                                }}
                                disabled={deletingMessage === msg.id}
                                className="opacity-0 group-hover:opacity-100 text-xs text-red-400 hover:text-red-600 transition-opacity disabled:opacity-50"
                                title={iAmAdmin ? "Supprimer (admin)" : "Supprimer mon message"}
                                aria-label="Supprimer ce message"
                              >
                                {deletingMessage === msg.id ? "…" : "✕"}
                              </button>
                            )}
                          </div>
                          {msg.is_file ? (
                            <div className="mt-2 w-full">
                              {/* Image — pleine largeur, clic pour agrandir */}
                              {msg.mime_type?.startsWith("image/") && fileDataCache[msg.id] && (
                                <img
                                  src={`data:${msg.mime_type};base64,${fileDataCache[msg.id]}`}
                                  alt={msg.filename ?? "image"}
                                  className="w-full rounded-xl border border-gray-200 object-contain cursor-zoom-in mb-1"
                                  onClick={() => setLightboxSrc(`data:${msg.mime_type};base64,${fileDataCache[msg.id]}`)}
                                />
                              )}
                              {/* Vidéo — pleine largeur */}
                              {msg.mime_type?.startsWith("video/") && fileDataCache[msg.id] && (
                                <video
                                  controls
                                  src={`data:${msg.mime_type};base64,${fileDataCache[msg.id]}`}
                                  className="w-full rounded-xl border border-gray-200 mb-1"
                                />
                              )}
                              {/* Audio */}
                              {msg.mime_type?.startsWith("audio/") && fileDataCache[msg.id] && (
                                <audio
                                  controls
                                  src={`data:${msg.mime_type};base64,${fileDataCache[msg.id]}`}
                                  className="w-full mb-1"
                                />
                              )}
                              {/* Texte brut (txt, md, csv, json…) */}
                              {(msg.mime_type?.startsWith("text/") || msg.mime_type === "application/json") && fileDataCache[msg.id] && (
                                <pre className="w-full bg-gray-900 text-green-300 text-xs rounded-xl p-3 mb-1 overflow-auto max-h-48 whitespace-pre-wrap break-words">
                                  {(() => {
                                    try {
                                      const bytes = Uint8Array.from(atob(fileDataCache[msg.id]), (c) => c.charCodeAt(0));
                                      return new TextDecoder("utf-8").decode(bytes);
                                    } catch { return atob(fileDataCache[msg.id]); }
                                  })()}
                                </pre>
                              )}
                              {/* PDF — iframe intégrée */}
                              {msg.mime_type === "application/pdf" && fileDataCache[msg.id] && (
                                <iframe
                                  src={`data:application/pdf;base64,${fileDataCache[msg.id]}`}
                                  className="w-full rounded-xl border border-gray-200 mb-1"
                                  style={{ height: "320px" }}
                                  title={msg.filename ?? "pdf"}
                                />
                              )}
                              {/* Barre de fichier : nom + taille + télécharger */}
                              <div className="flex items-center gap-2 bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-700">
                                <span className="text-lg flex-shrink-0">
                                  {msg.mime_type?.startsWith("image/") ? "🖼️"
                                    : msg.mime_type?.startsWith("audio/") ? "🎵"
                                    : msg.mime_type?.startsWith("video/") ? "🎬"
                                    : msg.mime_type === "application/pdf" ? "📄"
                                    : msg.mime_type?.startsWith("text/") ? "📝"
                                    : "📎"}
                                </span>
                                <div className="min-w-0 flex-1">
                                  <p className="font-medium truncate">{msg.filename}</p>
                                  <p className="text-xs text-gray-400">
                                    {msg.size_bytes && msg.size_bytes >= 1_048_576
                                      ? `${(msg.size_bytes / 1_048_576).toFixed(1)} Mo`
                                      : `${Math.round((msg.size_bytes ?? 0) / 1024)} Ko`}
                                    {loadingFile === msg.id && " — chargement…"}
                                  </p>
                                </div>
                                {fileDataCache[msg.id] && (
                                  <button
                                    onClick={() => handleDownloadFile(msg)}
                                    className="flex-shrink-0 text-xs px-2 py-1 bg-civium-600 text-white rounded hover:bg-civium-700 transition-colors"
                                  >
                                    ⬇ Télécharger
                                  </button>
                                )}
                              </div>
                            </div>
                          ) : msg.is_calendar_event ? (
                            <div className="mt-1 flex items-center gap-2 bg-blue-50 border border-blue-200
                                            rounded-lg px-3 py-2 text-sm text-blue-800 max-w-xs">
                              <span className="text-lg">📅</span>
                              <div>
                                <p className="font-medium">{msg.event_title}</p>
                                {msg.event_start && (
                                  <p className="text-xs text-blue-600">
                                    {new Date(msg.event_start * 1000).toLocaleString("fr-FR")}
                                    {msg.event_end ? ` → ${new Date(msg.event_end * 1000).toLocaleTimeString("fr-FR")}` : ""}
                                  </p>
                                )}
                                {msg.event_location && (
                                  <p className="text-xs text-blue-500">📍 {msg.event_location}</p>
                                )}
                              </div>
                            </div>
                          ) : (
                            <div className="text-sm text-gray-700 mt-0.5 prose prose-sm max-w-none
                                            prose-p:my-0.5 prose-pre:bg-gray-100 prose-pre:rounded prose-code:text-xs
                                            prose-a:text-civium-600 prose-a:underline break-words">
                              <ReactMarkdown
                                rehypePlugins={[rehypeSanitize]}
                                components={{
                                  a: ({ href, children }) => (
                                    <a
                                      href={href}
                                      onClick={(e) => { e.preventDefault(); if (href) shellOpen(href); }}
                                      className="text-civium-600 underline hover:text-civium-800 cursor-pointer"
                                    >{children}</a>
                                  ),
                                }}
                              >{msg.body}</ReactMarkdown>
                            </div>
                          )}
                        </div>
                      </div>
                      );
                    })
                )}
                <div ref={messagesEndRef} />
              </div>

              {/* Send form */}
              <div className="bg-white border border-gray-200 rounded-b-xl">
                {/* Markdown toolbar */}
                <div className="flex items-center gap-1 px-3 pt-2 pb-1 border-b border-gray-100">
                  {[
                    { label: "G", title: "Gras", wrap: "**", icon: <strong>G</strong> },
                    { label: "I", title: "Italique", wrap: "_", icon: <em>I</em> },
                    { label: "`", title: "Code inline", wrap: "`", icon: <code className="font-mono">`</code> },
                  ].map(({ label, title, wrap, icon }) => (
                    <button
                      key={label}
                      type="button"
                      title={title}
                      disabled={sending || sendingFile}
                      className="text-xs w-7 h-7 flex items-center justify-center rounded border border-gray-200
                                 text-gray-600 hover:bg-gray-100 disabled:opacity-40 transition-colors"
                      onClick={() => {
                        const ta = document.querySelector<HTMLTextAreaElement>("#msg-textarea");
                        if (!ta) return;
                        const { selectionStart: s, selectionEnd: e } = ta;
                        const before = msgBody.slice(0, s);
                        const selected = msgBody.slice(s, e);
                        const after = msgBody.slice(e);
                        setMsgBody(`${before}${wrap}${selected || title}${wrap}${after}`);
                        setTimeout(() => { ta.focus(); ta.setSelectionRange(s + wrap.length, s + wrap.length + (selected || title).length); }, 0);
                      }}
                    >{icon}</button>
                  ))}
                  <span className="text-xs text-gray-300 ml-1">Markdown supporté</span>
                </div>
                <div className="flex gap-2 p-3">
                  <input
                    type="file"
                    ref={fileInputRef}
                    onChange={handleFileSelect}
                    className="hidden"
                  />
                  <button
                    onClick={() => fileInputRef.current?.click()}
                    disabled={sendingFile || sending}
                    title="Joindre un fichier"
                    className="self-end text-lg px-2 py-1.5 text-gray-400 hover:text-civium-600
                               disabled:opacity-40 transition-colors rounded-lg hover:bg-gray-50"
                  >
                    📎
                  </button>
                  <button
                    onClick={openWebcam}
                    disabled={sendingFile || sending}
                    title="Prendre une photo avec la webcam"
                    className="self-end text-lg px-2 py-1.5 text-gray-400 hover:text-civium-600
                               disabled:opacity-40 transition-colors rounded-lg hover:bg-gray-50"
                  >
                    📷
                  </button>
                  <textarea
                    id="msg-textarea"
                    value={msgBody}
                    onChange={(e) => setMsgBody(e.target.value)}
                    onKeyDown={handleMsgKeyDown}
                    placeholder="Écrire un message… (Entrée pour envoyer, Maj+Entrée pour sauter une ligne)"
                    rows={2}
                    disabled={sending || sendingFile}
                    className="flex-1 text-sm resize-none border border-gray-200 rounded-lg px-3 py-2
                               focus:outline-none focus:ring-2 focus:ring-civium-400 disabled:opacity-50
                               placeholder:text-gray-400"
                  />
                  <button
                    onClick={handleSendMessage}
                    disabled={sending || sendingFile || !msgBody.trim()}
                    className="self-end text-xs px-4 py-2 bg-civium-600 text-white rounded-lg
                               hover:bg-civium-700 disabled:opacity-50 transition-colors font-medium"
                  >
                    {sending || sendingFile ? "…" : "Envoyer"}
                  </button>
                </div>
              </div>
            </section>}

            {/* ── Activité section ── */}
            {selected && activeView === 'activite' && <section>
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
            {selected && activeView === 'agenda' && <section>
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                  Agenda ({agendaEvents.length})
                </h3>
                <div className="flex gap-2">
                  {agendaEvents.length > 0 && (
                    <button
                      onClick={async () => {
                        if (!selected) return;
                        try {
                          const ics = await tauriInvoke<string>("agenda_export_ics", { networkCidShort: selected.cid_short });
                          const blob = new Blob([ics], { type: "text/calendar;charset=utf-8" });
                          const url = URL.createObjectURL(blob);
                          const a = document.createElement("a");
                          a.href = url;
                          a.download = `civium-agenda-${selected.cid_short}.ics`;
                          a.click();
                          URL.revokeObjectURL(url);
                        } catch (e) { showToast(String(e)); }
                      }}
                      className="text-xs text-gray-500 hover:text-gray-700"
                      title="Exporter vers calendrier (.ics)"
                    >
                      ↓ .ics
                    </button>
                  )}
                  <button
                    onClick={() => setShowAgendaForm((v) => !v)}
                    className="text-xs text-indigo-500 hover:text-indigo-700"
                  >
                    {showAgendaForm ? "Annuler" : "+ Événement"}
                  </button>
                </div>
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
                  {agendaEvents.slice(agendaPage * PAGE_SIZE, (agendaPage + 1) * PAGE_SIZE).map((ev) => (
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
                  {agendaEvents.length > PAGE_SIZE && (
                    <div className="flex items-center justify-between mt-2 text-xs text-gray-500">
                      <button
                        disabled={agendaPage === 0}
                        onClick={() => setAgendaPage((p) => p - 1)}
                        className="px-3 py-1 rounded border border-gray-200 bg-white hover:bg-gray-50 disabled:opacity-40 transition-colors"
                      >← Précédent</button>
                      <span>Page {agendaPage + 1} / {Math.ceil(agendaEvents.length / PAGE_SIZE)}</span>
                      <button
                        disabled={(agendaPage + 1) * PAGE_SIZE >= agendaEvents.length}
                        onClick={() => setAgendaPage((p) => p + 1)}
                        className="px-3 py-1 rounded border border-gray-200 bg-white hover:bg-gray-50 disabled:opacity-40 transition-colors"
                      >Suivant →</button>
                    </div>
                  )}
                </div>
              )}
            </section>}

            {/* ── Documents section ── */}
            {selected && activeView === 'documents' && <section>
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
                  {documents.slice(docsPage * PAGE_SIZE, (docsPage + 1) * PAGE_SIZE).map((doc) => (
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
                  {documents.length > PAGE_SIZE && (
                    <div className="flex items-center justify-between mt-2 text-xs text-gray-500">
                      <button
                        disabled={docsPage === 0}
                        onClick={() => setDocsPage((p) => p - 1)}
                        className="px-3 py-1 rounded border border-gray-200 bg-white hover:bg-gray-50 disabled:opacity-40 transition-colors"
                      >← Précédent</button>
                      <span>Page {docsPage + 1} / {Math.ceil(documents.length / PAGE_SIZE)}</span>
                      <button
                        disabled={(docsPage + 1) * PAGE_SIZE >= documents.length}
                        onClick={() => setDocsPage((p) => p + 1)}
                        className="px-3 py-1 rounded border border-gray-200 bg-white hover:bg-gray-50 disabled:opacity-40 transition-colors"
                      >Suivant →</button>
                    </div>
                  )}
                </div>
              )}
            </section>}

            {/* ── RRM section (RRM networks only) ── */}
            {selected && activeView === 'rrm' && selected.is_rrm && (
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
            {selected && activeView === 'annuaire' && !selected.is_directory && !selected.is_rrm && (trustedRrms.length > 0 || showTrustForm) && (
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
            {selected && activeView === 'annuaire' && !selected.is_directory && !selected.is_rrm && trustedRrms.length === 0 && !showTrustForm && (
              <button
                onClick={() => setShowTrustForm(true)}
                className="text-xs text-gray-400 hover:text-orange-600 transition-colors block"
              >
                + Approuver un registre de surveillance
              </button>
            )}

            {/* ── Annuaire section (directory networks only) ── */}
            {selected && activeView === 'annuaire' && selected.is_directory && (
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
                  const pageItems = items.slice(dirPage * PAGE_SIZE, (dirPage + 1) * PAGE_SIZE);
                  return (
                    <div className="bg-white border border-gray-200 rounded-xl divide-y divide-gray-100">
                      {pageItems.map((entry) => (
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
                      {items.length > PAGE_SIZE && (
                        <div className="flex items-center justify-between px-4 py-2 text-xs text-gray-500">
                          <button
                            disabled={dirPage === 0}
                            onClick={() => setDirPage((p) => p - 1)}
                            className="px-3 py-1 rounded border border-gray-200 bg-white hover:bg-gray-50 disabled:opacity-40 transition-colors"
                          >← Précédent</button>
                          <span>Page {dirPage + 1} / {Math.ceil(items.length / PAGE_SIZE)}</span>
                          <button
                            disabled={(dirPage + 1) * PAGE_SIZE >= items.length}
                            onClick={() => setDirPage((p) => p + 1)}
                            className="px-3 py-1 rounded border border-gray-200 bg-white hover:bg-gray-50 disabled:opacity-40 transition-colors"
                          >Suivant →</button>
                        </div>
                      )}
                    </div>
                  );
                })()}
              </section>
            )}

            {/* ── Notifications section ── */}
            {selected && activeView === 'notifications' && (
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

            {/* ── Connexions inter-réseaux (APC) ── */}
            {selected && activeView === 'connexions' && (
              <section className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide">
                    Connexions inter-réseaux (APC)
                  </h3>
                  <button
                    className="text-xs text-civium-600 hover:text-civium-800"
                    onClick={() => tauriInvoke<ConnectionInfo[]>("connection_list", { networkCid: selected.cid_short })
                      .then(setConnections).catch(() => {})}
                  >
                    ↻ Actualiser
                  </button>
                </div>
                <p className="text-xs text-gray-400">
                  Chaque connexion est régie par un Accord de Partage Civium (APC — Accord de Partage Civium) signé par les deux réseaux.
                  Les connexions en attente doivent être acceptées ou refusées par un admin.
                </p>
                {connections.length === 0 ? (
                  <div className="bg-white rounded-xl border border-gray-200 px-4 py-6 text-center">
                    <p className="text-sm text-gray-400">Aucune connexion inter-réseau.</p>
                    <p className="text-xs text-gray-300 mt-1">Utilisez le CLI <code>civium connect request</code> pour initier une connexion.</p>
                  </div>
                ) : (
                  <div className="space-y-3">
                    {connections.map((c) => {
                      const isPending = c.state === "En attente";
                      const isActive  = c.state === "Active";
                      const stateColor = isPending ? "text-amber-600 bg-amber-50 border-amber-200"
                        : isActive ? "text-green-600 bg-green-50 border-green-200"
                        : "text-gray-500 bg-gray-50 border-gray-200";
                      return (
                        <div key={c.peer_cid_full} className="bg-white rounded-xl border border-gray-200 p-4 space-y-3">
                          <div className="flex items-start justify-between gap-3">
                            <div className="min-w-0">
                              <p className="text-sm font-semibold text-gray-800 truncate">{c.peer_name}</p>
                              <p className="text-xs font-mono text-gray-400">{c.peer_cid_short}</p>
                            </div>
                            <span className={`text-xs px-2 py-0.5 rounded-full border shrink-0 ${stateColor}`}>{c.state}</span>
                          </div>
                          {/* APC terms */}
                          {isActive && (
                            <div className="grid grid-cols-2 gap-2 text-xs text-gray-600">
                              <div className="bg-gray-50 rounded-lg px-3 py-2">
                                <p className="font-medium text-gray-500 mb-0.5">Nous exposons</p>
                                <p>{c.expose_directory_to_peer ? "Annuaire des membres" : "Rien (privé)"}</p>
                              </div>
                              <div className="bg-gray-50 rounded-lg px-3 py-2">
                                <p className="font-medium text-gray-500 mb-0.5">Ils exposent</p>
                                <p>{c.peer_exposes_directory ? "Annuaire des membres" : "Rien (privé)"}</p>
                              </div>
                            </div>
                          )}
                          {c.apc_nonce && (
                            <p className="text-xs text-gray-400">
                              Nonce APC : <code className="font-mono">{c.apc_nonce.slice(0, 12)}…</code>
                              {" — "}{new Date(c.updated_at * 1000).toLocaleDateString("fr-FR")}
                            </p>
                          )}
                          {/* Actions */}
                          {isPending && (
                            <div className="flex flex-wrap gap-2">
                              <label className="flex items-center gap-1.5 text-xs text-gray-600">
                                <input
                                  type="checkbox"
                                  id={`expose-${c.peer_cid_full}`}
                                  defaultChecked={true}
                                  className="rounded"
                                />
                                Exposer mon annuaire
                              </label>
                              <button
                                className="text-xs px-3 py-1 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50 transition-colors"
                                disabled={acceptingConn === c.peer_cid_full}
                                onClick={async () => {
                                  const checkbox = document.getElementById(`expose-${c.peer_cid_full}`) as HTMLInputElement;
                                  setAcceptingConn(c.peer_cid_full);
                                  try {
                                    await tauriInvoke("connection_accept", {
                                      networkCid: selected.cid_short,
                                      peerCidFull: c.peer_cid_full,
                                      exposeDirectory: checkbox?.checked ?? true,
                                    });
                                    showToast("Connexion acceptée — APC signé.", "ok");
                                    const updated = await tauriInvoke<ConnectionInfo[]>("connection_list", { networkCid: selected.cid_short });
                                    setConnections(updated);
                                  } catch (e) { showToast(String(e)); }
                                  finally { setAcceptingConn(null); }
                                }}
                              >
                                {acceptingConn === c.peer_cid_full ? "Signature…" : "✓ Accepter"}
                              </button>
                              <button
                                className="text-xs px-3 py-1 border border-gray-300 text-gray-600 rounded-lg hover:bg-gray-50 disabled:opacity-50 transition-colors"
                                disabled={actingConn === c.peer_cid_full}
                                onClick={async () => {
                                  setActingConn(c.peer_cid_full);
                                  try {
                                    await tauriInvoke("connection_refuse", { networkCid: selected.cid_short, peerCidFull: c.peer_cid_full, reason: null });
                                    showToast("Connexion refusée.", "ok");
                                    setConnections((prev) => prev.filter((x) => x.peer_cid_full !== c.peer_cid_full));
                                  } catch (e) { showToast(String(e)); }
                                  finally { setActingConn(null); }
                                }}
                              >
                                ✗ Refuser
                              </button>
                            </div>
                          )}
                          {(isActive || c.state === "Demandée") && (
                            <div className="flex gap-2">
                              <button
                                className="text-xs px-2 py-1 border border-red-200 text-red-600 rounded hover:bg-red-50 disabled:opacity-50 transition-colors"
                                disabled={actingConn === c.peer_cid_full}
                                onClick={async () => {
                                  if (!confirm(`Révoquer la connexion avec « ${c.peer_name} » ?`)) return;
                                  setActingConn(c.peer_cid_full);
                                  try {
                                    await tauriInvoke("connection_revoke", { networkCid: selected.cid_short, peerCidFull: c.peer_cid_full });
                                    showToast("Connexion révoquée.", "ok");
                                    const updated = await tauriInvoke<ConnectionInfo[]>("connection_list", { networkCid: selected.cid_short });
                                    setConnections(updated);
                                  } catch (e) { showToast(String(e)); }
                                  finally { setActingConn(null); }
                                }}
                              >
                                Révoquer
                              </button>
                              <button
                                className="text-xs px-2 py-1 border border-gray-300 text-gray-500 rounded hover:bg-gray-50 disabled:opacity-50 transition-colors"
                                disabled={actingConn === c.peer_cid_full}
                                onClick={async () => {
                                  if (!confirm(`Bloquer « ${c.peer_name} » ? Le réseau verra une connexion refusée.`)) return;
                                  setActingConn(c.peer_cid_full);
                                  try {
                                    await tauriInvoke("connection_block", { networkCid: selected.cid_short, peerCidFull: c.peer_cid_full });
                                    showToast("Réseau bloqué.", "ok");
                                    const updated = await tauriInvoke<ConnectionInfo[]>("connection_list", { networkCid: selected.cid_short });
                                    setConnections(updated);
                                  } catch (e) { showToast(String(e)); }
                                  finally { setActingConn(null); }
                                }}
                              >
                                Bloquer
                              </button>
                            </div>
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
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide mb-1">Extensions (Plugins)</h3>
                <p className="text-xs text-gray-400 mb-3">
                  Les extensions s'appliquent à <strong>tous vos réseaux</strong>. Activer ou désactiver une extension ici la modifie pour l'ensemble du nœud.
                </p>
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

      {/* Lightbox */}
      {lightboxSrc && (
        <div
          className="fixed inset-0 z-50 bg-black/90 flex items-center justify-center p-4 cursor-zoom-out"
          onClick={() => setLightboxSrc(null)}
        >
          <img
            src={lightboxSrc}
            alt="Aperçu"
            className="max-w-full max-h-full object-contain rounded-lg select-none"
            onClick={(e) => e.stopPropagation()}
          />
          <button
            onClick={() => setLightboxSrc(null)}
            className="absolute top-4 right-4 text-white/80 hover:text-white text-3xl leading-none"
          >
            ✕
          </button>
        </div>
      )}

      {/* Webcam capture modal — always mounted so webcamVideoRef is never null */}
      <div
        className={`fixed inset-0 z-50 bg-black/70 flex items-center justify-center p-4 ${showWebcam ? "" : "hidden"}`}
        onClick={(e) => { if (e.target === e.currentTarget) closeWebcam(); }}
      >
        <div className="bg-white rounded-2xl shadow-xl max-w-lg w-full p-6 space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="font-semibold text-gray-900">Caméra</h3>
            <button onClick={closeWebcam} className="text-gray-400 hover:text-gray-600 text-xl leading-none">✕</button>
          </div>

          {/* Mode toggle — hidden during/after recording */}
          {!capturedPhoto && !recordedBlob && !isRecording && (
            <div className="flex rounded-lg border border-gray-200 overflow-hidden text-sm font-medium">
              <button
                onClick={() => setWebcamMode("photo")}
                className={`flex-1 py-2 transition-colors ${webcamMode === "photo" ? "bg-civium-600 text-white" : "text-gray-600 hover:bg-gray-50"}`}
              >
                📷 Photo
              </button>
              <button
                onClick={() => setWebcamMode("video")}
                className={`flex-1 py-2 transition-colors ${webcamMode === "video" ? "bg-civium-600 text-white" : "text-gray-600 hover:bg-gray-50"}`}
              >
                🎥 Vidéo
              </button>
            </div>
          )}

          {/* Preview après capture photo */}
          {capturedPhoto ? (
            <div className="space-y-3">
              <img src={capturedPhoto} alt="Aperçu" className="w-full rounded-xl border border-gray-200" />
              <div className="flex gap-2">
                <button
                  onClick={() => { setCapturedPhoto(null); openWebcam(); }}
                  className="flex-1 py-2 border border-gray-300 rounded-lg text-sm text-gray-600 hover:bg-gray-50"
                >
                  Reprendre
                </button>
                <button onClick={sendCapturedPhoto} disabled={sendingFile}
                  className="flex-1 py-2 bg-civium-600 text-white rounded-lg text-sm font-medium hover:bg-civium-700 disabled:opacity-50">
                  {sendingFile ? "Envoi…" : "Envoyer"}
                </button>
              </div>
            </div>

          ) : recordedBlob ? (
            /* Preview après enregistrement vidéo */
            <div className="space-y-3">
              <video
                src={recordedUrl ?? ""}
                controls
                playsInline
                className="w-full rounded-xl bg-black aspect-video"
                autoPlay
              />
              <div className="flex gap-2">
                <button
                  onClick={() => {
                    setRecordedBlob(null);
                    if (recordedUrl) { URL.revokeObjectURL(recordedUrl); setRecordedUrl(null); }
                    openWebcam();
                  }}
                  className="flex-1 py-2 border border-gray-300 rounded-lg text-sm text-gray-600 hover:bg-gray-50"
                >
                  Reprendre
                </button>
                <button onClick={sendRecordedVideo} disabled={sendingFile}
                  className="flex-1 py-2 bg-civium-600 text-white rounded-lg text-sm font-medium hover:bg-civium-700 disabled:opacity-50">
                  {sendingFile ? "Envoi…" : "Envoyer"}
                </button>
              </div>
            </div>

          ) : (
            /* Viewfinder en direct */
            <div className="space-y-3">
              <div className="relative">
                <video
                  ref={webcamVideoRef}
                  autoPlay
                  muted
                  playsInline
                  className="w-full rounded-xl bg-black aspect-video object-cover"
                />
                {isRecording && (
                  <div className="absolute top-3 left-3 flex items-center gap-1.5 bg-black/60 text-white text-xs px-2 py-1 rounded-full">
                    <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
                    {String(Math.floor(recordingSeconds / 60)).padStart(2, "0")}:{String(recordingSeconds % 60).padStart(2, "0")}
                  </div>
                )}
              </div>
              <canvas ref={webcamCanvasRef} className="hidden" />

              {webcamMode === "photo" ? (
                <button onClick={capturePhoto}
                  className="w-full py-3 bg-civium-600 text-white rounded-xl text-sm font-semibold hover:bg-civium-700">
                  📷 Capturer
                </button>
              ) : isRecording ? (
                <button onClick={stopRecording}
                  className="w-full py-3 bg-red-600 text-white rounded-xl text-sm font-semibold hover:bg-red-700 flex items-center justify-center gap-2">
                  <span className="w-3 h-3 rounded-sm bg-white inline-block" />
                  Arrêter l'enregistrement
                </button>
              ) : (
                <button onClick={startRecording}
                  className="w-full py-3 bg-red-600 text-white rounded-xl text-sm font-semibold hover:bg-red-700 flex items-center justify-center gap-2">
                  <span className="w-3 h-3 rounded-full bg-white inline-block" />
                  Démarrer l'enregistrement
                </button>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
