use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use civium_core::{
    network::{Invitation, Network},
    add_contest, compute_result_with_delegations,
    ActivityKind, AdminAction, AdminActionKind, AdminActionStatus, AgendaEvent,
    CiviumKeypair, CiviumNode, CiviumRequest, CiviumResponse,
    DirectoryEntry, Document, EntryKind, FederatedDirectory, GroupKey, GuardianLink, MemberRole, Message, MessageKind,
    MinorRestrictions, Multiaddr, NetworkKind, NodeCommand, NodeConfig, NodeEvent, PairedDevice, PairKey, PluginState, Proposal, ProposalStatus,
    RrmEntry, TrustedRrm, TrustCircle, Vote, VoteDelegation, complete_pairing, init_pairing, peer_id_from_multiaddr,
    RCC_URL,
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

    let network = Network::create(name, &admin_cid, display_name, Some(keypair.pub_key_b58()))
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
        .submit_join_request(&member_cid, display_name, &invitation, Some(keypair.pub_key_b58()))
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
    let _ = store::emit_activity(&conn, &network_cid, ActivityKind::MemberJoined,
        &record.cid_short,
        &format!("{} a rejoint le réseau", record.display_name));

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
        mcp_port: None,
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
    pub is_e2e: bool,
}

/// Return decrypted thread messages for a network, ordered by sent_at.
#[tauri::command]
pub fn message_list(app: AppHandle, network_cid: String) -> Result<Vec<MessageDisplay>, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    let group_key =
        GroupKey::from_b58(&network.data.group_key_b58).map_err(|e| e.to_string())?;

    // Index member names and public keys for display + E2E decryption
    let member_names: HashMap<String, String> = network
        .data
        .members
        .iter()
        .map(|m| (m.cid_short.clone(), m.display_name.clone()))
        .collect();
    let member_pubkeys: HashMap<String, String> = network
        .data
        .members
        .iter()
        .filter_map(|m| m.pub_key_b58.as_ref().map(|k| (m.cid_full.clone(), k.clone())))
        .collect();

    let messages = store::load_messages(&conn, &network_cid).map_err(|e| e.to_string())?;

    let mut result = Vec::with_capacity(messages.len());
    for msg in messages {
        let (body, is_direct, is_e2e, to_cid_short) = match &msg.kind {
            MessageKind::Thread => {
                let body = group_key
                    .decrypt(&msg.nonce_b58, &msg.ciphertext_b58)
                    .map(|b| String::from_utf8_lossy(&b).into_owned())
                    .unwrap_or_else(|_| "[message illisible]".into());
                (body, false, false, None)
            }
            MessageKind::Direct { to_cid_short } => {
                let body = group_key
                    .decrypt(&msg.nonce_b58, &msg.ciphertext_b58)
                    .map(|b| String::from_utf8_lossy(&b).into_owned())
                    .unwrap_or_else(|_| "[message illisible]".into());
                (body, true, false, Some(to_cid_short.clone()))
            }
            MessageKind::E2E { to_cid_full } => {
                // Derive pair key: our secret + the other party's pubkey
                let my_cid_full = keypair.cid().full().to_string();
                let peer_cid_full = if msg.author_cid_short == keypair.cid().short() {
                    to_cid_full.clone()
                } else {
                    my_cid_full.clone()
                };
                let body = member_pubkeys
                    .get(&peer_cid_full)
                    .and_then(|pk_b58| {
                        let pk_bytes = bs58::decode(pk_b58).into_vec().ok()?;
                        let pk_arr: [u8; 32] = pk_bytes.try_into().ok()?;
                        let pair_key = PairKey::derive(keypair.secret_bytes(), &pk_arr).ok()?;
                        let plain = pair_key.decrypt(&msg.nonce_b58, &msg.ciphertext_b58).ok()?;
                        String::from_utf8(plain).ok()
                    })
                    .unwrap_or_else(|| "[message E2E — clé introuvable]".into());
                // to_cid_short: resolve from cid_full
                let to_short = network.data.members.iter()
                    .find(|m| &m.cid_full == to_cid_full)
                    .map(|m| m.cid_short.clone());
                (body, true, true, to_short)
            }
        };

        let author_name = member_names
            .get(&msg.author_cid_short)
            .cloned()
            .unwrap_or_else(|| msg.author_cid_short.clone());

        result.push(MessageDisplay {
            id: msg.id,
            author_cid_short: msg.author_cid_short,
            author_name,
            body,
            sent_at: msg.sent_at,
            is_direct,
            to_cid_short,
            is_e2e,
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
    let _ = store::enqueue_outbox(&conn, &network_cid, &msg.id);

    let author_name = network
        .data
        .members
        .iter()
        .find(|m| m.cid_short == author_cid.short())
        .map(|m| m.display_name.clone())
        .unwrap_or_else(|| author_cid.short().to_string());

    let _ = store::emit_activity(&conn, &network_cid, ActivityKind::MessagePosted,
        author_cid.short(),
        &format!("{} a publié un message", author_name));

    Ok(MessageDisplay {
        id: msg.id,
        author_cid_short: msg.author_cid_short,
        author_name,
        body,
        sent_at: msg.sent_at,
        is_direct: false,
        to_cid_short: None,
        is_e2e: false,
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
    let _ = store::enqueue_outbox(&conn, &network_cid, &msg.id);

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
        is_e2e: false,
    })
}

/// Send a true end-to-end encrypted message (Cercle 3 — Intime).
/// Only the sender and the recipient can decrypt. Uses X25519 DH pair key.
#[tauri::command]
pub fn message_send_e2e(
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
        .ok_or_else(|| "vous n'êtes pas membre de ce réseau".to_string())?;

    let recipient = network.data.members.iter()
        .find(|m| m.cid_short == to_cid_short)
        .ok_or_else(|| format!("membre '{}' introuvable dans ce réseau", to_cid_short))?;

    let recipient_cid_full = recipient.cid_full.clone();

    let pk_b58 = recipient.pub_key_b58.as_ref()
        .ok_or_else(|| format!("clé publique de '{}' inconnue — mise à jour requise", to_cid_short))?;
    let pk_bytes = bs58::decode(pk_b58)
        .into_vec()
        .map_err(|e| format!("clé publique invalide : {e}"))?;
    let pk_arr: [u8; 32] = pk_bytes.try_into()
        .map_err(|_| "la clé publique doit faire 32 octets".to_string())?;

    let pair_key = PairKey::derive(keypair.secret_bytes(), &pk_arr)
        .map_err(|e| e.to_string())?;
    let (nonce_b58, ciphertext_b58) = pair_key.encrypt(body.as_bytes())
        .map_err(|e| e.to_string())?;

    let sent_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let msg = Message {
        id: nonce_b58.clone(),
        author_cid_short: author_cid_short.clone(),
        kind: MessageKind::E2E { to_cid_full: recipient_cid_full },
        nonce_b58,
        ciphertext_b58,
        sent_at,
    };
    store::save_message(&conn, &network_cid, &msg).map_err(|e| e.to_string())?;
    let _ = store::enqueue_outbox(&conn, &network_cid, &msg.id);

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
        is_e2e: true,
    })
}

