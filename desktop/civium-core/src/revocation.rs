use serde::{Deserialize, Serialize};

/// A signed revocation notice for a compromised or abandoned CID.
///
/// Validity rules:
/// - `pub_key_b58` must be the Ed25519 public key whose BLAKE3 hash equals `cid_full`
///   (CID = "civ1" + bs58(blake3(pub_key_bytes))).
/// - `signature_b58` is an Ed25519 signature over `signing_payload(cid_full, revoked_at)`,
///   verifiable with `pub_key_b58`.
/// - Once a revocation is stored, messages authored by `cid_full` should be
///   treated as suspect (filtered from display, rejected on receipt).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationRecord {
    /// Full CID being revoked (e.g. "civ1abc…").
    pub cid_full: String,
    /// Base58-encoded Ed25519 public key — must hash to `cid_full` via BLAKE3.
    pub pub_key_b58: String,
    /// Human-readable reason (optional, max 256 chars).
    pub reason: String,
    /// Unix timestamp of the revocation claim.
    pub revoked_at: u64,
    /// Ed25519 signature (base58) over `signing_payload(cid_full, revoked_at)`.
    pub signature_b58: String,
}

impl RevocationRecord {
    /// Return the canonical payload that must be signed.
    pub fn signing_payload(cid_full: &str, revoked_at: u64) -> Vec<u8> {
        format!("civium-revoke:{cid_full}:{revoked_at}").into_bytes()
    }

    /// Verify:
    /// 1. `pub_key_b58` hashes (BLAKE3) to the raw bytes inside `cid_full`.
    /// 2. `signature_b58` is a valid Ed25519 signature over the canonical payload.
    pub fn verify(&self) -> bool {
        // 1. Reconstruct and compare CID
        let pk_bytes = match bs58::decode(&self.pub_key_b58).into_vec() {
            Ok(b) if b.len() == 32 => {
                let arr: [u8; 32] = b.try_into().unwrap();
                arr
            }
            _ => return false,
        };
        let expected_cid = crate::identity::Cid::from_public_key_bytes(&pk_bytes);
        if expected_cid.full() != self.cid_full {
            return false;
        }

        // 2. Verify Ed25519 signature
        let payload = Self::signing_payload(&self.cid_full, self.revoked_at);
        let sig_bytes = match bs58::decode(&self.signature_b58).into_vec() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let vk = match libp2p::identity::ed25519::PublicKey::try_from_bytes(&pk_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };
        vk.verify(&payload, &sig_bytes)
    }
}
