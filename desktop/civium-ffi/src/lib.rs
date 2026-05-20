//! civium-ffi — UniFFI bindings for mobile (Android/iOS).
//!
//! Exposes a synchronous, flat API over civium-core that can be called from
//! Kotlin (Android) and Swift (iOS) through generated UniFFI stubs, and
//! subsequently bridged to React Native via a thin native module.
//!
//! All functions take an explicit `data_dir` (String path) so the mobile app
//! can locate the SQLite database without a Tauri AppState.

mod store;

uniffi::setup_scaffolding!();

use civium_core::{CiviumKeypair, GroupKey, MessageKind, complete_pairing as core_complete_pairing};
use std::path::PathBuf;

// ── Data transfer types ───────────────────────────────────────────────────────

#[derive(uniffi::Record)]
pub struct IdentityInfo {
    pub cid_short:  String,
    pub cid_full:   String,
    pub secret_b58: String,
}

#[derive(uniffi::Record)]
pub struct NetworkInfo {
    pub cid_short:    String,
    pub name:         String,
    pub member_count: u32,
}

#[derive(uniffi::Record)]
pub struct MessageInfo {
    pub id:               String,
    pub author_cid_short: String,
    pub body:             String,
    pub sent_at:          u64,
    pub is_direct:        bool,
}

// ── Identity ──────────────────────────────────────────────────────────────────

/// Returns true if the local database at `data_dir` already contains an identity.
#[uniffi::export]
pub fn identity_exists(data_dir: String) -> bool {
    let conn = match store::open_db(&PathBuf::from(&data_dir)) {
        Ok(c) => c,
        Err(_) => return false,
    };
    store::identity_exists(&conn)
}

/// Generate a brand-new Ed25519 identity and persist it.
/// Fails if an identity already exists (call `identity_info` instead).
#[uniffi::export]
pub fn identity_init(data_dir: String) -> Result<IdentityInfo, CiviumFfiError> {
    let conn = store::open_db(&PathBuf::from(&data_dir))?;
    if store::identity_exists(&conn) {
        return Err(CiviumFfiError::IdentityAlreadyExists);
    }
    let kp = CiviumKeypair::generate()?;
    store::save_identity(&conn, &kp)?;
    let cid = kp.cid();
    Ok(IdentityInfo {
        cid_short:  cid.short().to_string(),
        cid_full:   cid.full().to_string(),
        secret_b58: kp.secret_b58().to_string(),
    })
}

/// Restore an identity from a `secret_b58` obtained via QR-code pairing.
/// Overwrites any existing identity.
#[uniffi::export]
pub fn identity_from_secret(data_dir: String, secret_b58: String) -> Result<IdentityInfo, CiviumFfiError> {
    let conn = store::open_db(&PathBuf::from(&data_dir))?;
    let kp = CiviumKeypair::from_secret_b58(&secret_b58)?;
    store::save_identity(&conn, &kp)?;
    let cid = kp.cid();
    Ok(IdentityInfo {
        cid_short:  cid.short().to_string(),
        cid_full:   cid.full().to_string(),
        secret_b58: kp.secret_b58().to_string(),
    })
}

/// Load the stored identity without changing it.
#[uniffi::export]
pub fn identity_info(data_dir: String) -> Result<IdentityInfo, CiviumFfiError> {
    let conn = store::open_db(&PathBuf::from(&data_dir))?;
    let (secret, short, full) = store::load_identity_row(&conn)?;
    Ok(IdentityInfo { cid_short: short, cid_full: full, secret_b58: secret })
}

// ── Pairing ───────────────────────────────────────────────────────────────────

/// Decode a `civium://pair/<b58payload>` QR-code link and return the `secret_b58`.
/// Call `identity_from_secret` afterwards to persist the recovered identity.
#[uniffi::export]
pub fn pairing_complete(link: String) -> Result<String, CiviumFfiError> {
    core_complete_pairing(&link).map_err(CiviumFfiError::Pairing)
}