// ── Outbox ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct OutboxCountInfo {
    pub network_cid_short: String,
    pub count: u64,
}

#[tauri::command]
pub fn outbox_count_all(app: AppHandle) -> Result<Vec<OutboxCountInfo>, String> {
    let conn = open(&app)?;
    let counts = store::count_all_outbox(&conn);
    Ok(counts.into_iter().map(|(cid, count)| OutboxCountInfo { network_cid_short: cid, count }).collect())
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
    let _ = store::emit_activity(&conn, &network_cid, ActivityKind::ProposalCreated,
        author_cid.short(),
        &format!("Proposition créée : « {} »", proposal.title));

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
    let option_label = proposal.options.get(choice_index).map(|s| s.as_str()).unwrap_or("?");
    let _ = store::emit_activity(&conn, &network_cid, ActivityKind::VoteCast,
        voter_cid.short(),
        &format!("Vote exprimé : « {} »", option_label));
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
    let mut network = Network::create(name, &admin_cid, display_name, Some(keypair.pub_key_b58()))
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
    let mut network = Network::create(name, &admin_cid, display_name, Some(keypair.pub_key_b58())).map_err(|e| e.to_string())?;
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

// ── Agenda ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct AgendaEventInfo {
    pub id: String,
    pub network_cid_short: String,
    pub title: String,
    pub description: String,
    pub start_at: u64,
    pub end_at: Option<u64>,
    pub location: Option<String>,
    pub created_by: String,
    pub created_at: u64,
    pub updated_at: u64,
}

