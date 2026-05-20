use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A shared document belonging to a Civium network.
/// The body is stored encrypted with the network group key.
///
/// Conflict resolution uses a LWW-Register (Last-Write-Wins) based on
/// `lamport_clock`. On equal clocks, the document with the lexicographically
/// smaller `id` wins (deterministic tiebreak).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub network_cid_short: String,
    pub title: String,
    /// ChaCha20-Poly1305 nonce (base58) for the encrypted body.
    pub nonce_b58: String,
    /// Encrypted body (base58).
    pub body_ciphertext: String,
    /// Monotonically increasing Lamport clock — incremented on each edit.
    /// Use this (not `updated_at`) for conflict resolution.
    pub lamport_clock: u64,
    /// Legacy field kept for display purposes only — NOT used for LWW ordering.
    pub version: u32,
    pub created_by: String,
    pub last_edited_by: String,
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
            lamport_clock: 1,
            version: 1,
            created_by: created_by.clone(),
            last_edited_by: created_by,
            created_at: now,
            updated_at: now,
        }
    }

    /// LWW merge: returns the winner between `self` and `other`.
    /// Higher `lamport_clock` wins; ties broken by smaller `id` (lexicographic).
    pub fn merge(self, other: Document) -> Document {
        if self.lamport_clock > other.lamport_clock {
            self
        } else if other.lamport_clock > self.lamport_clock {
            other
        } else if self.id <= other.id {
            self
        } else {
            other
        }
    }

    /// Produce an updated version of this document with an incremented clock.
    /// The caller supplies the new encrypted body, title and editor CID.
    pub fn update(
        &self,
        title: String,
        nonce_b58: String,
        body_ciphertext: String,
        edited_by: String,
        remote_clock: u64,
    ) -> Document {
        let now = unix_now();
        Document {
            id:              self.id.clone(),
            network_cid_short: self.network_cid_short.clone(),
            title,
            nonce_b58,
            body_ciphertext,
            lamport_clock:   self.lamport_clock.max(remote_clock) + 1,
            version:         self.version + 1,
            created_by:      self.created_by.clone(),
            last_edited_by:  edited_by,
            created_at:      self.created_at,
            updated_at:      now,
        }
    }
}

fn unix_now() -> u64 { crate::time::unix_now() }
fn uuid() -> String { Uuid::new_v4().to_string() }