// ── Networks ──────────────────────────────────────────────────────────────────

/// List all networks stored in the local database (offline read).
#[uniffi::export]
pub fn network_list(data_dir: String) -> Result<Vec<NetworkInfo>, CiviumFfiError> {
    let conn = store::open_db(&PathBuf::from(&data_dir))?;
    let networks = store::list_networks(&conn)?;
    Ok(networks
        .into_iter()
        .map(|n| NetworkInfo {
            cid_short:    n.cid_short().to_string(),
            name:         n.data.name.clone(),
            member_count: n.data.members.len() as u32,
        })
        .collect())
}

// ── Messages ──────────────────────────────────────────────────────────────────

/// List decrypted thread messages for a network, most recent last.
#[uniffi::export]
pub fn message_list(data_dir: String, network_cid: String) -> Result<Vec<MessageInfo>, CiviumFfiError> {
    let conn  = store::open_db(&PathBuf::from(&data_dir))?;
    let net   = store::list_networks(&conn)?
        .into_iter()
        .find(|n| n.cid_short() == network_cid)
        .ok_or_else(|| CiviumFfiError::NetworkNotFound(network_cid.clone()))?;
    let gk    = GroupKey::from_b58(&net.data.group_key_b58)?;
    let msgs  = store::load_messages(&conn, &network_cid)?;
    let mut result = Vec::with_capacity(msgs.len());
    for msg in msgs {
        let body = gk.decrypt(&msg.nonce_b58, &msg.ciphertext_b58)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_else(|_| "[chiffré]".into());
        let is_direct = matches!(msg.kind, MessageKind::Direct { .. } | MessageKind::E2E { .. });
        result.push(MessageInfo {
            id:               msg.id,
            author_cid_short: msg.author_cid_short,
            body,
            sent_at:          msg.sent_at,
            is_direct,
        });
    }
    Ok(result)
}

/// Encrypt and store a thread message locally (synced to P2P when connected).
#[uniffi::export]
pub fn message_send(
    data_dir:    String,
    network_cid: String,
    body:        String,
) -> Result<MessageInfo, CiviumFfiError> {
    let conn   = store::open_db(&PathBuf::from(&data_dir))?;
    let net    = store::list_networks(&conn)?
        .into_iter()
        .find(|n| n.cid_short() == network_cid)
        .ok_or_else(|| CiviumFfiError::NetworkNotFound(network_cid.clone()))?;
    let kp     = store::load_identity(&conn)?;
    let gk     = GroupKey::from_b58(&net.data.group_key_b58)?;
    let (nonce_b58, ciphertext_b58) = gk.encrypt(body.as_bytes())?;
    let sent_at = unix_now();
    let msg = civium_core::Message {
        id:               nonce_b58.clone(),
        author_cid_short: kp.cid().short().to_string(),
        kind:             MessageKind::Thread,
        nonce_b58,
        ciphertext_b58,
        sent_at,
        reply_to_id:      None,
    };
    store::save_message(&conn, &network_cid, &msg)?;
    Ok(MessageInfo {
        id:               msg.id,
        author_cid_short: msg.author_cid_short,
        body,
        sent_at,
        is_direct: false,
    })
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum CiviumFfiError {
    #[error("identité déjà présente")]
    IdentityAlreadyExists,
    #[error("réseau introuvable : {0}")]
    NetworkNotFound(String),
    #[error("jumelage : {0}")]
    Pairing(String),
    #[error("{0}")]
    Core(String),
    #[error("base de données : {0}")]
    Db(String),
}

impl From<civium_core::CiviumError> for CiviumFfiError {
    fn from(e: civium_core::CiviumError) -> Self { Self::Core(e.to_string()) }
}

impl From<anyhow::Error> for CiviumFfiError {
    fn from(e: anyhow::Error) -> Self { Self::Db(e.to_string()) }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
