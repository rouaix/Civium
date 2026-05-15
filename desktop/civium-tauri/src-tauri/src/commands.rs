use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use civium_core::{
    network::{Invitation, Network},
    add_contest, compute_result_with_delegations,
    AdminAction, AdminActionKind, AdminActionStatus,
    CiviumKeypair, CiviumNode, CiviumRequest, CiviumResponse,
    DirectoryEntry, EntryKind, FederatedDirectory, GroupKey, GuardianLink, MemberRole, Message, MessageKind,
    MinorRestrictions, Multiaddr, NetworkKind, NodeCommand, NodeConfig, NodeEvent, PluginState, Proposal, ProposalStatus,
    RrmEntry, TrustedRrm, TrustCircle, Vote, VoteDelegation, peer_id_from_multiaddr,
};

use crate::{node::AppState, store};

// ── Return types ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct IdentityInfo {
    pub cid_short: String,
    pub cid_full: String,
    pub secret_b58: String,
}

#[derive(Serialize)]
pub struct NetworkInfo {
    pub cid_short: String,
    pub cid_full: String,
    pub name: String,
    pub member_count: usize,
    pub is_directory: bool,
    pub is_rrm: bool,
}

#[derive(Serialize)]
pub struct MemberInfo {
    pub cid_short: String,
    pub display_name: String,
    pub circle: u8,
    pub role: String,
    pub is_minor: bool,
}

#[derive(Serialize)]
pub struct GuardianLinkInfo {
    pub id: String,
    pub network_cid_short: String,
    pub minor_cid_short: String,
    pub guardian_cid_short: String,
    pub added_by: String,
    pub added_at: u64,
}

#[derive(Serialize)]
pub struct MinorRestrictionsInfo {
    pub network_cid_short: String,
    pub minor_cid_short: String,
    pub max_circle: u8,
    pub allowed_cid_shorts: Vec<String>,
    pub updated_by: String,
    pub updated_at: u64,
}

#[derive(Serialize)]
pub struct PendingMemberInfo {
    pub cid_short: String,
    pub display_name: String,
    pub requested_at: u64,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn data_dir(app: &AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("./civium-data"))
}

fn open(app: &AppHandle) -> Result<rusqlite::Connection, String> {
    store::open_db(&data_dir(app)).map_err(|e| e.to_string())
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn identity_exists(app: AppHandle) -> bool {
    open(&app).map(|c| store::identity_exists(&c)).unwrap_or(false)
}

#[tauri::command]
pub fn identity_init(app: AppHandle) -> Result<IdentityInfo, String> {
    let conn = open(&app)?;
    if store::identity_exists(&conn) {
        return Err("identity already exists — use identity_show".into());
    }
    let keypair = CiviumKeypair::generate().map_err(|e| e.to_string())?;
    let cid = keypair.cid();
    store::save_identity(&conn, &keypair).map_err(|e| e.to_string())?;
    Ok(IdentityInfo {
        cid_short: cid.short().to_string(),
        cid_full: cid.full().to_string(),
        secret_b58: keypair.secret_b58(),
    })
}

#[tauri::command]
pub fn network_create(
    app: AppHandle,
    name: String,
    display_name: String,
) -> Result<NetworkInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let admin_cid = keypair.cid();

    let network = Network::create(name, &admin_cid, display_name)
        .map_err(|e| e.to_string())?;

    let info = NetworkInfo {
        cid_short: network.cid_short().to_string(),
        cid_full: network.cid_full().to_string(),
        name: network.name().to_string(),
        member_count: network.data.members.len(),
        is_directory: false,
        is_rrm: false,
    };

    store::save_network(&conn, &network).map_err(|e| e.to_string())?;
    Ok(info)
}

#[tauri::command]
pub fn network_list(app: AppHandle) -> Result<Vec<NetworkInfo>, String> {
    let conn = open(&app)?;
    let networks = store::list_networks(&conn).map_err(|e| e.to_string())?;
    Ok(networks
        .iter()
        .map(|n| NetworkInfo {
            cid_short: n.cid_short().to_string(),
            cid_full: n.cid_full().to_string(),
            name: n.name().to_string(),
            member_count: n.data.members.len(),
            is_directory: n.data.kind == NetworkKind::Directory,
            is_rrm: n.data.kind == NetworkKind::Rrm,
        })
        .collect())
}

#[tauri::command]
pub fn network_invite(
    app: AppHandle,
    network_cid: String,
    expires_in: u64,
) -> Result<String, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let inviter_cid = keypair.cid();
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    network
        .create_invitation(&inviter_cid, expires_in)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn member_list(
    app: AppHandle,
    network_cid: String,
) -> Result<Vec<MemberInfo>, String> {
    let conn = open(&app)?;
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    Ok(store::network_members(&network)
        .iter()
        .map(|m| MemberInfo {
            cid_short: m.cid_short.clone(),
            display_name: m.display_name.clone(),
            circle: m.circle as u8,
            role: m.role.to_string(),
            is_minor: m.is_minor,
        })
        .collect())
}

/// Join a network via an invitation link (Phase 0: network must already be in local DB).
/// Auto-admits the joiner at circle Connaissance — no P2P needed in Phase 0.
#[tauri::command]
pub fn network_join(
    app: AppHandle,
    invite_link: String,
    display_name: String,
) -> Result<NetworkInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let member_cid = keypair.cid();

    let invitation = Invitation::from_link(&invite_link).map_err(|e| e.to_string())?;
    invitation.verify().map_err(|e| e.to_string())?;

    let network_cid_full = invitation.network_cid_full().to_string();
    let networks = store::list_networks(&conn).map_err(|e| e.to_string())?;
    let mut network = networks
        .into_iter()
        .find(|n| n.cid_full() == network_cid_full)
        .ok_or_else(|| format!(
            "Réseau '{}' introuvable localement. En Phase 0, partagez la même base de données que l'admin.",
            invitation.network_name()
        ))?;

    network
        .submit_join_request(&member_cid, display_name, &invitation)
        .map_err(|e| e.to_string())?;

    let record = network
        .admit(member_cid.short(), TrustCircle::Connaissance, MemberRole::Member)
        .map_err(|e| e.to_string())?;

    let info = NetworkInfo {
        cid_short: network.cid_short().to_string(),
        cid_full: network.cid_full().to_string(),
        name: network.name().to_string(),
        member_count: network.data.members.len(),
        is_directory: network.data.kind == NetworkKind::Directory,
        is_rrm: network.data.kind == NetworkKind::Rrm,
    };

    store::save_network(&conn, &network).map_err(|e| e.to_string())?;
    let _ = record;
    Ok(info)
}

