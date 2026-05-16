//! Messaging types — decrypted messages from the local mailbox.

/// A decrypted message in a Civium network.
#[derive(Debug, Clone)]
pub struct MessageInfo {
    /// Unique message ID.
    pub id: String,
    /// Short CID of the author.
    pub author_cid_short: String,
    /// Display name of the author in this network.
    pub author_name: String,
    /// Decrypted message body.
    pub body: String,
    /// Unix timestamp (seconds) when the message was sent.
    pub sent_at: u64,
    /// `true` if this is a direct (non-broadcast) message.
    pub is_direct: bool,
}
