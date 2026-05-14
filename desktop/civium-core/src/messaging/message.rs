use serde::{Deserialize, Serialize};

/// Routing descriptor for a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageKind {
    /// Broadcast to the whole network thread.
    Thread,
    /// Direct message to a single member.
    Direct { to_cid_short: String },
}

/// An encrypted message persisted in a network mailbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID — the base58 nonce used during encryption.
    pub id: String,
    /// CID short of the author.
    pub author_cid_short: String,
    pub kind: MessageKind,
    /// ChaCha20-Poly1305 nonce (base58) — also used as the CRDT identity key.
    pub nonce_b58: String,
    /// Encrypted body + Poly1305 tag (base58).
    pub ciphertext_b58: String,
    /// Unix timestamp in seconds.
    pub sent_at: u64,
}