#[tauri::command]
pub fn member_pending_list(
    app: AppHandle,
    network_cid: String,
) -> Result<Vec<PendingMemberInfo>, String> {
    let conn = open(&app)?;
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    Ok(network.data.pending.iter().map(|p| PendingMemberInfo {
        cid_short: p.cid_short.clone(),
        display_name: p.display_name.clone(),
        requested_at: p.requested_at,
    }).collect())
}

#[tauri::command]
pub fn member_admit(
    app: AppHandle,
    network_cid: String,
    member_cid: String,
    circle: u8,
) -> Result<MemberInfo, String> {
    let circle = TrustCircle::from_u8(circle)
        .ok_or_else(|| format!("cercle invalide: {circle} — utiliser 0, 1 ou 2"))?;
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let admin_cid = keypair.cid();
    let mut network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    let record = network
        .admit(&member_cid, circle, MemberRole::Member)
        .map_err(|e| e.to_string())?;
    store::save_network(&conn, &network).map_err(|e| e.to_string())?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let action = AdminAction::new(
        network_cid.clone(),
        AdminActionKind::MemberAdmitted {
            member_cid_short: record.cid_short.clone(),
            display_name: record.display_name.clone(),
        },
        admin_cid.short().to_string(),
        now,
        0,
    );
    store::save_admin_action(&conn, &network_cid, &action).map_err(|e| e.to_string())?;

    Ok(MemberInfo {
        cid_short: record.cid_short,
        display_name: record.display_name,
        circle: record.circle as u8,
        role: record.role.to_string(),
        is_minor: record.is_minor,
    })
}

// ── P2P node commands ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct NodeStatus {
    pub running: bool,
    pub listen_addrs: Vec<String>,
}

/// Return the current P2P node status (running + listen addresses).
#[tauri::command]
pub fn node_status(app: AppHandle) -> NodeStatus {
    let state = app.state::<AppState>();
    let running = state.node_tx.lock().unwrap().is_some();
    let listen_addrs = state.listen_addrs.lock().unwrap().clone();
    NodeStatus { running, listen_addrs }
}

/// Trigger an immediate peer-discovery + sync cycle for a network.
/// Returns an error if the P2P node isn't running.
#[tauri::command]
pub async fn node_sync(app: AppHandle, network_cid: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let cmd_tx = state.node_tx.lock().unwrap().clone()
        .ok_or_else(|| "P2P node is not running".to_string())?;
    cmd_tx
        .send(NodeCommand::DiscoverPeers { network_cid_short: network_cid })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn member_reject(
    app: AppHandle,
    network_cid: String,
    member_cid: String,
) -> Result<(), String> {
    let conn = open(&app)?;
    let mut network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    network.reject(&member_cid).map_err(|e| e.to_string())?;
    store::save_network(&conn, &network).map_err(|e| e.to_string())?;
    Ok(())
}

// ── P2P join ──────────────────────────────────────────────────────────────────

/// Join a network over P2P using a real invite link + known peer multiaddr.
/// Spins up a short-lived local P2P node, dials the peer, and waits up to 30 s
/// for JoinAccepted / JoinRejected.  Returns the network info on success.
#[tauri::command]
pub async fn network_join_p2p(
    app: AppHandle,
    invite_link: String,
    display_name: String,
    peer_addr: String,
) -> Result<NetworkInfo, String> {
    // Load identity — sync, before any await.
    let keypair = {
        let conn = open(&app)?;
        store::load_identity(&conn).map_err(|e| e.to_string())?
    };
    let member_cid = keypair.cid();

    // Validate invitation and peer address.
    let invitation =
        Invitation::from_link(&invite_link).map_err(|e| e.to_string())?;
    invitation.verify().map_err(|e| e.to_string())?;

    let via_addr: Multiaddr = peer_addr
        .parse()
        .map_err(|e| format!("adresse invalide : {e}"))?;
    let peer_id = peer_id_from_multiaddr(&via_addr)
        .ok_or_else(|| "l'adresse doit inclure /p2p/<PeerId>".to_string())?;

    // Start a short-lived P2P node for this join.
    let config = NodeConfig {
        listen_tcp: "/ip4/0.0.0.0/tcp/0".into(),
        listen_quic: "/ip4/0.0.0.0/udp/0/quic-v1".into(),
        bootstrap_peers: vec![peer_addr.clone()],
    };
    let (node, mut handle) =
        CiviumNode::new(keypair, config).await.map_err(|e| e.to_string())?;

    let join_request = CiviumRequest::Join {
        invite_link: invite_link.clone(),
        member_cid_full: member_cid.full().to_string(),
        display_name: display_name.clone(),
    };

    // Drive the libp2p swarm in the background.
    let node_task = tauri::async_runtime::spawn(async move { node.run().await });

    let join_result = tokio::time::timeout(
        Duration::from_secs(30),
        async {
            let mut join_sent = false;
            loop {
                match handle.events.recv().await {
                    Some(NodeEvent::PeerConnected { peer_id: connected }) => {
                        if connected == peer_id && !join_sent {
                            let _ = handle.commands.send(NodeCommand::SendRequest {
                                peer: peer_id,
                                request: join_request.clone(),
                            }).await;
                            join_sent = true;
                        }
                    }
                    Some(NodeEvent::OutboundResponse { response, .. }) => match response {
                        CiviumResponse::JoinAccepted { network_data } => {
                            return Ok(network_data);
                        }
                        CiviumResponse::JoinRejected { reason } => {
                            return Err(format!("Rejoindre refusé : {reason}"));
                        }
                        _ => {}
                    },
                    None => return Err("connexion au nœud P2P perdue".to_string()),
                    _ => {}
                }
            }
        },
    )
    .await
    .map_err(|_| "délai dépassé — le pair ne répond pas (30 s)".to_string())?;

    node_task.abort();

    let network_data = join_result?;
    let network = Network::from_data(network_data).map_err(|e| e.to_string())?;

    // Save the network — new connection after all awaits.
    {
        let conn = open(&app)?;
        store::save_network(&conn, &network).map_err(|e| e.to_string())?;
    }

    Ok(NetworkInfo {
        cid_short: network.cid_short().to_string(),
        cid_full: network.cid_full().to_string(),
        name: network.name().to_string(),
        member_count: network.data.members.len(),
        is_directory: network.data.kind == NetworkKind::Directory,
        is_rrm: network.data.kind == NetworkKind::Rrm,
    })
}

// ── Messaging commands ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct MessageDisplay {
    pub id: String,
    pub author_cid_short: String,
    pub author_name: String,
    pub body: String,
    pub sent_at: u64,
    pub is_direct: bool,
    pub to_cid_short: Option<String>,
}