fn event_to_info(e: AgendaEvent) -> AgendaEventInfo {
    AgendaEventInfo {
        id: e.id,
        network_cid_short: e.network_cid_short,
        title: e.title,
        description: e.description,
        start_at: e.start_at,
        end_at: e.end_at,
        location: e.location,
        created_by: e.created_by,
        created_at: e.created_at,
        updated_at: e.updated_at,
    }
}

/// Create a new agenda event.
#[tauri::command]
pub fn agenda_create(
    app: AppHandle,
    network_cid_short: String,
    title: String,
    description: String,
    start_at: u64,
    end_at: Option<u64>,
    location: Option<String>,
) -> Result<AgendaEventInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let created_by = keypair.cid().short().to_string();
    let event = AgendaEvent::new(
        network_cid_short,
        title,
        description,
        start_at,
        end_at,
        location,
        created_by,
    );
    store::save_agenda_event(&conn, &event).map_err(|e| e.to_string())?;
    let _ = store::emit_activity(&conn, &event.network_cid_short, ActivityKind::AgendaEventCreated,
        &event.created_by,
        &format!("Événement créé : « {} »", event.title));
    Ok(event_to_info(event))
}

/// List agenda events for a network.
#[tauri::command]
pub fn agenda_list(app: AppHandle, network_cid_short: String) -> Result<Vec<AgendaEventInfo>, String> {
    let conn = open(&app)?;
    let events = store::list_agenda_events(&conn, &network_cid_short)
        .map_err(|e| e.to_string())?;
    Ok(events.into_iter().map(event_to_info).collect())
}

/// Update an existing agenda event (title, description, start_at, end_at, location).
#[tauri::command]
pub fn agenda_update(
    app: AppHandle,
    network_cid_short: String,
    event_id: String,
    title: String,
    description: String,
    start_at: u64,
    end_at: Option<u64>,
    location: Option<String>,
) -> Result<AgendaEventInfo, String> {
    let conn = open(&app)?;
    let mut event = store::get_agenda_event(&conn, &network_cid_short, &event_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("event '{}' not found", event_id))?;
    event.title = title;
    event.description = description;
    event.start_at = start_at;
    event.end_at = end_at;
    event.location = location;
    event.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    store::save_agenda_event(&conn, &event).map_err(|e| e.to_string())?;
    Ok(event_to_info(event))
}

/// Delete an agenda event.
#[tauri::command]
pub fn agenda_delete(
    app: AppHandle,
    network_cid_short: String,
    event_id: String,
) -> Result<(), String> {
    let conn = open(&app)?;
    store::delete_agenda_event(&conn, &network_cid_short, &event_id)
        .map_err(|e| e.to_string())
}

// ── Activity & Notifications ──────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ActivityEventInfo {
    pub id: String,
    pub network_cid_short: String,
    pub kind: String,
    pub actor_cid_short: String,
    pub summary: String,
    pub occurred_at: u64,
}

#[derive(Serialize)]
pub struct NotificationInfo {
    pub id: String,
    pub network_cid_short: String,
    pub source_event_id: String,
    pub target_cid_short: String,
    pub read: bool,
    pub created_at: u64,
}

/// List the last 100 activity events for a network.
#[tauri::command]
pub fn activity_list(app: AppHandle, network_cid_short: String) -> Result<Vec<ActivityEventInfo>, String> {
    let conn = open(&app)?;
    let events = store::list_activity(&conn, &network_cid_short).map_err(|e| e.to_string())?;
    Ok(events.into_iter().map(|e| ActivityEventInfo {
        id: e.id,
        network_cid_short: e.network_cid_short,
        kind: e.kind.to_string(),
        actor_cid_short: e.actor_cid_short,
        summary: e.summary,
        occurred_at: e.occurred_at,
    }).collect())
}

/// List notifications for the current identity in a network.
#[tauri::command]
pub fn notification_list(app: AppHandle, network_cid_short: String) -> Result<Vec<NotificationInfo>, String> {
    let conn = open(&app)?;
    let notifs = store::list_notifications(&conn, &network_cid_short).map_err(|e| e.to_string())?;
    Ok(notifs.into_iter().map(|n| NotificationInfo {
        id: n.id,
        network_cid_short: n.network_cid_short,
        source_event_id: n.source_event_id,
        target_cid_short: n.target_cid_short,
        read: n.read,
        created_at: n.created_at,
    }).collect())
}

