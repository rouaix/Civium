use std::time::{SystemTime, UNIX_EPOCH};

use civium_core::{CiviumKeypair, RCC_URL};

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

/// Active la fédération ActivityPub pour un réseau via le RCC.
/// Retourne l'actor_url si succès.
pub async fn enable_ap(
    network_cid_full: &str,
    network_cid_short: &str,
    keypair: &CiviumKeypair,
) -> Result<String, String> {
    let timestamp = now();
    let canonical = format!("{}|ap_enable|{}", network_cid_full, timestamp);

    let sig_bytes = keypair
        .sign_bytes(canonical.as_bytes())
        .map_err(|e| e.to_string())?;
    let sig_b58 = bs58::encode(&sig_bytes).into_string();

    let body = serde_json::json!({
        "network_cid":       network_cid_full,
        "network_cid_short": network_cid_short,
        "timestamp":         timestamp,
        "signature":         sig_b58,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{}/api/ap/enable", RCC_URL);
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("réseau : {e}"))?;

    let status = resp.status();
    let text   = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("RCC {}: {}", status.as_u16(), text));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|_| format!("réponse RCC invalide : {text}"))?;

    json["actor_url"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("actor_url manquant dans la réponse RCC : {text}"))
}

/// Publie une note publique sur ActivityPub via le RCC.
/// Retourne (activity_id, delivered_count).
pub async fn post_note(
    network_cid_full: &str,
    note_id: &str,
    content: &str,
    keypair: &CiviumKeypair,
) -> Result<(String, u32), String> {
    let timestamp = now();
    let canonical = format!("{}|{}|{}|{}", network_cid_full, note_id, content, timestamp);

    let sig_bytes = keypair
        .sign_bytes(canonical.as_bytes())
        .map_err(|e| e.to_string())?;
    let sig_b58 = bs58::encode(&sig_bytes).into_string();

    let body = serde_json::json!({
        "network_cid": network_cid_full,
        "note_id":     note_id,
        "content":     content,
        "timestamp":   timestamp,
        "signature":   sig_b58,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{}/api/ap/post", RCC_URL);
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("réseau : {e}"))?;

    let status = resp.status();
    let text   = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("RCC {}: {}", status.as_u16(), text));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|_| format!("réponse RCC invalide : {text}"))?;

    let activity_id  = json["activity_id"].as_str().unwrap_or(note_id).to_string();
    let delivered_to = json["delivered_to"].as_u64().unwrap_or(0) as u32;

    Ok((activity_id, delivered_to))
}