/// Return decrypted thread messages for a network, ordered by sent_at.
#[tauri::command]
pub fn message_list(app: AppHandle, network_cid: String) -> Result<Vec<MessageDisplay>, String> {
    let conn = open(&app)?;
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    let group_key =
        GroupKey::from_b58(&network.data.group_key_b58).map_err(|e| e.to_string())?;

    let member_names: HashMap<String, String> = network
        .data
        .members
        .iter()
        .map(|m| (m.cid_short.clone(), m.display_name.clone()))
        .collect();

    let messages = store::load_messages(&conn, &network_cid).map_err(|e| e.to_string())?;

    let mut result = Vec::with_capacity(messages.len());
    for msg in messages {
        let body = group_key
            .decrypt(&msg.nonce_b58, &msg.ciphertext_b58)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_else(|_| "[message illisible]".into());

        let author_name = member_names
            .get(&msg.author_cid_short)
            .cloned()
            .unwrap_or_else(|| msg.author_cid_short.clone());

        let (is_direct, to_cid_short) = match &msg.kind {
            MessageKind::Direct { to_cid_short } => (true, Some(to_cid_short.clone())),
            MessageKind::Thread => (false, None),
        };

        result.push(MessageDisplay {
            id: msg.id,
            author_cid_short: msg.author_cid_short,
            author_name,
            body,
            sent_at: msg.sent_at,
            is_direct,
            to_cid_short,
        });
    }
    Ok(result)
}

/// Encrypt and store a new thread message in the local network mailbox.
#[tauri::command]
pub fn message_send(
    app: AppHandle,
    network_cid: String,
    body: String,
) -> Result<MessageDisplay, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let author_cid = keypair.cid();
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    let group_key =
        GroupKey::from_b58(&network.data.group_key_b58).map_err(|e| e.to_string())?;

    let (nonce_b58, ciphertext_b58) =
        group_key.encrypt(body.as_bytes()).map_err(|e| e.to_string())?;
    let sent_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let msg = Message {
        id: nonce_b58.clone(),
        author_cid_short: author_cid.short().to_string(),
        kind: MessageKind::Thread,
        nonce_b58,
        ciphertext_b58,
        sent_at,
    };
    store::save_message(&conn, &network_cid, &msg).map_err(|e| e.to_string())?;

    let author_name = network
        .data
        .members
        .iter()
        .find(|m| m.cid_short == author_cid.short())
        .map(|m| m.display_name.clone())
        .unwrap_or_else(|| author_cid.short().to_string());

    Ok(MessageDisplay {
        id: msg.id,
        author_cid_short: msg.author_cid_short,
        author_name,
        body,
        sent_at: msg.sent_at,
        is_direct: false,
        to_cid_short: None,
    })
}

/// Send a direct message to a specific member (enforces minor restrictions on both sides).
#[tauri::command]
pub fn message_send_direct(
    app: AppHandle,
    network_cid: String,
    to_cid_short: String,
    body: String,
) -> Result<MessageDisplay, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let author_cid = keypair.cid();
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;

    let author_cid_short = network.data.members.iter()
        .find(|m| m.cid_full == author_cid.full())
        .map(|m| m.cid_short.clone())
        .ok_or_else(|| "you are not a member of this network".to_string())?;

    if !network.data.members.iter().any(|m| m.cid_short == to_cid_short) {
        return Err(format!("member '{}' not found in this network", to_cid_short));
    }

    // Enforce minor restrictions (both directions)
    store::check_minor_interaction(&conn, &network_cid, &to_cid_short, &author_cid_short)
        .map_err(|e| e.to_string())?;
    store::check_minor_interaction(&conn, &network_cid, &author_cid_short, &to_cid_short)
        .map_err(|e| e.to_string())?;

    let group_key = GroupKey::from_b58(&network.data.group_key_b58).map_err(|e| e.to_string())?;
    let (nonce_b58, ciphertext_b58) = group_key.encrypt(body.as_bytes()).map_err(|e| e.to_string())?;
    let sent_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let msg = Message {
        id: nonce_b58.clone(),
        author_cid_short: author_cid_short.clone(),
        kind: MessageKind::Direct { to_cid_short: to_cid_short.clone() },
        nonce_b58,
        ciphertext_b58,
        sent_at,
    };
    store::save_message(&conn, &network_cid, &msg).map_err(|e| e.to_string())?;

    let author_name = network.data.members.iter()
        .find(|m| m.cid_short == author_cid_short)
        .map(|m| m.display_name.clone())
        .unwrap_or_else(|| author_cid_short.clone());

    Ok(MessageDisplay {
        id: msg.id,
        author_cid_short: msg.author_cid_short,
        author_name,
        body,
        sent_at: msg.sent_at,
        is_direct: true,
        to_cid_short: Some(to_cid_short),
    })
}

// ── Governance commands ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ProposalInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub options: Vec<String>,
    pub created_by: String,
    pub created_at: u64,
    pub closes_at: u64,
    pub quorum_percent: u8,
    pub status: String,
}

