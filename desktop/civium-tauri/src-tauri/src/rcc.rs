use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter};

use civium_core::{RccPayload, RCC_URL};

use crate::store::{self, RccRegistration};

/// Délais de retry en secondes : 5s → 30s → 5min → 30min → 1h → toutes les heures.
const RETRY_DELAYS: [u64; 6] = [5, 30, 300, 1800, 3600, 3600];

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn retry_delay(attempts: u32) -> u64 {
    let idx = (attempts as usize).min(RETRY_DELAYS.len() - 1);
    RETRY_DELAYS[idx]
}

/// Tente d'envoyer la requête d'enregistrement au RCC.
/// Retourne Ok(()) si le serveur répond 200/201, Err(reason) sinon.
async fn attempt_http(reg: &RccRegistration, keypair: &civium_core::CiviumKeypair) -> Result<(), String> {
    let payload = RccPayload::new(
        reg.network_cid_full.clone(),
        reg.network_name.clone(),
        keypair.cid().full().to_string(),
        keypair.pub_key_b58(),
        reg.admin_email.clone(),
        reg.registered_at,
    );

    let sig_bytes = keypair
        .sign_bytes(&payload.canonical_bytes())
        .map_err(|e| e.to_string())?;
    let sig_b58 = bs58::encode(&sig_bytes).into_string();

    let body = serde_json::json!({
        "network_cid":   payload.network_cid,
        "network_name":  payload.network_name,
        "admin_cid":     payload.admin_cid,
        "admin_pubkey":  payload.admin_pubkey,
        "admin_email":   payload.admin_email,
        "registered_at": payload.registered_at,
        "signature":     sig_b58,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{}/api/register", RCC_URL);
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("réseau : {e}"))?;

    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else {
        let text = resp.text().await.unwrap_or_default();
        Err(format!("RCC réponse {}: {}", status.as_u16(), text))
    }
}

/// Enregistre un réseau au RCC avec retry exponentiel en arrière-plan.
/// À appeler après avoir inséré une entrée `pending` dans la DB.
pub async fn register_with_retry(app: AppHandle, data_dir: PathBuf, network_cid_short: String) {
    loop {
        let conn = match store::open_db(&data_dir) {
            Ok(c) => c,
            Err(_) => return,
        };

        let reg = match store::get_rcc_registration(&conn, &network_cid_short) {
            Some(r) => r,
            None => return,
        };

        if reg.status == "registered" {
            return;
        }

        let keypair = match store::load_identity(&conn) {
            Ok(k) => k,
            Err(_) => return,
        };
        drop(conn);

        let delay = if reg.attempts == 0 { 0 } else { retry_delay(reg.attempts - 1) };
        if delay > 0 {
            tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
        }

        let ts = now();
        match attempt_http(&reg, &keypair).await {
            Ok(()) => {
                let conn = store::open_db(&data_dir).ok();
                if let Some(conn) = conn {
                    let _ = store::update_rcc_status(&conn, &network_cid_short, "registered", reg.attempts + 1, ts);
                }
                let _ = app.emit("civium://rcc-status-changed", serde_json::json!({
                    "network_cid_short": network_cid_short,
                    "status": "registered",
                }));
                return;
            }
            Err(reason) => {
                let new_attempts = reg.attempts + 1;
                let conn = store::open_db(&data_dir).ok();
                if let Some(conn) = conn {
                    let _ = store::update_rcc_status(&conn, &network_cid_short, "pending", new_attempts, ts);
                }
                // Après 10 tentatives infructueuses, marquer failed et arrêter
                if new_attempts >= 10 {
                    let conn = store::open_db(&data_dir).ok();
                    if let Some(conn) = conn {
                        let _ = store::update_rcc_status(&conn, &network_cid_short, "failed", new_attempts, ts);
                    }
                    let _ = app.emit("civium://rcc-status-changed", serde_json::json!({
                        "network_cid_short": network_cid_short,
                        "status": "failed",
                        "reason": reason,
                    }));
                    return;
                }
                // Continue la boucle — le prochain tour attendra `retry_delay(new_attempts)`
                let conn_upd = store::open_db(&data_dir).ok();
                if let Some(conn) = conn_upd {
                    // Recharger pour avoir le nouveau attempts
                    let _ = store::update_rcc_status(&conn, &network_cid_short, "pending", new_attempts, ts);
                }
            }
        }
    }
}
