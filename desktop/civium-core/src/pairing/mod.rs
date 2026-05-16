use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Key, Nonce};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

const PAIRING_CONTEXT: &str = "civium pairing session v0";
/// Pairing link validity window (10 minutes).
const PAIRING_EXPIRY_SECS: u64 = 600;

/// Record of a paired (secondary) device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedDevice {
    pub id: String,
    pub label: String,
    pub paired_at: u64,
    pub revoked: bool,
    #[serde(default)]
    pub revoked_at: Option<u64>,
}

/// Result of `init_pairing`: the deep-link to share with the secondary device.
#[derive(Debug, Clone)]
pub struct PairingInit {
    /// civium://pair/<base58_payload> — contains the encrypted secret.
    /// Share via QR code or secure channel. Valid for 10 minutes.
    pub link: String,
    pub expires_at: u64,
}

/// Generate a pairing link that encodes the encrypted identity secret.
///
/// Payload layout: `pairing_key (16 B) || nonce (12 B) || ciphertext (secret_b58.len() + 16 B)`
/// The receiving device calls `complete_pairing(link)` to recover `secret_b58`.
pub fn init_pairing(secret_b58: &str) -> Result<PairingInit, String> {
    // One-time 16-byte random key
    let mut pairing_key = [0u8; 16];
    OsRng.fill_bytes(&mut pairing_key);

    // Derive a 32-byte encryption key via blake3 KDF
    let enc_key_bytes = blake3::derive_key(PAIRING_CONTEXT, &pairing_key);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&enc_key_bytes));

    // Encrypt the secret
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, secret_b58.as_bytes())
        .map_err(|e| format!("chiffrement du jumelage échoué: {e}"))?;

    // Build payload: pairing_key || nonce || ciphertext
    let mut payload = Vec::with_capacity(16 + 12 + ciphertext.len());
    payload.extend_from_slice(&pairing_key);
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&ciphertext);

    let link = format!("civium://pair/{}", bs58::encode(&payload).into_string());
    let now = unix_now();
    Ok(PairingInit { link, expires_at: now + PAIRING_EXPIRY_SECS })
}

/// Recover the `secret_b58` from a `civium://pair/<payload>` link.
pub fn complete_pairing(link: &str) -> Result<String, String> {
    let encoded = link
        .strip_prefix("civium://pair/")
        .ok_or_else(|| "lien invalide — doit commencer par civium://pair/".to_string())?;

    let payload = bs58::decode(encoded)
        .into_vec()
        .map_err(|e| format!("décodage base58 échoué: {e}"))?;

    // Minimum: 16 (key) + 12 (nonce) + 1 (at least one byte of plaintext) + 16 (AEAD tag)
    if payload.len() < 45 {
        return Err("payload de jumelage trop court".to_string());
    }

    let pairing_key = &payload[..16];
    let nonce = Nonce::from_slice(&payload[16..28]);
    let ciphertext = &payload[28..];

    let enc_key_bytes = blake3::derive_key(PAIRING_CONTEXT, pairing_key);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&enc_key_bytes));

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "déchiffrement échoué — lien invalide ou expiré".to_string())?;

    String::from_utf8(plaintext)
        .map_err(|_| "secret déchiffré invalide".to_string())
}

fn unix_now() -> u64 { crate::time::unix_now() }
fn uuid() -> String { uuid::Uuid::new_v4().to_string() }

impl PairedDevice {
    pub fn new(label: String) -> Self {
        Self {
            id: uuid(),
            label,
            paired_at: unix_now(),
            revoked: false,
            revoked_at: None,
        }
    }

    pub fn revoke(&mut self) {
        self.revoked = true;
        self.revoked_at = Some(unix_now());
    }
}
