use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{crypto::GroupKey, CiviumError};

use super::message::{EncryptedChunk, Message, MessageKind};

/// Sliding-window rate limit: max messages per author per window.
const RATE_LIMIT_WINDOW_SECS: u64 = 60;
const RATE_LIMIT_MAX_MSGS: usize = 60;

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
    /// In-memory sliding-window rate counter: author → Vec<sent_at timestamps>.
    /// Not serialized — resets on restart (intentional: limit bursts, not total).
    #[serde(skip)]
    rate_buckets: HashMap<String, Vec<u64>>,
}

impl Mailbox {
    fn check_rate_limit(&mut self, author: &str, now: u64) -> Result<(), CiviumError> {
        let bucket = self.rate_buckets.entry(author.to_string()).or_default();
        // Evict timestamps outside the window
        bucket.retain(|&ts| now.saturating_sub(ts) < RATE_LIMIT_WINDOW_SECS);
        if bucket.len() >= RATE_LIMIT_MAX_MSGS {
            return Err(CiviumError::RateLimited(format!(
                "author {author} exceeded {RATE_LIMIT_MAX_MSGS} messages/{RATE_LIMIT_WINDOW_SECS}s"
            )));
        }
        bucket.push(now);
        Ok(())
    }
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
        let now = unix_now();
        self.check_rate_limit(&author_cid_short, now)?;
        let (nonce_b58, ciphertext_b58) = group_key.encrypt(body.as_bytes())?;
        let msg = Message {
            id: nonce_b58.clone(),
            author_cid_short,
            kind,
            nonce_b58,
            ciphertext_b58,
            sent_at: now,
        };
        self.messages.push(msg);
        Ok(self.messages.last().unwrap())
    }

    const CHUNK_SIZE: usize = 65_536; // 64 KB
    const MAX_FILE_BYTES: usize = 52_428_800; // 50 MB

    /// Encrypt a binary file in 64 KB chunks and append as a File message.
    pub fn post_file(
        &mut self,
        author_cid_short: String,
        filename: String,
        mime_type: String,
        data: &[u8],
        group_key: &GroupKey,
    ) -> Result<&Message, CiviumError> {
        self.check_rate_limit(&author_cid_short, unix_now())?;
        if data.len() > Self::MAX_FILE_BYTES {
            return Err(CiviumError::Messaging("fichier trop volumineux (max 50 Mo)".into()));
        }
        let chunks: Vec<EncryptedChunk> = data
            .chunks(Self::CHUNK_SIZE)
            .enumerate()
            .map(|(i, chunk)| {
                let (nonce_b58, ciphertext_b58) = group_key.encrypt(chunk)?;
                Ok(EncryptedChunk { index: i as u32, nonce_b58, ciphertext_b58 })
            })
            .collect::<Result<_, CiviumError>>()?;
        // ID = nonce of an empty encryption (random, unique per message)
        let (id, _) = group_key.encrypt(b"")?;
        let msg = Message {
            id,
            author_cid_short,
            kind: MessageKind::File { filename, mime_type, size_bytes: data.len() as u64, chunks },
            nonce_b58: String::new(),
            ciphertext_b58: String::new(),
            sent_at: unix_now(),
        };
        self.messages.push(msg);
        Ok(self.messages.last().unwrap())
    }

    /// Decrypt and reassemble all chunks of a File message.
    pub fn decrypt_file(&self, msg: &Message, group_key: &GroupKey) -> Result<Vec<u8>, CiviumError> {
        if let MessageKind::File { chunks, .. } = &msg.kind {
            let mut data = Vec::new();
            let mut sorted = chunks.clone();
            sorted.sort_by_key(|c| c.index);
            for chunk in &sorted {
                let bytes = group_key.decrypt(&chunk.nonce_b58, &chunk.ciphertext_b58)?;
                data.extend_from_slice(&bytes);
            }
            Ok(data)
        } else {
            Err(CiviumError::Messaging("ce message n'est pas un fichier".into()))
        }
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
