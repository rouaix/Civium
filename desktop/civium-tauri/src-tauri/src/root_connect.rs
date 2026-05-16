//! Auto-connexion au réseau racine "civium".
//!
//! Chaque réseau créé se connecte automatiquement au réseau racine (permanent,
//! non révocable). La connexion est établie via P2P en envoyant un
//! `CiviumRequest::ConnectRequest` signé. Le serveur racine l'accepte
//! automatiquement (`auto_accept_connections = true`).
//!
//! Le schéma de retry est identique à l'enregistrement RCC :
//! 5 s → 30 s → 5 min → 30 min → 1 h → toutes les heures.

use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::{AppHandle, Emitter, Manager};
use tracing::warn;

use civium_core::{
    ConnectionRecord, ConnectionState, NodeConfig, CiviumNode,
    ShareAgreement, ShareTerms,
    CiviumRequest, CiviumResponse, NodeCommand, NodeEvent,
    peer_id_from_multiaddr,
    CIVIUM_ROOT_NETWORK_CID_FULL, CIVIUM_ROOT_NETWORK_CID_SHORT,
    CIVIUM_ROOT_NETWORK_NAME, CIVIUM_ROOT_NODE_ADDR,
    root_configured,
};

use crate::store;

const RETRY_DELAYS: [u64; 6] = [5, 30, 300, 1800, 3600, 3600];

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn data_dir(app: &AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("./civium-data"))
}

fn retry_delay(attempts: u32) -> u64 {
    RETRY_DELAYS[(attempts as usize).min(RETRY_DELAYS.len() - 1)]
}

/// Lance la connexion au réseau racine en arrière-plan avec retry.
///
/// Appelé juste après `network_create`. Si la connexion existe déjà (état Active),
/// ne fait rien.
pub async fn connect_to_root_with_retry(
    app: AppHandle,
    network_cid_full: String,
    network_cid_short: String,
    network_name: String,
    privacy: bool,
) {
    if !root_configured() {
        return; // constantes de production non encore configurées
    }

    // Vérifier si déjà connecté
    {
        let dir = data_dir(&app);
        if let Ok(conn) = store::open_db(&dir) {
            if let Ok(Some(rec)) = store::load_connection(&conn, &network_cid_full, CIVIUM_ROOT_NETWORK_CID_FULL) {
                if rec.state == ConnectionState::Active {
                    return;
                }
            }
        }
    }

    let mut attempts = 0u32;
    loop {
        match attempt_connect(&app, &network_cid_full, &network_name, privacy).await {
            Ok(()) => {
                let _ = app.emit("civium://root-connected", &network_cid_short);
                return;
            }
            Err(e) => {
                warn!("root connect attempt {attempts} failed: {e}");
                attempts += 1;
                tokio::time::sleep(Duration::from_secs(retry_delay(attempts))).await;
            }
        }
    }
}

/// Tente une connexion P2P unique au réseau racine.
async fn attempt_connect(
    app: &AppHandle,
    network_cid_full: &str,
    network_name: &str,
    privacy: bool,
) -> Result<(), String> {
    let dir = data_dir(app);
    let conn = store::open_db(&dir).map_err(|e| e.to_string())?;
    let keypair = store::load_identity(&conn).map_err(|_| "no identity".to_string())?;

    // Construire la SignedRequest
    let our_terms = ShareTerms { expose_member_directory: true, privacy };
    let signed_req = ShareAgreement::build_request(
        &keypair,
        network_name,
        our_terms.clone(),
        CIVIUM_ROOT_NETWORK_CID_FULL,
    ).map_err(|e| e.to_string())?;

    let signed_req_json = serde_json::to_string(&signed_req)
        .map_err(|e| e.to_string())?;

    // Démarrer un nœud P2P éphémère
    let config = NodeConfig {
        bootstrap_peers: vec![CIVIUM_ROOT_NODE_ADDR.to_string()],
        ..NodeConfig::default()
    };
    let (node, mut handle) = CiviumNode::new(keypair, config).await
        .map_err(|e| e.to_string())?;

    tokio::spawn(node.run());

    // Parser le PeerId depuis l'adresse du nœud racine
    let root_addr: civium_core::Multiaddr = CIVIUM_ROOT_NODE_ADDR.parse()
        .map_err(|_| "invalid CIVIUM_ROOT_NODE_ADDR multiaddr".to_string())?;
    let root_peer_id = peer_id_from_multiaddr(&root_addr)
        .ok_or_else(|| "CIVIUM_ROOT_NODE_ADDR must include /p2p/<PeerId>".to_string())?;

    // Attendre la connexion puis envoyer la requête
    let result = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            match handle.events.recv().await {
                Some(NodeEvent::PeerConnected { peer_id }) if peer_id == root_peer_id => {
                    let _ = handle.commands.send(NodeCommand::SendRequest {
                        peer: root_peer_id,
                        request: CiviumRequest::ConnectRequest {
                            signed_request_json: signed_req_json.clone(),
                        },
                    }).await;
                }
                Some(NodeEvent::OutboundResponse {
                    response: CiviumResponse::ConnectAccepted { apc_json }, ..
                }) => {
                    return Ok(apc_json);
                }
                Some(NodeEvent::OutboundResponse {
                    response: CiviumResponse::ConnectRejected { reason }, ..
                }) => {
                    return Err(format!("root rejected: {reason}"));
                }
                None => return Err("node stopped".to_string()),
                _ => {}
            }
        }
    }).await;

    let apc_json = match result {
        Ok(Ok(j))  => j,
        Ok(Err(e)) => return Err(e),
        Err(_)     => return Err("timeout waiting for root node".to_string()),
    };

    // Stocker la ConnectionRecord Active
    let apc: ShareAgreement = serde_json::from_str(&apc_json)
        .map_err(|e| format!("invalid APC json: {e}"))?;

    let ts = now();
    let record = ConnectionRecord {
        peer_cid_full:    CIVIUM_ROOT_NETWORK_CID_FULL.to_string(),
        peer_cid_short:   CIVIUM_ROOT_NETWORK_CID_SHORT.to_string(),
        peer_name:        CIVIUM_ROOT_NETWORK_NAME.to_string(),
        peer_pubkey_b58:  apc.acceptance.from_pubkey_b58.clone(),
        state:            ConnectionState::Active,
        initiated_at:     ts,
        updated_at:       ts,
        our_terms,
        their_terms:      Some(apc.acceptance.from_terms.clone()),
        incoming_request: None,
        apc:              Some(apc),
    };

    let conn2 = store::open_db(&dir).map_err(|e| e.to_string())?;
    store::save_connection(&conn2, network_cid_full, &record)
        .map_err(|e| e.to_string())?;

    Ok(())
}
