use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A shared document belonging to a Civium network.
/// The body is stored encrypted with the network group key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub network_cid_short: String,
    pub title: String,
    /// ChaCha20-Poly1305 nonce (base58) for the encrypted body.
    pub nonce_b58: String,
    /// Encrypted body (base58).
    pub body_ciphertext: String,
    pub version: u32,
    pub created_by: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Document {
    pub fn new(
        network_cid_short: String,
        title: String,
        nonce_b58: String,
        body_ciphertext: String,
        created_by: String,
    ) -> Self {
        let now = unix_now();
        Self {
            id: uuid(),
            network_cid_short,
            title,
            nonce_b58,
            body_ciphertext,
            version: 1,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }
}

fn unix_now() -> u64 { crate::time::unix_now() }
fn uuid() -> String { Uuid::new_v4().to_string() }
