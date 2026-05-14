use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use civium_core::{
    network::{Invitation, Network},
    add_contest, compute_result_with_delegations,
    AdminAction, AdminActionKind, AdminActionStatus,
    CiviumKeypair, CiviumNode, CiviumRequest, CiviumResponse,
    GroupKey, MemberRole, Message, MessageKind, Multiaddr,
    NodeCommand, NodeConfig, NodeEvent, Proposal, ProposalStatus, TrustCircle, Vote, VoteDelegation,
    peer_id_from_multiaddr,
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
}

#[derive(Serialize)]
pub struct MemberInfo {
    pub cid_short: String,
    pub display_name: String,
    pub circle: u8,
    pub role: String,
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
