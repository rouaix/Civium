use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::{CiviumError, CiviumKeypair};

use super::record::{AcceptPayload, RequestPayload, ShareTerms, SignedRequest};

/// Accord de Partage Civium (APC) — the cryptographic contract binding two networks.
///
/// Contains two independently-signed documents:
///   1. `request`    — signed by the requesting network (A)
///   2. `acceptance` — signed by the accepting network (B), referencing the request nonce
///
/// Either side can verify the agreement's integrity without a trusted third party.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareAgreement {
    pub request: RequestPayload,
    pub sig_request_b58: String,
    pub acceptance: AcceptPayload,
    pub sig_acceptance_b58: String,
}

impl ShareAgreement {
    /// Build and sign a connection request (called by the initiating network A).
    pub fn build_request(
        from_keypair: &CiviumKeypair,
        from_name: &str,
        from_terms: ShareTerms,
        to_cid_full: &str,
    ) -> Result<SignedRequest, CiviumError> {
        let mut nonce = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut nonce);

        let payload = RequestPayload {
            v: 1,
            nonce_b58: bs58::encode(nonce).into_string(),
            from_cid_full: from_keypair.cid().full().to_string(),
            from_pubkey_b58: bs58::encode(from_keypair.public_key_bytes()).into_string(),
            from_name: from_name.to_string(),
            from_terms,
            to_cid_full: to_cid_full.to_string(),
            created_at: unix_now(),
        };

        let bytes = serde_json::to_vec(&payload)
            .map_err(|e| CiviumError::Network(e.to_string()))?;
        let sig = from_keypair
            .libp2p_keypair()
            .sign(&bytes)
            .map_err(|e| CiviumError::Network(e.to_string()))?;

        Ok(SignedRequest {
            payload,
            sig_b58: bs58::encode(sig).into_string(),
        })
    }

    /// Build, sign, and finalize the APC (called by the accepting network B).
    ///
    /// Verifies A's request signature, then B signs the acceptance.
    pub fn build_from_acceptance(
        signed_request: &SignedRequest,
        acceptor_keypair: &CiviumKeypair,
        acceptor_name: &str,
        acceptor_terms: ShareTerms,
    ) -> Result<Self, CiviumError> {
        // Verify A's signature before countersigning
        let request_bytes = serde_json::to_vec(&signed_request.payload)
            .map_err(|e| CiviumError::Network(e.to_string()))?;
        verify_ed25519_sig(
            &signed_request.payload.from_pubkey_b58,
            &request_bytes,
            &signed_request.sig_b58,
        )?;

        let acceptance = AcceptPayload {
            v: 1,
            request_nonce_b58: signed_request.payload.nonce_b58.clone(),
            from_cid_full: acceptor_keypair.cid().full().to_string(),
            from_pubkey_b58: bs58::encode(acceptor_keypair.public_key_bytes()).into_string(),
            from_name: acceptor_name.to_string(),
            from_terms: acceptor_terms,
            accepted_at: unix_now(),
        };

        let accept_bytes = serde_json::to_vec(&acceptance)
            .map_err(|e| CiviumError::Network(e.to_string()))?;
        let sig_b = acceptor_keypair
            .libp2p_keypair()
            .sign(&accept_bytes)
            .map_err(|e| CiviumError::Network(e.to_string()))?;

        Ok(Self {
            request: signed_request.payload.clone(),
            sig_request_b58: signed_request.sig_b58.clone(),
            acceptance,
            sig_acceptance_b58: bs58::encode(sig_b).into_string(),
        })
    }

    /// Verify both signatures. Returns `Ok(())` if both are valid.
    pub fn verify(&self) -> Result<(), CiviumError> {
        let req_bytes = serde_json::to_vec(&self.request)
            .map_err(|e| CiviumError::Network(e.to_string()))?;
        verify_ed25519_sig(&self.request.from_pubkey_b58, &req_bytes, &self.sig_request_b58)?;

        let acc_bytes = serde_json::to_vec(&self.acceptance)
            .map_err(|e| CiviumError::Network(e.to_string()))?;
        verify_ed25519_sig(
            &self.acceptance.from_pubkey_b58,
            &acc_bytes,
            &self.sig_acceptance_b58,
        )?;

        // Nonces must match — ties acceptance to the original request
        if self.acceptance.request_nonce_b58 != self.request.nonce_b58 {
            return Err(CiviumError::Network(
                "APC nonce mismatch — acceptance does not match the request".into(),
            ));
        }

        Ok(())
    }
}

fn verify_ed25519_sig(pubkey_b58: &str, msg: &[u8], sig_b58: &str) -> Result<(), CiviumError> {
    let pubkey_bytes: [u8; 32] = bs58::decode(pubkey_b58)
        .into_vec()
        .map_err(|e| CiviumError::Network(e.to_string()))?
        .try_into()
        .map_err(|_| CiviumError::Network("invalid public key length in APC".into()))?;

    let ed_pub = libp2p::identity::ed25519::PublicKey::try_from_bytes(&pubkey_bytes)
        .map_err(|e| CiviumError::Network(e.to_string()))?;
    let lib_pub = libp2p::identity::PublicKey::from(ed_pub);

    let sig = bs58::decode(sig_b58)
        .into_vec()
        .map_err(|e| CiviumError::Network(e.to_string()))?;

    if !lib_pub.verify(msg, &sig) {
        return Err(CiviumError::Network("invalid APC signature".into()));
    }

    Ok(())
}

fn unix_now() -> u64 { crate::time::unix_now() }