#[derive(Serialize)]
pub struct OptionResult {
    pub label: String,
    pub votes: usize,
    pub percent: f64,
}

#[derive(Serialize)]
pub struct VoteResultInfo {
    pub proposal_id: String,
    pub total_votes: usize,
    pub total_members: usize,
    pub participation_percent: f64,
    pub quorum_reached: bool,
    pub options: Vec<OptionResult>,
    pub winner: Option<usize>,
}

/// List all proposals for a network.
#[tauri::command]
pub fn proposal_list(app: AppHandle, network_cid: String) -> Result<Vec<ProposalInfo>, String> {
    let conn = open(&app)?;
    let proposals = store::list_proposals(&conn, &network_cid).map_err(|e| e.to_string())?;
    Ok(proposals
        .into_iter()
        .map(|p| ProposalInfo {
            id: p.id,
            title: p.title,
            description: p.description,
            options: p.options,
            created_by: p.created_by,
            created_at: p.created_at,
            closes_at: p.closes_at,
            quorum_percent: p.quorum_percent,
            status: p.status.to_string(),
        })
        .collect())
}

/// Create a new proposal (admin or any member, depending on network policy — open here).
#[tauri::command]
pub fn proposal_create(
    app: AppHandle,
    network_cid: String,
    title: String,
    description: String,
    options: Vec<String>,
    hours: u64,
    quorum_percent: u8,
) -> Result<ProposalInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let author_cid = keypair.cid();
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;

    if !network.data.members.iter().any(|m| m.cid_short == author_cid.short()) {
        return Err("you are not a member of this network".into());
    }
    if options.len() < 2 {
        return Err("at least 2 options are required".into());
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let closes_at = if hours == 0 { 0 } else { now + hours * 3600 };

    let proposal = Proposal::new(
        network_cid.clone(),
        title,
        description,
        options,
        author_cid.short().to_string(),
        now,
        closes_at,
        quorum_percent,
    );

    store::save_proposal(&conn, &network_cid, &proposal).map_err(|e| e.to_string())?;

    Ok(ProposalInfo {
        id: proposal.id,
        title: proposal.title,
        description: proposal.description,
        options: proposal.options,
        created_by: proposal.created_by,
        created_at: proposal.created_at,
        closes_at: proposal.closes_at,
        quorum_percent: proposal.quorum_percent,
        status: proposal.status.to_string(),
    })
}

/// Cast a vote on a proposal.
#[tauri::command]
pub fn vote_cast(
    app: AppHandle,
    network_cid: String,
    proposal_id: String,
    choice_index: usize,
) -> Result<(), String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let voter_cid = keypair.cid();

    let proposals = store::list_proposals(&conn, &network_cid).map_err(|e| e.to_string())?;
    let proposal = proposals
        .iter()
        .find(|p| p.id == proposal_id)
        .ok_or_else(|| format!("proposal '{}' not found", proposal_id))?;

    if proposal.status != ProposalStatus::Open {
        return Err(format!("proposal '{}' is not open", proposal_id));
    }
    if choice_index >= proposal.options.len() {
        return Err(format!(
            "choice {} out of range (0–{})",
            choice_index,
            proposal.options.len() - 1
        ));
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    if proposal.is_expired(now) {
        return Err(format!("proposal '{}' has expired", proposal_id));
    }

    let vote = Vote {
        proposal_id,
        voter_cid_short: voter_cid.short().to_string(),
        choice_index,
        cast_at: now,
    };
    store::save_vote(&conn, &vote).map_err(|e| e.to_string())?;
    Ok(())
}

/// Return vote results for a proposal.
#[tauri::command]
pub fn vote_results(
    app: AppHandle,
    network_cid: String,
    proposal_id: String,
) -> Result<VoteResultInfo, String> {
    let conn = open(&app)?;
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    let proposals = store::list_proposals(&conn, &network_cid).map_err(|e| e.to_string())?;
    let proposal = proposals
        .iter()
        .find(|p| p.id == proposal_id)
        .ok_or_else(|| format!("proposal '{}' not found", proposal_id))?;

    let votes = store::list_votes(&conn, &proposal_id).map_err(|e| e.to_string())?;
    let delegations = store::list_delegations(&conn, &network_cid).map_err(|e| e.to_string())?;
    let total_members = network.data.members.len();
    let result = compute_result_with_delegations(proposal, &votes, &delegations, total_members);

    Ok(VoteResultInfo {
        proposal_id: result.proposal_id,
        total_votes: result.total_votes,
        total_members: result.total_members,
        participation_percent: result.participation_percent,
        quorum_reached: result.quorum_reached,
        options: result
            .options
            .into_iter()
            .map(|o| OptionResult { label: o.label, votes: o.votes, percent: o.percent })
            .collect(),
        winner: result.winner,
    })
}

// ── Vote delegation ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct DelegationInfo {
    pub delegator_cid_short: String,
    pub delegate_cid_short: String,
    pub proposal_id: Option<String>,
    pub created_at: u64,
}

/// Set or replace a vote delegation (network-wide or per-proposal).
#[tauri::command]
pub fn vote_delegate(
    app: AppHandle,
    network_cid: String,
    delegate_cid_short: String,
    proposal_id: Option<String>,
) -> Result<DelegationInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let delegator_cid = keypair.cid();
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;

    if delegator_cid.short() == delegate_cid_short.as_str() {
        return Err("cannot delegate to yourself".into());
    }
    if !network.data.members.iter().any(|m| m.cid_short == delegate_cid_short) {
        return Err(format!("member '{}' not found in this network", delegate_cid_short));
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let delegation = VoteDelegation {
        delegator_cid_short: delegator_cid.short().to_string(),
        delegate_cid_short: delegate_cid_short.clone(),
        network_cid_short: network_cid.clone(),
        proposal_id: proposal_id.clone(),
        created_at: now,
    };
    store::save_delegation(&conn, &delegation).map_err(|e| e.to_string())?;
    Ok(DelegationInfo {
        delegator_cid_short: delegation.delegator_cid_short,
        delegate_cid_short: delegation.delegate_cid_short,
        proposal_id: delegation.proposal_id,
        created_at: delegation.created_at,
    })
}

