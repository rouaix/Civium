use libp2p::identity::{self, Keypair as Libp2pKeypair};
use zeroize::ZeroizeOnDrop;
use crate::error::CiviumError;
use super::Cid;

/// Member identity — an Ed25519 keypair from which the CID is derived.
///
/// The same keypair drives both the Civium CID and the libp2p PeerId,
/// ensuring a single cryptographic root per member (Phase 0 — single device).
/// Multi-device (sub-key derivation) comes in weeks 11-12.
#[derive(ZeroizeOnDrop)]
pub struct CiviumKeypair {
    #[zeroize(skip)]
    libp2p: Libp2pKeypair,
    secret: [u8; 32],
    pub_key: [u8; 32],
}

impl CiviumKeypair {
    /// Generate a new random Ed25519 keypair.
    pub fn generate() -> Result<Self, CiviumError> {
        let ed = identity::ed25519::Keypair::generate();
        Self::from_ed25519(ed)
    }

    /// Restore a keypair from its 32-byte secret key.
    pub fn from_secret_bytes(bytes: [u8; 32]) -> Result<Self, CiviumError> {
        let secret = identity::ed25519::SecretKey::try_from_bytes(bytes)
            .map_err(|e| CiviumError::Identity(e.to_string()))?;
        let ed = identity::ed25519::Keypair::from(secret);
        Self::from_ed25519(ed)
    }

    /// Restore a keypair from a base58-encoded secret key (as exported by `secret_b58`).
    pub fn from_secret_b58(s: &str) -> Result<Self, CiviumError> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| CiviumError::Identity(e.to_string()))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| CiviumError::Identity("secret key must be 32 bytes".into()))?;
        Self::from_secret_bytes(arr)
    }

    fn from_ed25519(ed: identity::ed25519::Keypair) -> Result<Self, CiviumError> {
        let pub_key: [u8; 32] = ed.public().to_bytes();
        let secret: [u8; 32] = ed
            .secret()
            .as_ref()
            .try_into()
            .map_err(|_| CiviumError::Identity("unexpected secret key length".into()))?;
        Ok(Self {
            libp2p: Libp2pKeypair::from(ed),
            secret,
            pub_key,
        })
    }

    pub fn cid(&self) -> Cid {
        Cid::from_public_key_bytes(&self.pub_key)
    }

    pub fn libp2p_keypair(&self) -> &Libp2pKeypair {
        &self.libp2p
    }

    pub fn public_key_bytes(&self) -> &[u8; 32] {
        &self.pub_key
    }

    /// Base58-encoded secret key — store securely, treat as a password.
    pub fn secret_b58(&self) -> String {
        bs58::encode(self.secret).into_string()
    }
}
