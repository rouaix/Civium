use serde::{Deserialize, Serialize};

use crate::network::{MemberRecord, NetworkData};
use crate::messaging::Message;

/// Requests sent between Civium nodes over the `/civium/1.0.0` protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CiviumRequest {
    /// Ask a node to admit us as a member of one of its networks.
    Join {
        /// The `civium-invite:…` link authorising this join.
        invite_link: String,
        /// Full CID of the joining member.
        member_cid_full: String,
        /// Desired display name in the network.
        display_name: String,
    },
    /// Pull state updates (members + messages) added since `since_ts`.
    Sync {
        network_cid_full: String,
        /// Unix timestamp — only items newer than this are returned.
        since_ts: u64,
    },
    Ping,
}

/// Responses returned by a Civium node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CiviumResponse {
    /// Join accepted — full network snapshot including the group key.
    JoinAccepted { network_data: NetworkData },
    /// Join refused by the remote node.
    JoinRejected { reason: String },
    /// Sync data: members and messages newer than the requested timestamp.
    SyncData {
        /// Echo of the requested network CID — lets the receiver correlate without tracking request IDs.
        network_cid_full: String,
        members: Vec<MemberRecord>,
        messages: Vec<Message>,
    },
    Pong,
    Error { message: String },
}
