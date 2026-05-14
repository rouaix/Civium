use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;
use rand::rngs::OsRng;

use crate::CiviumError;

/// Symmetric group key (ChaCha20-Poly1305) shared among network members.
///
/// Encrypts messages for circles 0-2. Every admitted member holds a copy.
/// Circle 3 (pair E2E) uses a separate key — not yet implemented (Phase 3).
pub struct GroupKey {
    key: [u8; 32],
    cipher: ChaCha20Poly1305,
}

impl GroupKey {
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
        Self { key, cipher }
    }

    pub fn from_b58(s: &str) -> Result<Self, CiviumError> {
        if s.is_empty() {
            return Err(CiviumError::Crypto(
                "no group key — network created before week 5, please recreate it".into(),
            ));
        }
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| CiviumError::Crypto(e.to_string()))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| CiviumError::Crypto("group key must be 32 bytes".into()))?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&arr));
        Ok(Self { key: arr, cipher })
    }

    pub fn to_b58(&self) -> String {
        bs58::encode(&self.key).into_string()
    }

    /// Encrypt `plaintext`. Returns `(nonce_b58, ciphertext_b58)`.
    ///
    /// The nonce is random and unique per call — doubles as the message ID.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(String, String), CiviumError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CiviumError::Crypto(e.to_string()))?;
        Ok((
            bs58::encode(&nonce_bytes).into_string(),
            bs58::encode(&ciphertext).into_string(),
        ))
    }

    /// Decrypt a message encrypted by `encrypt`.
    pub fn decrypt(&self, nonce_b58: &str, ciphertext_b58: &str) -> Result<Vec<u8>, CiviumError> {
        let nonce_bytes = bs58::decode(nonce_b58)
            .into_vec()
            .map_err(|e| CiviumError::Crypto(e.to_string()))?;
        let ciphertext = bs58::decode(ciphertext_b58)
            .into_vec()
            .map_err(|e| CiviumError::Crypto(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        self.cipher
            .decrypt(nonce, ciphertext.as_slice())
            .map_err(|_| CiviumError::Crypto("decryption failed — wrong key or corrupted data".into()))
    }
}
