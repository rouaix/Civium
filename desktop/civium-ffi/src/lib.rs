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

use civium_core::{
    network::{MemberRecord, MemberRole, TrustCircle},
    CiviumKeypair, Document, GroupKey, MessageKind, Proposal, Vote,
    complete_pairing as core_complete_pairing,
};
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

#[derive(uniffi::Record)]
pub struct DocumentInfo {
    pub id:            String,
    pub title:         String,
    pub created_by:    String,
    pub created_at:    u64,
    pub updated_at:    u64,
}

#[derive(uniffi::Record)]
pub struct AgendaEventInfo {
    pub id:          String,
    pub title:       String,
    pub description: String,
    pub start_at:    u64,
    pub created_by:  String,
}

#[derive(uniffi::Record)]
pub struct ProposalInfo {
    pub id:          String,
    pub title:       String,
    pub description: String,
    pub options:     Vec<String>,
    pub created_by:  String,
    pub created_at:  u64,
    pub closes_at:   u64,
    pub status:      String,
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

// ── Networks (write) ─────────────────────────────────────────────────────────

/// Create a new network with the local identity as admin (offline, no RCC).
#[uniffi::export]
pub fn network_create(
    data_dir:     String,
    name:         String,
    description:  String,
    is_public:    bool,
) -> Result<NetworkInfo, CiviumFfiError> {
    let conn = store::open_db(&PathBuf::from(&data_dir))?;
    let kp   = store::load_identity(&conn)?;
    let cid  = kp.cid();
    let net  = civium_core::network::Network::create(
        name,
        &cid,
        cid.short().to_string(),
        Some(bs58::encode(kp.public_key_bytes()).into_string()),
        is_public,
        None,
    )?;
    // Store description in the network name for now (no dedicated field yet).
    let _ = description;
    let member_count = net.data.members.len() as u32;
    let cid_short = net.cid_short().to_string();
    let net_name = net.data.name.clone();
    store::save_network(&conn, &net)?;
    Ok(NetworkInfo { cid_short, name: net_name, member_count })
}

/// Admit a member to a network (admin-only in production; not enforced in FFI).
#[uniffi::export]
pub fn member_admit(
    data_dir:         String,
    network_cid:      String,
    member_cid_short: String,
    member_cid_full:  String,
    display_name:     String,
) -> Result<(), CiviumFfiError> {
    let conn = store::open_db(&PathBuf::from(&data_dir))?;
    let mut nets = store::list_networks(&conn)?;
    let net = nets.iter_mut()
        .find(|n| n.cid_short() == network_cid)
        .ok_or_else(|| CiviumFfiError::NetworkNotFound(network_cid.clone()))?;
    let member = MemberRecord {
        cid_short:    member_cid_short,
        cid_full:     member_cid_full,
        display_name,
        circle:       TrustCircle::Connaissance,
        role:         MemberRole::Member,
        joined_at:    unix_now(),
        is_minor:     false,
        pub_key_b58:  None,
    };
    net.data.members.push(member);
    store::save_network(&conn, net)?;
    Ok(())
}

// ── Messages — direct ────────────────────────────────────────────────────────

/// Encrypt and store a direct message locally.
#[uniffi::export]
pub fn message_send_direct(
    data_dir:    String,
    network_cid: String,
    to_cid:      String,
    body:        String,
) -> Result<MessageInfo, CiviumFfiError> {
    let conn    = store::open_db(&PathBuf::from(&data_dir))?;
    let net     = store::list_networks(&conn)?
        .into_iter()
        .find(|n| n.cid_short() == network_cid)
        .ok_or_else(|| CiviumFfiError::NetworkNotFound(network_cid.clone()))?;
    let kp      = store::load_identity(&conn)?;
    let gk      = GroupKey::from_b58(&net.data.group_key_b58)?;
    let (nonce_b58, ciphertext_b58) = gk.encrypt(body.as_bytes())?;
    let sent_at = unix_now();
    let msg = civium_core::Message {
        id:               nonce_b58.clone(),
        author_cid_short: kp.cid().short().to_string(),
        kind:             MessageKind::Direct { to_cid_short: to_cid },
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
        is_direct:        true,
    })
}

// ── Documents ─────────────────────────────────────────────────────────────────

