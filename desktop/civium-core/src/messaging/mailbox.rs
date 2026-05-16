use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{crypto::GroupKey, CiviumError};

use super::message::{Message, MessageKind};

/// Per-network message store — a Grow-only Set (G-Set) CRDT.
///
/// Merge semantics: union by message `id`. No deletions in Phase 0.
/// Transport (P2P sync) is wired up in weeks 7-8; until then the mailbox
/// is local-only and the outbox holds messages queued for future broadcast.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Mailbox {
    /// All messages received or confirmed sent.
    pub messages: Vec<Message>,
    /// Messages created locally but not yet transmitted to peers.
    pub outbox: Vec<Message>,
}

impl Mailbox {
    pub fn new() -> Self {
        Self::default()
    }

    /// Encrypt `body` with the network group key and append to `messages`.
    pub fn post(
        &mut self,
        author_cid_short: String,
        kind: MessageKind,
        body: &str,
        group_key: &GroupKey,
    ) -> Result<&Message, CiviumError> {
        let (nonce_b58, ciphertext_b58) = group_key.encrypt(body.as_bytes())?;
        let msg = Message {
            id: nonce_b58.clone(),
            author_cid_short,
            kind,
            nonce_b58,
            ciphertext_b58,
            sent_at: unix_now(),
        };
        self.messages.push(msg);
        Ok(self.messages.last().unwrap())
    }

    /// Decrypt and return the plaintext body of a message.
    pub fn decrypt_body(&self, msg: &Message, group_key: &GroupKey) -> Result<String, CiviumError> {
        let bytes = group_key.decrypt(&msg.nonce_b58, &msg.ciphertext_b58)?;
        String::from_utf8(bytes)
            .map_err(|e| CiviumError::Messaging(format!("invalid UTF-8 body: {e}")))
    }

    /// G-Set merge: absorb messages from `other` that this mailbox does not have.
    ///
    /// After merge, `messages` is sorted by `sent_at`.
    pub fn merge(&mut self, other: &Mailbox) {
        let existing: HashSet<_> = self.messages.iter().map(|m| m.id.clone()).collect();
        for msg in &other.messages {
            if !existing.contains(&msg.id) {
                self.messages.push(msg.clone());
            }
        }
        self.messages.sort_by_key(|m| m.sent_at);
    }

    /// Iterate network thread messages, oldest first.
    pub fn thread_messages(&self) -> impl Iterator<Item = &Message> {
        self.messages
            .iter()
            .filter(|m| m.kind == MessageKind::Thread)
    }

    /// Iterate direct messages exchanged between `local_cid` and `peer_cid`.
    pub fn direct_messages<'a>(
        &'a self,
        local_cid: &'a str,
        peer_cid: &'a str,
    ) -> impl Iterator<Item = &'a Message> + 'a {
        self.messages.iter().filter(move |m| match &m.kind {
            MessageKind::Direct { to_cid_short } => {
                let author = m.author_cid_short.as_str();
                let to = to_cid_short.as_str();
                (author == local_cid && to == peer_cid)
                    || (author == peer_cid && to == local_cid)
            }
            _ => false,
        })
    }
}

fn unix_now() -> u64 { crate::time::unix_now() }