/// Revoke a delegation (network-wide or per-proposal).
#[tauri::command]
pub fn vote_revoke_delegation(
    app: AppHandle,
    network_cid: String,
    proposal_id: Option<String>,
) -> Result<(), String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let delegator_cid = keypair.cid();
    store::delete_delegation(&conn, &network_cid, delegator_cid.short(), proposal_id.as_deref())
        .map_err(|e| e.to_string())
}

/// List my delegations for a network.
#[tauri::command]
pub fn vote_list_delegations(
    app: AppHandle,
    network_cid: String,
) -> Result<Vec<DelegationInfo>, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let my_cid = keypair.cid();
    let all = store::list_delegations(&conn, &network_cid).map_err(|e| e.to_string())?;
    Ok(all
        .into_iter()
        .filter(|d| d.delegator_cid_short == my_cid.short())
        .map(|d| DelegationInfo {
            delegator_cid_short: d.delegator_cid_short,
            delegate_cid_short: d.delegate_cid_short,
            proposal_id: d.proposal_id,
            created_at: d.created_at,
        })
        .collect())
}

// ── Admin actions (garde-fou) ─────────────────────────────────────────────────

#[derive(Serialize)]
pub struct AdminActionInfo {
    pub id: String,
    pub kind: String,
    pub taken_by: String,
    pub taken_at: u64,
    pub contest_window_secs: u64,
    pub contest_count: usize,
    pub status: String,
    pub suspended_proposal_id: Option<String>,
}

/// List admin actions for a network (most recent first).
#[tauri::command]
pub fn admin_action_list(app: AppHandle, network_cid: String) -> Result<Vec<AdminActionInfo>, String> {
    let conn = open(&app)?;
    let actions = store::list_admin_actions(&conn, &network_cid).map_err(|e| e.to_string())?;
    Ok(actions
        .into_iter()
        .map(|a| {
            let suspended_proposal_id = match &a.status {
                AdminActionStatus::Suspended { proposal_id } => Some(proposal_id.clone()),
                _ => None,
            };
            AdminActionInfo {
                id: a.id,
                kind: a.kind.to_string(),
                taken_by: a.taken_by,
                taken_at: a.taken_at,
                contest_window_secs: a.contest_window_secs,
                contest_count: a.contests.len(),
                status: a.status.to_string(),
                suspended_proposal_id,
            }
        })
        .collect())
}

/// Contest an admin action. If majority threshold is reached, suspends the action
/// and auto-creates a governance proposal.
#[tauri::command]
pub fn admin_action_contest(
    app: AppHandle,
    network_cid: String,
    action_id: String,
) -> Result<AdminActionInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let voter_cid = keypair.cid();
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;

    let mut actions = store::list_admin_actions(&conn, &network_cid).map_err(|e| e.to_string())?;
    let action = actions
        .iter_mut()
        .find(|a| a.id == action_id)
        .ok_or_else(|| format!("action '{}' not found", action_id))?;

    if action.status != AdminActionStatus::Active {
        return Err(format!("action '{}' is not contestable (status: {})", action_id, action.status));
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    if !action.is_window_open(now) {
        action.status = AdminActionStatus::Confirmed;
        store::save_admin_action(&conn, &network_cid, action).map_err(|e| e.to_string())?;
        return Err(format!("contest window has closed for action '{}'", action_id));
    }

    let total_members = network.data.members.len();
    let threshold_reached = add_contest(action, voter_cid.short(), total_members);

    if threshold_reached {
        let proposal = Proposal::new(
            network_cid.clone(),
            format!("Garde-fou : {}", action.kind),
            "La majorité a contesté une action de l'admin. Que décide le réseau ?".into(),
            vec!["Maintenir l'action".into(), "Annuler l'action".into()],
            "système".into(),
            now,
            now + 72 * 3600,
            0,
        );
        store::save_proposal(&conn, &network_cid, &proposal).map_err(|e| e.to_string())?;
        action.status = AdminActionStatus::Suspended { proposal_id: proposal.id.clone() };
    }

    store::save_admin_action(&conn, &network_cid, action).map_err(|e| e.to_string())?;

    let suspended_proposal_id = match &action.status {
        AdminActionStatus::Suspended { proposal_id } => Some(proposal_id.clone()),
        _ => None,
    };
    Ok(AdminActionInfo {
        id: action.id.clone(),
        kind: action.kind.to_string(),
        taken_by: action.taken_by.clone(),
        taken_at: action.taken_at,
        contest_window_secs: action.contest_window_secs,
        contest_count: action.contests.len(),
        status: action.status.to_string(),
        suspended_proposal_id,
    })
}

// ── Directory ─────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct DirectoryEntryInfo {
    pub id: String,
    pub directory_cid_short: String,
    pub kind: String,
    pub subject_cid_short: String,
    pub subject_name: String,
    pub description: String,
    pub contact_addr: Option<String>,
    pub published_by: String,
    pub published_at: u64,
    pub tags: Vec<String>,
    /// Set for results that come from a federated peer directory.
    pub source_dir_name: Option<String>,
}

impl From<DirectoryEntry> for DirectoryEntryInfo {
    fn from(e: DirectoryEntry) -> Self {
        Self {
            id: e.id,
            directory_cid_short: e.directory_cid_short,
            kind: e.kind.to_string(),
            subject_cid_short: e.subject_cid_short,
            subject_name: e.subject_name,
            description: e.description,
            contact_addr: e.contact_addr,
            published_by: e.published_by,
            published_at: e.published_at,
            tags: e.tags,
            source_dir_name: None,
        }
    }
}

#[derive(Serialize)]
pub struct FederationInfo {
    pub id: String,
    pub host_cid_short: String,
    pub peer_cid_short: String,
    pub peer_name: String,
    pub peer_addr: Option<String>,
    pub added_by: String,
    pub added_at: u64,
}

