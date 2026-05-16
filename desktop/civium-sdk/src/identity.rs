//! Identity management — create, load, and inspect Civium cryptographic identities.

use civium_core::CiviumKeypair;

/// Summary of a Civium identity.
#[derive(Debug, Clone)]
pub struct IdentityInfo {
    /// Short CID (first 8 chars of the full CID) — safe to display in UIs.
    pub cid_short: String,
    /// Full CID — unique global identifier derived from the Ed25519 public key.
    pub cid_full: String,
    /// Base58-encoded Ed25519 public key — safe to share.
    pub pub_key_b58: String,
}

impl From<&CiviumKeypair> for IdentityInfo {
    fn from(kp: &CiviumKeypair) -> Self {
        let cid = kp.cid();
        Self {
            cid_short:  cid.short().to_string(),
            cid_full:   cid.to_string(),
            pub_key_b58: kp.pub_key_b58(),
        }
    }
}