/// Count unread notifications for a network (used for badge).
#[tauri::command]
pub fn notification_unread_count(app: AppHandle, network_cid_short: String) -> usize {
    let Ok(conn) = open(&app) else { return 0 };
    store::count_unread_notifications(&conn, &network_cid_short)
}

/// Mark a notification as read.
#[tauri::command]
pub fn notification_mark_read(app: AppHandle, notif_id: String) -> Result<(), String> {
    let conn = open(&app)?;
    store::mark_notification_read(&conn, &notif_id).map_err(|e| e.to_string())
}

// ── Documents ─────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct DocumentInfo {
    pub id: String,
    pub network_cid_short: String,
    pub title: String,
    pub body: String,
    pub version: u32,
    pub created_by: String,
    pub created_at: u64,
    pub updated_at: u64,
}

fn doc_to_info(doc: Document, group_key: &GroupKey) -> DocumentInfo {
    let body = group_key
        .decrypt(&doc.nonce_b58, &doc.body_ciphertext)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_else(|_| "[contenu illisible]".into());
    DocumentInfo {
        id: doc.id,
        network_cid_short: doc.network_cid_short,
        title: doc.title,
        body,
        version: doc.version,
        created_by: doc.created_by,
        created_at: doc.created_at,
        updated_at: doc.updated_at,
    }
}

/// Encrypt and store a new document in the network.
#[tauri::command]
pub fn document_create(
    app: AppHandle,
    network_cid_short: String,
    title: String,
    body: String,
) -> Result<DocumentInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let network = store::load_network(&conn, &network_cid_short).map_err(|e| e.to_string())?;
    let group_key = GroupKey::from_b58(&network.data.group_key_b58).map_err(|e| e.to_string())?;
    let (nonce_b58, body_ciphertext) = group_key.encrypt(body.as_bytes()).map_err(|e| e.to_string())?;
    let doc = Document::new(
        network_cid_short,
        title,
        nonce_b58,
        body_ciphertext,
        keypair.cid().short().to_string(),
    );
    store::save_document(&conn, &doc).map_err(|e| e.to_string())?;
    let _ = store::emit_activity(&conn, &doc.network_cid_short, ActivityKind::DocumentCreated,
        &doc.created_by,
        &format!("Document créé : « {} »", doc.title));
    Ok(doc_to_info(doc, &group_key))
}

/// List documents for a network (decrypted).
#[tauri::command]
pub fn document_list(app: AppHandle, network_cid_short: String) -> Result<Vec<DocumentInfo>, String> {
    let conn = open(&app)?;
    let network = store::load_network(&conn, &network_cid_short).map_err(|e| e.to_string())?;
    let group_key = GroupKey::from_b58(&network.data.group_key_b58).map_err(|e| e.to_string())?;
    let docs = store::list_documents(&conn, &network_cid_short).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(|d| doc_to_info(d, &group_key)).collect())
}

/// Update the title and/or body of an existing document.
#[tauri::command]
pub fn document_update(
    app: AppHandle,
    network_cid_short: String,
    doc_id: String,
    title: String,
    body: String,
) -> Result<DocumentInfo, String> {
    let conn = open(&app)?;
    let network = store::load_network(&conn, &network_cid_short).map_err(|e| e.to_string())?;
    let group_key = GroupKey::from_b58(&network.data.group_key_b58).map_err(|e| e.to_string())?;
    let mut doc = store::get_document(&conn, &network_cid_short, &doc_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("document '{doc_id}' introuvable"))?;
    let (nonce_b58, body_ciphertext) = group_key.encrypt(body.as_bytes()).map_err(|e| e.to_string())?;
    doc.title = title;
    doc.nonce_b58 = nonce_b58;
    doc.body_ciphertext = body_ciphertext;
    doc.version += 1;
    doc.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    store::save_document(&conn, &doc).map_err(|e| e.to_string())?;
    Ok(doc_to_info(doc, &group_key))
}

/// Delete a document.
#[tauri::command]
pub fn document_delete(
    app: AppHandle,
    network_cid_short: String,
    doc_id: String,
) -> Result<(), String> {
    let conn = open(&app)?;
    store::delete_document(&conn, &network_cid_short, &doc_id).map_err(|e| e.to_string())
}

// ── MCP server ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct McpStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub token: Option<String>,
    pub url: Option<String>,
}

