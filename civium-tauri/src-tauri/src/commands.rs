use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use civium_core::{
    network::{Invitation, Network},
    CiviumKeypair, GroupKey, MemberRole, Message, MessageKind, NodeCommand, TrustCircle,
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
    let mut network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    let record = network
        .admit(&member_cid, circle, MemberRole::Member)
        .map_err(|e| e.to_string())?;
    store::save_network(&conn, &network).map_err(|e| e.to_string())?;
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
