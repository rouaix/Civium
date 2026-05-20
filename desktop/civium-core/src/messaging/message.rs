use serde::{Deserialize, Serialize};

/// A single encrypted chunk of a file attachment (64 KB block, group-key encrypted).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedChunk {
    pub index: u32,
    pub nonce_b58: String,
    pub ciphertext_b58: String,
}

/// Routing descriptor for a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageKind {
    /// Broadcast to the whole network thread.
    Thread,
    /// Direct message to a single member (group-key encrypted).
    Direct { to_cid_short: String },
    /// True end-to-end message — encrypted with the sender/recipient pair key.
    /// Only the two parties can decrypt, not even network admins.
    E2E { to_cid_full: String },
    /// Binary file attachment chunked and encrypted with the group key (circles 0-2).
    File {
        filename: String,
        mime_type: String,
        size_bytes: u64,
        chunks: Vec<EncryptedChunk>,
    },
    /// Reference to a calendar event shared in the network thread.
    CalendarEvent {
        title: String,
        start_at: u64,
        end_at: u64,
        location: Option<String>,
        description: Option<String>,
    },
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
    /// ID of the message being replied to (for threaded conversations).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<String>,
}
