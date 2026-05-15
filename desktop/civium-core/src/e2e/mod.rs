use crate::CiviumError;

const E2E_CONTEXT: &str = "civium e2e pair v0";

/// Symmetric encryption key derived from an X25519 DH exchange between two Civium members.
///
/// Ed25519 keys are converted to X25519 (Curve25519 Montgomery form) via SHA-512 expansion
/// + clamping (standard Signal/libsodium approach). Both parties independently derive the
/// same 32-byte secret from their own private key and the peer's public key.
pub struct PairKey {
    key: [u8; 32],
}

impl PairKey {
    /// Derive the shared pair encryption key.
    ///
    /// - `our_secret_bytes` — raw 32-byte Ed25519 secret (from `CiviumKeypair::secret_bytes`)
    /// - `their_pub_key_bytes` — raw 32-byte Ed25519 public key (from `MemberRecord::pub_key_b58`)
    pub fn derive(
        our_secret_bytes: &[u8; 32],
        their_pub_key_bytes: &[u8; 32],
    ) -> Result<Self, CiviumError> {
        let dh = ed25519_x25519_dh(our_secret_bytes, their_pub_key_bytes)
            .map_err(|e| CiviumError::Crypto(format!("E2E DH failed: {e}")))?;
        let key = blake3::derive_key(E2E_CONTEXT, &dh);
        Ok(Self { key })
    }

    /// Encrypt `plaintext` with the pair key. Returns (nonce_b58, ciphertext_b58).
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(String, String), CiviumError> {
        use chacha20poly1305::{
            aead::{Aead, AeadCore, OsRng},
            ChaCha20Poly1305, KeyInit,
        };
        let cipher = ChaCha20Poly1305::new((&self.key).into());
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| CiviumError::Crypto(e.to_string()))?;
        Ok((
            bs58::encode(nonce.as_slice()).into_string(),
            bs58::encode(&ciphertext).into_string(),
        ))
    }

    /// Decrypt a pair-encrypted message.
    pub fn decrypt(&self, nonce_b58: &str, ciphertext_b58: &str) -> Result<Vec<u8>, CiviumError> {
        use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce};
        let nonce_bytes = bs58::decode(nonce_b58)
            .into_vec()
            .map_err(|e| CiviumError::Crypto(e.to_string()))?;
        let ct_bytes = bs58::decode(ciphertext_b58)
            .into_vec()
            .map_err(|e| CiviumError::Crypto(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let cipher = ChaCha20Poly1305::new((&self.key).into());
        cipher
            .decrypt(nonce, ct_bytes.as_ref())
            .map_err(|e| CiviumError::Crypto(format!("E2E decrypt failed: {e}")))
    }
}

/// Convert an Ed25519 keypair to X25519 and perform DH with `their_ed25519_pubkey`.
///
/// Conversion follows the standard Signal/libsodium method:
///   - Secret: expand via SHA-512, take first 32 bytes, clamp (RFC 7748 §5)
///   - Public: decompress Edwards point, convert to Montgomery via birational map
fn ed25519_x25519_dh(
    our_ed25519_secret: &[u8; 32],
    their_ed25519_pubkey: &[u8; 32],
) -> Result<[u8; 32], String> {
    use curve25519_dalek::{
        edwards::CompressedEdwardsY, montgomery::MontgomeryPoint, scalar::Scalar,
    };
    use sha2::{Digest, Sha512};

    // Step 1: Ed25519 secret → X25519 scalar (SHA-512 + clamp)
    let hash = Sha512::digest(our_ed25519_secret);
    let mut scalar_bytes = [0u8; 32];
    scalar_bytes.copy_from_slice(&hash[..32]);
    // RFC 7748 clamping
    scalar_bytes[0] &= 248;
    scalar_bytes[31] &= 127;
    scalar_bytes[31] |= 64;
    let our_scalar = Scalar::from_bytes_mod_order(scalar_bytes);

    // Step 2: Ed25519 public key (compressed Edwards) → Montgomery (X25519) point
    let compressed = CompressedEdwardsY(*their_ed25519_pubkey);
    let edwards = compressed
        .decompress()
        .ok_or_else(|| "invalid Ed25519 public key (not on curve)".to_string())?;
    let montgomery: MontgomeryPoint = edwards.to_montgomery();

    // Step 3: DH — our scalar × their Montgomery point
    let shared = our_scalar * montgomery;
    Ok(shared.to_bytes())
}