/// List documents for a network.
#[uniffi::export]
pub fn document_list(data_dir: String, network_cid: String) -> Result<Vec<DocumentInfo>, CiviumFfiError> {
    let conn = store::open_db(&PathBuf::from(&data_dir))?;
    let docs = store::load_documents(&conn, &network_cid)?;
    Ok(docs.into_iter().map(|d| DocumentInfo {
        id:         d.id,
        title:      d.title,
        created_by: d.created_by,
        created_at: d.created_at,
        updated_at: d.updated_at,
    }).collect())
}

/// Create and store a new document with encrypted body.
#[uniffi::export]
pub fn document_create(
    data_dir:    String,
    network_cid: String,
    title:       String,
    body:        String,
) -> Result<DocumentInfo, CiviumFfiError> {
    let conn    = store::open_db(&PathBuf::from(&data_dir))?;
    let net     = store::list_networks(&conn)?
        .into_iter()
        .find(|n| n.cid_short() == network_cid)
        .ok_or_else(|| CiviumFfiError::NetworkNotFound(network_cid.clone()))?;
    let kp      = store::load_identity(&conn)?;
    let gk      = GroupKey::from_b58(&net.data.group_key_b58)?;
    let (nonce_b58, body_ciphertext) = gk.encrypt(body.as_bytes())?;
    let doc = Document::new(
        network_cid.clone(),
        title,
        nonce_b58,
        body_ciphertext,
        kp.cid().short().to_string(),
    );
    let info = DocumentInfo {
        id:         doc.id.clone(),
        title:      doc.title.clone(),
        created_by: doc.created_by.clone(),
        created_at: doc.created_at,
        updated_at: doc.updated_at,
    };
    store::save_document(&conn, &network_cid, &doc)?;
    Ok(info)
}

// ── Agenda ────────────────────────────────────────────────────────────────────

/// List agenda events for a network.
#[uniffi::export]
pub fn agenda_list(data_dir: String, network_cid: String) -> Result<Vec<AgendaEventInfo>, CiviumFfiError> {
    let conn = store::open_db(&PathBuf::from(&data_dir))?;
    let events = store::load_agenda_events(&conn, &network_cid)?;
    Ok(events.into_iter().map(|e| AgendaEventInfo {
        id:          e.id,
        title:       e.title,
        description: e.description,
        start_at:    e.start_at,
        created_by:  e.created_by,
    }).collect())
}

// ── Proposals ────────────────────────────────────────────────────────────────

/// List proposals for a network.
#[uniffi::export]
pub fn proposal_list(data_dir: String, network_cid: String) -> Result<Vec<ProposalInfo>, CiviumFfiError> {
    let conn = store::open_db(&PathBuf::from(&data_dir))?;
    let proposals = store::load_proposals(&conn, &network_cid)?;
    Ok(proposals.into_iter().map(proposal_to_info).collect())
}

/// Create a new proposal.
#[uniffi::export]
pub fn proposal_create(
    data_dir:    String,
    network_cid: String,
    title:       String,
    description: String,
    options:     Vec<String>,
    hours:       u64,
) -> Result<ProposalInfo, CiviumFfiError> {
    let conn    = store::open_db(&PathBuf::from(&data_dir))?;
    let kp      = store::load_identity(&conn)?;
    let now     = unix_now();
    let closes_at = if hours > 0 { now + hours * 3600 } else { 0 };
    let proposal = Proposal::new(
        network_cid.clone(),
        title,
        description,
        options,
        kp.cid().short().to_string(),
        now,
        closes_at,
        0,
    );
    let info = proposal_to_info(proposal.clone());
    store::save_proposal(&conn, &network_cid, &proposal)?;
    Ok(info)
}

/// Cast a vote on a proposal.
#[uniffi::export]
pub fn vote_cast(
    data_dir:    String,
    proposal_id: String,
    choice:      u32,
) -> Result<(), CiviumFfiError> {
    let conn = store::open_db(&PathBuf::from(&data_dir))?;
    let kp   = store::load_identity(&conn)?;
    let vote = Vote {
        proposal_id,
        voter_cid_short: kp.cid().short().to_string(),
        choice_index:    choice as usize,
        cast_at:         unix_now(),
    };
    store::save_vote(&conn, &vote)?;
    Ok(())
}

fn proposal_to_info(p: Proposal) -> ProposalInfo {
    ProposalInfo {
        id:          p.id,
        title:       p.title,
        description: p.description,
        options:     p.options,
        created_by:  p.created_by,
        created_at:  p.created_at,
        closes_at:   p.closes_at,
        status:      p.status.to_string(),
    }
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