/// Start the MCP HTTP server on the given port (default 7523).
#[tauri::command]
pub fn mcp_start(app: AppHandle, port: Option<u16>) -> Result<McpStatus, String> {
    let state = app.state::<crate::node::AppState>();
    {
        let running = state.mcp_shutdown.lock().unwrap();
        if running.is_some() {
            return Err("Le serveur MCP est déjà en cours d'exécution.".to_string());
        }
    }

    let port = port.unwrap_or(7523);
    let token = crate::mcp::generate_token();
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let data_dir = app.path().app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("./civium-data"));
    let token_clone = token.clone();

    tauri::async_runtime::spawn(async move {
        crate::mcp::run_mcp_server(data_dir, port, token_clone, rx).await;
    });

    *state.mcp_shutdown.lock().unwrap() = Some(tx);
    *state.mcp_token.lock().unwrap() = Some(token.clone());
    *state.mcp_port.lock().unwrap() = Some(port);

    Ok(McpStatus {
        running: true,
        port: Some(port),
        token: Some(token.clone()),
        url: Some(format!("http://127.0.0.1:{port}")),
    })
}

/// Stop the running MCP server.
#[tauri::command]
pub fn mcp_stop(app: AppHandle) -> Result<(), String> {
    let state = app.state::<crate::node::AppState>();
    let tx = state.mcp_shutdown.lock().unwrap().take();
    match tx {
        Some(sender) => {
            let _ = sender.send(());
            *state.mcp_token.lock().unwrap() = None;
            *state.mcp_port.lock().unwrap() = None;
            Ok(())
        }
        None => Err("Le serveur MCP n'est pas en cours d'exécution.".to_string()),
    }
}

/// Return the current MCP server status (running, port, token).
#[tauri::command]
pub fn mcp_status(app: AppHandle) -> McpStatus {
    let state = app.state::<crate::node::AppState>();
    let running = state.mcp_shutdown.lock().unwrap().is_some();
    let port = *state.mcp_port.lock().unwrap();
    let token = state.mcp_token.lock().unwrap().clone();
    McpStatus {
        running,
        url: port.map(|p| format!("http://127.0.0.1:{p}")),
        port,
        token,
    }
}

// ── RCC — Registre Central Civium ────────────────────────────────────────────

#[derive(Serialize)]
pub struct RccStatusInfo {
    pub network_cid_short: String,
    pub network_name: String,
    pub admin_email: String,
    /// "pending" | "registered" | "failed"
    pub status: String,
    pub attempts: u32,
    pub last_attempt: Option<u64>,
    pub registered_at: u64,
    pub rcc_url: String,
}

/// Initialise l'enregistrement RCC pour un réseau et lance la tentative en arrière-plan.
/// Idempotent : si déjà enregistré, retourne le statut existant sans relancer.
#[tauri::command]
pub fn rcc_register(
    app: AppHandle,
    network_cid: String,
    admin_email: String,
) -> Result<RccStatusInfo, String> {
    let conn = open(&app)?;
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;

    // Si déjà enregistré, renvoyer le statut existant
    if let Some(existing) = store::get_rcc_registration(&conn, &network.cid_short().to_string()) {
        if existing.status == "registered" {
            return Ok(RccStatusInfo {
                network_cid_short: existing.network_cid_short,
                network_name: existing.network_name,
                admin_email: existing.admin_email,
                status: existing.status,
                attempts: existing.attempts,
                last_attempt: existing.last_attempt,
                registered_at: existing.registered_at,
                rcc_url: RCC_URL.to_string(),
            });
        }
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let reg = store::RccRegistration {
        network_cid_short: network.cid_short().to_string(),
        network_cid_full: network.cid_full().to_string(),
        network_name: network.name().to_string(),
        admin_email: admin_email.clone(),
        status: "pending".to_string(),
        attempts: 0,
        last_attempt: None,
        registered_at: now,
    };
    store::save_rcc_registration(&conn, &reg).map_err(|e| e.to_string())?;

    // Lancer le retry en arrière-plan
    let data_dir_bg = data_dir(&app);
    let cid_short = network.cid_short().to_string();
    let app_bg = app.clone();
    tauri::async_runtime::spawn(async move {
        crate::rcc::register_with_retry(app_bg, data_dir_bg, cid_short).await;
    });

    Ok(RccStatusInfo {
        network_cid_short: reg.network_cid_short,
        network_name: reg.network_name,
        admin_email: reg.admin_email,
        status: reg.status,
        attempts: reg.attempts,
        last_attempt: reg.last_attempt,
        registered_at: reg.registered_at,
        rcc_url: RCC_URL.to_string(),
    })
}

/// Retourner le statut RCC d'un réseau (None si jamais initié).
#[tauri::command]
pub fn rcc_status(app: AppHandle, network_cid: String) -> Result<Option<RccStatusInfo>, String> {
    let conn = open(&app)?;
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    Ok(store::get_rcc_registration(&conn, network.cid_short()).map(|r| RccStatusInfo {
        network_cid_short: r.network_cid_short,
        network_name: r.network_name,
        admin_email: r.admin_email,
        status: r.status,
        attempts: r.attempts,
        last_attempt: r.last_attempt,
        registered_at: r.registered_at,
        rcc_url: RCC_URL.to_string(),
    }))
}

/// Retourner le statut RCC de tous les réseaux enregistrés.
#[tauri::command]
pub fn rcc_status_list(app: AppHandle) -> Result<Vec<RccStatusInfo>, String> {
    let conn = open(&app)?;
    Ok(store::list_rcc_registrations(&conn).into_iter().map(|r| RccStatusInfo {
        network_cid_short: r.network_cid_short,
        network_name: r.network_name,
        admin_email: r.admin_email,
        status: r.status,
        attempts: r.attempts,
        last_attempt: r.last_attempt,
        registered_at: r.registered_at,
        rcc_url: RCC_URL.to_string(),
    }).collect())
}

// ── Pairing ───────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct PairingInitInfo {
    pub link: String,
    pub expires_at: u64,
    pub device_id: String,
    pub device_label: String,
}