/// Create a directory network (kind = Directory).
#[tauri::command]
pub fn directory_create(
    app: AppHandle,
    name: String,
    display_name: String,
) -> Result<NetworkInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let admin_cid = keypair.cid();
    let mut network = Network::create(name, &admin_cid, display_name)
        .map_err(|e| e.to_string())?;
    network.data.kind = NetworkKind::Directory;
    store::save_network(&conn, &network).map_err(|e| e.to_string())?;
    Ok(NetworkInfo {
        cid_short: network.cid_short().to_string(),
        cid_full: network.cid_full().to_string(),
        name: network.name().to_string(),
        member_count: network.data.members.len(),
        is_directory: true,
        is_rrm: false,
    })
}

/// List all local directory networks.
#[tauri::command]
pub fn directory_list_networks(app: AppHandle) -> Result<Vec<NetworkInfo>, String> {
    let conn = open(&app)?;
    let networks = store::list_networks(&conn).map_err(|e| e.to_string())?;
    Ok(networks
        .into_iter()
        .filter(|n| n.data.kind == NetworkKind::Directory)
        .map(|n| NetworkInfo {
            cid_short: n.cid_short().to_string(),
            cid_full: n.cid_full().to_string(),
            name: n.name().to_string(),
            member_count: n.data.members.len(),
            is_directory: n.data.kind == NetworkKind::Directory,
            is_rrm: n.data.kind == NetworkKind::Rrm,
        })
        .collect())
}

/// Publish a network or member to a directory.
#[tauri::command]
pub fn directory_publish(
    app: AppHandle,
    directory_cid: String,
    kind: String,
    subject_cid_short: String,
    subject_name: String,
    description: String,
    contact_addr: Option<String>,
    tags: Vec<String>,
) -> Result<DirectoryEntryInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let publisher = keypair.cid().short().to_string();

    let dir_net = store::load_network(&conn, &directory_cid).map_err(|e| e.to_string())?;
    if dir_net.data.kind != NetworkKind::Directory {
        return Err(format!("network '{}' is not a directory", directory_cid));
    }

    let entry_kind: EntryKind = kind.parse().map_err(|e: String| e)?;
    let entry = DirectoryEntry::new(
        directory_cid,
        entry_kind,
        subject_cid_short,
        subject_name,
        description,
        contact_addr,
        publisher,
        tags,
    );
    store::save_directory_entry(&conn, &entry).map_err(|e| e.to_string())?;
    Ok(DirectoryEntryInfo::from(entry))
}

/// List all entries in a directory.
#[tauri::command]
pub fn directory_list(
    app: AppHandle,
    directory_cid: String,
) -> Result<Vec<DirectoryEntryInfo>, String> {
    let conn = open(&app)?;
    let entries = store::list_directory_entries(&conn, &directory_cid).map_err(|e| e.to_string())?;
    Ok(entries.into_iter().map(DirectoryEntryInfo::from).collect())
}

/// Search entries in a directory by free-text query.
/// When include_federated is true, also searches entries from all federated peer directories.
#[tauri::command]
pub fn directory_search(
    app: AppHandle,
    directory_cid: String,
    query: String,
    include_federated: bool,
) -> Result<Vec<DirectoryEntryInfo>, String> {
    let conn = open(&app)?;
    let mut results: Vec<DirectoryEntryInfo> = store::search_directory_entries(&conn, &directory_cid, &query)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(DirectoryEntryInfo::from)
        .collect();

    if include_federated {
        let feds = store::list_federations(&conn, &directory_cid).map_err(|e| e.to_string())?;
        for fed in feds {
            let peer_entries = store::search_directory_entries(&conn, &fed.peer_cid_short, &query)
                .unwrap_or_default();
            for entry in peer_entries {
                let mut info = DirectoryEntryInfo::from(entry);
                info.source_dir_name = Some(fed.peer_name.clone());
                results.push(info);
            }
        }
    }

    Ok(results)
}

/// Remove an entry from a directory.
#[tauri::command]
pub fn directory_remove(
    app: AppHandle,
    directory_cid: String,
    entry_id: String,
) -> Result<(), String> {
    let conn = open(&app)?;
    store::delete_directory_entry(&conn, &directory_cid, &entry_id).map_err(|e| e.to_string())
}

// ── Directory federations ─────────────────────────────────────────────────────

/// Add a federation link from this directory to a peer directory.
#[tauri::command]
pub fn directory_federate(
    app: AppHandle,
    directory_cid: String,
    peer_cid: String,
    peer_name: String,
    peer_addr: Option<String>,
) -> Result<FederationInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let dir_net = store::load_network(&conn, &directory_cid).map_err(|e| e.to_string())?;
    if dir_net.data.kind != NetworkKind::Directory {
        return Err(format!("network '{}' is not a directory", directory_cid));
    }
    let fed = FederatedDirectory::new(
        directory_cid,
        peer_cid,
        peer_name,
        peer_addr,
        keypair.cid().short().to_string(),
    );
    store::save_federation(&conn, &fed).map_err(|e| e.to_string())?;
    Ok(FederationInfo {
        id: fed.id,
        host_cid_short: fed.host_cid_short,
        peer_cid_short: fed.peer_cid_short,
        peer_name: fed.peer_name,
        peer_addr: fed.peer_addr,
        added_by: fed.added_by,
        added_at: fed.added_at,
    })
}

/// Remove a federation link.
#[tauri::command]
pub fn directory_unfederate(
    app: AppHandle,
    directory_cid: String,
    peer_cid: String,
) -> Result<(), String> {
    let conn = open(&app)?;
    store::delete_federation(&conn, &directory_cid, &peer_cid).map_err(|e| e.to_string())
}

/// List all federation links for a directory.
#[tauri::command]
pub fn directory_federations(
    app: AppHandle,
    directory_cid: String,
) -> Result<Vec<FederationInfo>, String> {
    let conn = open(&app)?;
    let feds = store::list_federations(&conn, &directory_cid).map_err(|e| e.to_string())?;
    Ok(feds
        .into_iter()
        .map(|f| FederationInfo {
            id: f.id,
            host_cid_short: f.host_cid_short,
            peer_cid_short: f.peer_cid_short,
            peer_name: f.peer_name,
            peer_addr: f.peer_addr,
            added_by: f.added_by,
            added_at: f.added_at,
        })
        .collect())
}

