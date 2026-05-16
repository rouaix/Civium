use crate::{Cid, CiviumError, CiviumKeypair};
use rand::RngCore;
use serde::{Deserialize, Serialize};

const PREFIX: &str = "civium-invite:";
const VERSION: u8 = 1;

/// Payload that is signed by the network's keypair.
#[derive(Debug, Serialize, Deserialize)]
pub struct InvitePayload {
    pub v: u8,
    pub network_cid_full: String,
    /// Base58 of the 32-byte Ed25519 public key — lets the recipient verify
    /// the signature without a running DHT node.
    pub network_pubkey_b58: String,
    pub network_name: String,
    pub inviter_cid_short: String,
    pub created_at: u64,
    /// Unix timestamp; 0 = no expiry.
    pub expires_at: u64,
    pub nonce_b58: String,
}

/// A signed invitation to join a Civium network.
///
/// Encoded as `civium-invite:<base58(json)>` for easy copy-paste and QR sharing.
#[derive(Debug, Serialize, Deserialize)]
pub struct Invitation {
    pub payload: InvitePayload,
    pub sig_b58: String,
}

impl Invitation {
    /// Create and sign a new invitation. The `expires_hours` parameter
    /// sets the TTL in hours; 0 means no expiry.
    pub fn create(
        network_keypair: &CiviumKeypair,
        network_name: &str,
        inviter_cid: &Cid,
        expires_hours: u64,
    ) -> Result<Self, CiviumError> {
        let now = unix_now();
        let expires_at = if expires_hours == 0 {
            0
        } else {
            now + expires_hours * 3600
        };

        let mut nonce = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut nonce);

        let payload = InvitePayload {
            v: VERSION,
            network_cid_full: network_keypair.cid().full().to_string(),
            network_pubkey_b58: bs58::encode(network_keypair.public_key_bytes()).into_string(),
            network_name: network_name.to_string(),
            inviter_cid_short: inviter_cid.short().to_string(),
            created_at: now,
            expires_at,
            nonce_b58: bs58::encode(nonce).into_string(),
        };

        let payload_bytes =
            serde_json::to_vec(&payload).map_err(|e| CiviumError::Invitation(e.to_string()))?;

        let sig = network_keypair
            .libp2p_keypair()
            .sign(&payload_bytes)
            .map_err(|e| CiviumError::Invitation(e.to_string()))?;

        Ok(Self {
            payload,
            sig_b58: bs58::encode(sig).into_string(),
        })
    }

    /// Encode to a `civium-invite:…` link.
    pub fn to_link(&self) -> Result<String, CiviumError> {
        let json =
            serde_json::to_string(self).map_err(|e| CiviumError::Invitation(e.to_string()))?;
        Ok(format!("{}{}", PREFIX, bs58::encode(json.as_bytes()).into_string()))
    }

    /// Decode from a `civium-invite:…` link.
    pub fn from_link(link: &str) -> Result<Self, CiviumError> {
        let encoded = link
            .strip_prefix(PREFIX)
            .ok_or_else(|| CiviumError::Invitation("invalid invite link".into()))?;
        let json_bytes = bs58::decode(encoded)
            .into_vec()
            .map_err(|e| CiviumError::Invitation(e.to_string()))?;
        serde_json::from_slice(&json_bytes)
            .map_err(|e| CiviumError::Invitation(e.to_string()))
    }

    /// Verify the signature and expiry. Returns `Ok(())` if valid.
    pub fn verify(&self) -> Result<(), CiviumError> {
        if self.payload.expires_at != 0 && unix_now() > self.payload.expires_at {
            return Err(CiviumError::Invitation("invitation has expired".into()));
        }

        let pubkey_bytes: [u8; 32] = bs58::decode(&self.payload.network_pubkey_b58)
            .into_vec()
            .map_err(|e| CiviumError::Invitation(e.to_string()))?
            .try_into()
            .map_err(|_| CiviumError::Invitation("invalid public key length in invite".into()))?;

        let ed_pub = libp2p::identity::ed25519::PublicKey::try_from_bytes(&pubkey_bytes)
            .map_err(|e| CiviumError::Invitation(e.to_string()))?;
        let lib_pub = libp2p::identity::PublicKey::from(ed_pub);

        let payload_bytes = serde_json::to_vec(&self.payload)
            .map_err(|e| CiviumError::Invitation(e.to_string()))?;

        let sig = bs58::decode(&self.sig_b58)
            .into_vec()
            .map_err(|e| CiviumError::Invitation(e.to_string()))?;

        if !lib_pub.verify(&payload_bytes, &sig) {
            return Err(CiviumError::Invitation("invalid signature".into()));
        }

        Ok(())
    }

    pub fn network_cid_full(&self) -> &str {
        &self.payload.network_cid_full
    }

    pub fn network_name(&self) -> &str {
        &self.payload.network_name
    }

    pub fn nonce_b58(&self) -> &str {
        &self.payload.nonce_b58
    }
}

fn unix_now() -> u64 { crate::time::unix_now() }