#[derive(Serialize)]
pub struct PairedDeviceInfo {
    pub id: String,
    pub label: String,
    pub paired_at: u64,
    pub revoked: bool,
    pub revoked_at: Option<u64>,
}

/// Generate a pairing link and register the secondary device.
#[tauri::command]
pub fn pair_init(app: AppHandle, label: String) -> Result<PairingInitInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let session = init_pairing(&keypair.secret_b58()).map_err(|e| e)?;
    let device = PairedDevice::new(label.clone());
    let device_id = device.id.clone();
    store::save_paired_device(&conn, &device).map_err(|e| e.to_string())?;
    Ok(PairingInitInfo {
        link: session.link,
        expires_at: session.expires_at,
        device_id,
        device_label: label,
    })
}

/// Complete pairing: restore identity from a civium://pair/... link.
#[tauri::command]
pub fn pair_complete(app: AppHandle, link: String, label: String) -> Result<PairedDeviceInfo, String> {
    let secret_b58 = complete_pairing(&link)?;
    let conn = open(&app)?;
    let keypair = CiviumKeypair::from_secret_b58(&secret_b58).map_err(|e| e.to_string())?;
    store::save_identity(&conn, &keypair).map_err(|e| e.to_string())?;
    let device = PairedDevice::new(label);
    let info = PairedDeviceInfo {
        id: device.id.clone(),
        label: device.label.clone(),
        paired_at: device.paired_at,
        revoked: false,
        revoked_at: None,
    };
    store::save_paired_device(&conn, &device).map_err(|e| e.to_string())?;
    Ok(info)
}

/// List all paired devices.
#[tauri::command]
pub fn pair_list(app: AppHandle) -> Result<Vec<PairedDeviceInfo>, String> {
    let conn = open(&app)?;
    let devices = store::list_paired_devices(&conn).map_err(|e| e.to_string())?;
    Ok(devices.into_iter().map(|d| PairedDeviceInfo {
        id: d.id,
        label: d.label,
        paired_at: d.paired_at,
        revoked: d.revoked,
        revoked_at: d.revoked_at,
    }).collect())
}

/// Revoke a paired device by ID.
#[tauri::command]
pub fn pair_revoke(app: AppHandle, device_id: String) -> Result<(), String> {
    let conn = open(&app)?;
    let devices = store::list_paired_devices(&conn).map_err(|e| e.to_string())?;
    let mut device = devices.into_iter()
        .find(|d| d.id == device_id)
        .ok_or_else(|| format!("appareil '{device_id}' introuvable"))?;
    if device.revoked {
        return Err(format!("l'appareil '{}' est déjà révoqué", device.label));
    }
    device.revoke();
    store::save_paired_device(&conn, &device).map_err(|e| e.to_string())
}