// ── RRM ───────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct RrmEntryInfo {
    pub id: String,
    pub rrm_cid_short: String,
    pub network_cid_short: String,
    pub network_name: String,
    pub reason: String,
    pub evidence_url: Option<String>,
    pub reported_by: String,
    pub reported_at: u64,
}

impl From<RrmEntry> for RrmEntryInfo {
    fn from(e: RrmEntry) -> Self {
        Self {
            id: e.id,
            rrm_cid_short: e.rrm_cid_short,
            network_cid_short: e.network_cid_short,
            network_name: e.network_name,
            reason: e.reason,
            evidence_url: e.evidence_url,
            reported_by: e.reported_by,
            reported_at: e.reported_at,
        }
    }
}

#[derive(Serialize)]
pub struct TrustedRrmInfo {
    pub id: String,
    pub network_cid_short: String,
    pub rrm_cid_short: String,
    pub rrm_name: String,
    pub added_by: String,
    pub added_at: u64,
}

#[derive(Serialize)]
pub struct RrmWarning {
    pub rrm_name: String,
    pub rrm_cid_short: String,
    pub network_name: String,
    pub reason: String,
    pub evidence_url: Option<String>,
}

/// Create an RRM network (kind = Rrm).
#[tauri::command]
pub fn rrm_create(app: AppHandle, name: String, display_name: String) -> Result<NetworkInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let admin_cid = keypair.cid();
    let mut network = Network::create(name, &admin_cid, display_name).map_err(|e| e.to_string())?;
    network.data.kind = NetworkKind::Rrm;
    store::save_network(&conn, &network).map_err(|e| e.to_string())?;
    Ok(NetworkInfo {
        cid_short: network.cid_short().to_string(),
        cid_full: network.cid_full().to_string(),
        name: network.name().to_string(),
        member_count: network.data.members.len(),
        is_directory: false,
        is_rrm: true,
    })
}

/// Report a network to an RRM.
#[tauri::command]
pub fn rrm_report(
    app: AppHandle,
    rrm_cid: String,
    network_cid_short: String,
    network_name: String,
    reason: String,
    evidence_url: Option<String>,
) -> Result<RrmEntryInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let reporter = keypair.cid().short().to_string();

    let rrm_net = store::load_network(&conn, &rrm_cid).map_err(|e| e.to_string())?;
    if rrm_net.data.kind != NetworkKind::Rrm {
        return Err(format!("network '{}' is not an RRM", rrm_cid));
    }

    let entry = RrmEntry::new(rrm_cid, network_cid_short, network_name, reason, evidence_url, reporter);
    store::save_rrm_entry(&conn, &entry).map_err(|e| e.to_string())?;
    Ok(RrmEntryInfo::from(entry))
}

/// List all reports in an RRM.
#[tauri::command]
pub fn rrm_list(app: AppHandle, rrm_cid: String) -> Result<Vec<RrmEntryInfo>, String> {
    let conn = open(&app)?;
    let entries = store::list_rrm_entries(&conn, &rrm_cid).map_err(|e| e.to_string())?;
    Ok(entries.into_iter().map(RrmEntryInfo::from).collect())
}

/// Remove a report from an RRM.
#[tauri::command]
pub fn rrm_remove(app: AppHandle, rrm_cid: String, entry_id: String) -> Result<(), String> {
    let conn = open(&app)?;
    store::delete_rrm_entry(&conn, &rrm_cid, &entry_id).map_err(|e| e.to_string())
}

/// Trust an RRM — this network will consult it on connection checks.
#[tauri::command]
pub fn network_trust_rrm(
    app: AppHandle,
    network_cid: String,
    rrm_cid: String,
    rrm_name: String,
) -> Result<TrustedRrmInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let rrm_net = store::load_network(&conn, &rrm_cid).map_err(|e| e.to_string())?;
    if rrm_net.data.kind != NetworkKind::Rrm {
        return Err(format!("network '{}' is not an RRM", rrm_cid));
    }
    let trust = TrustedRrm::new(
        network_cid,
        rrm_cid,
        rrm_name,
        keypair.cid().short().to_string(),
    );
    store::save_trusted_rrm(&conn, &trust).map_err(|e| e.to_string())?;
    Ok(TrustedRrmInfo {
        id: trust.id,
        network_cid_short: trust.network_cid_short,
        rrm_cid_short: trust.rrm_cid_short,
        rrm_name: trust.rrm_name,
        added_by: trust.added_by,
        added_at: trust.added_at,
    })
}

/// Stop trusting an RRM.
#[tauri::command]
pub fn network_untrust_rrm(app: AppHandle, network_cid: String, rrm_cid: String) -> Result<(), String> {
    let conn = open(&app)?;
    store::delete_trusted_rrm(&conn, &network_cid, &rrm_cid).map_err(|e| e.to_string())
}

/// List all RRMs trusted by a network.
#[tauri::command]
pub fn network_trusted_rrms(app: AppHandle, network_cid: String) -> Result<Vec<TrustedRrmInfo>, String> {
    let conn = open(&app)?;
    let trusts = store::list_trusted_rrms(&conn, &network_cid).map_err(|e| e.to_string())?;
    Ok(trusts.into_iter().map(|t| TrustedRrmInfo {
        id: t.id,
        network_cid_short: t.network_cid_short,
        rrm_cid_short: t.rrm_cid_short,
        rrm_name: t.rrm_name,
        added_by: t.added_by,
        added_at: t.added_at,
    }).collect())
}

/// Check if a peer network is listed in any trusted RRM. Returns warnings if found.
#[tauri::command]
pub fn rrm_check(
    app: AppHandle,
    network_cid: String,
    peer_cid: String,
) -> Result<Vec<RrmWarning>, String> {
    let conn = open(&app)?;
    let warnings = store::check_rrm_warnings(&conn, &network_cid, &peer_cid)
        .map_err(|e| e.to_string())?;
    Ok(warnings.into_iter().map(|(trust, entry)| RrmWarning {
        rrm_name: trust.rrm_name,
        rrm_cid_short: trust.rrm_cid_short,
        network_name: entry.network_name,
        reason: entry.reason,
        evidence_url: entry.evidence_url,
    }).collect())
}

// ── Minor / Guardian ──────────────────────────────────────────────────────────

/// Mark or unmark a network member as a minor (admin only).
#[tauri::command]
pub fn member_set_minor(
    app: AppHandle,
    network_cid: String,
    member_cid: String,
    is_minor: bool,
) -> Result<(), String> {
    let conn = open(&app)?;
    store::set_member_minor(&conn, &network_cid, &member_cid, is_minor)
        .map_err(|e| e.to_string())
}

/// Add a guardian–minor link.
#[tauri::command]
pub fn member_set_guardian(
    app: AppHandle,
    network_cid: String,
    minor_cid: String,
    guardian_cid: String,
) -> Result<GuardianLinkInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let link = GuardianLink::new(network_cid, minor_cid, guardian_cid, keypair.cid().short().to_string());
    store::save_guardian_link(&conn, &link).map_err(|e| e.to_string())?;
    Ok(GuardianLinkInfo {
        id: link.id,
        network_cid_short: link.network_cid_short,
        minor_cid_short: link.minor_cid_short,
        guardian_cid_short: link.guardian_cid_short,
        added_by: link.added_by,
        added_at: link.added_at,
    })
}

/// Remove a guardian–minor link.
#[tauri::command]
pub fn member_remove_guardian(
    app: AppHandle,
    network_cid: String,
    minor_cid: String,
    guardian_cid: String,
) -> Result<(), String> {
    let conn = open(&app)?;
    store::delete_guardian_link(&conn, &network_cid, &minor_cid, &guardian_cid)
        .map_err(|e| e.to_string())
}

/// List all guardians of a minor member.
#[tauri::command]
pub fn member_guardians(
    app: AppHandle,
    network_cid: String,
    minor_cid: String,
) -> Result<Vec<GuardianLinkInfo>, String> {
    let conn = open(&app)?;
    let links = store::list_guardians(&conn, &network_cid, &minor_cid)
        .map_err(|e| e.to_string())?;
    Ok(links.into_iter().map(|l| GuardianLinkInfo {
        id: l.id,
        network_cid_short: l.network_cid_short,
        minor_cid_short: l.minor_cid_short,
        guardian_cid_short: l.guardian_cid_short,
        added_by: l.added_by,
        added_at: l.added_at,
    }).collect())
}

/// List all minors for which a given member is guardian.
#[tauri::command]
pub fn member_wards(
    app: AppHandle,
    network_cid: String,
    guardian_cid: String,
) -> Result<Vec<GuardianLinkInfo>, String> {
    let conn = open(&app)?;
    let links = store::list_wards(&conn, &network_cid, &guardian_cid)
        .map_err(|e| e.to_string())?;
    Ok(links.into_iter().map(|l| GuardianLinkInfo {
        id: l.id,
        network_cid_short: l.network_cid_short,
        minor_cid_short: l.minor_cid_short,
        guardian_cid_short: l.guardian_cid_short,
        added_by: l.added_by,
        added_at: l.added_at,
    }).collect())
}

/// Set interaction restrictions for a minor member.
#[tauri::command]
pub fn member_set_restrictions(
    app: AppHandle,
    network_cid: String,
    minor_cid: String,
    max_circle: u8,
    allowed_cid_shorts: Vec<String>,
) -> Result<MinorRestrictionsInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let r = MinorRestrictions::new(
        network_cid,
        minor_cid,
        max_circle,
        allowed_cid_shorts,
        keypair.cid().short().to_string(),
    );
    store::save_minor_restrictions(&conn, &r).map_err(|e| e.to_string())?;
    Ok(MinorRestrictionsInfo {
        network_cid_short: r.network_cid_short,
        minor_cid_short: r.minor_cid_short,
        max_circle: r.max_circle,
        allowed_cid_shorts: r.allowed_cid_shorts,
        updated_by: r.updated_by,
        updated_at: r.updated_at,
    })
}

/// Get interaction restrictions for a minor member (returns null if not set).
#[tauri::command]
pub fn member_get_restrictions(
    app: AppHandle,
    network_cid: String,
    minor_cid: String,
) -> Result<Option<MinorRestrictionsInfo>, String> {
    let conn = open(&app)?;
    let r = store::get_minor_restrictions(&conn, &network_cid, &minor_cid)
        .map_err(|e| e.to_string())?;
    Ok(r.map(|r| MinorRestrictionsInfo {
        network_cid_short: r.network_cid_short,
        minor_cid_short: r.minor_cid_short,
        max_circle: r.max_circle,
        allowed_cid_shorts: r.allowed_cid_shorts,
        updated_by: r.updated_by,
        updated_at: r.updated_at,
    }))
}

// ── Plugin commands ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub permissions: Vec<String>,
    pub is_system: bool,
    pub state: String,
    pub installed_at: u64,
}

/// List all installed plugins.
#[tauri::command]
pub fn plugin_list(app: AppHandle) -> Result<Vec<PluginInfo>, String> {
    let conn = open(&app)?;
    let records = store::list_plugins(&conn).map_err(|e| e.to_string())?;
    Ok(records.into_iter().map(|r| PluginInfo {
        id: r.manifest.id,
        name: r.manifest.name,
        version: r.manifest.version,
        description: r.manifest.description,
        author: r.manifest.author,
        permissions: r.manifest.permissions.iter().map(|p| p.to_string()).collect(),
        is_system: r.manifest.is_system,
        state: r.state.to_string(),
        installed_at: r.installed_at,
    }).collect())
}

/// Enable a plugin by ID.
#[tauri::command]
pub fn plugin_enable(app: AppHandle, plugin_id: String) -> Result<(), String> {
    let conn = open(&app)?;
    store::set_plugin_state(&conn, &plugin_id, PluginState::Enabled)
        .map_err(|e| e.to_string())
}

/// Disable a plugin by ID (system plugins cannot be disabled).
#[tauri::command]
pub fn plugin_disable(app: AppHandle, plugin_id: String) -> Result<(), String> {
    let conn = open(&app)?;
    store::set_plugin_state(&conn, &plugin_id, PluginState::Disabled)
        .map_err(|e| e.to_string())
}
