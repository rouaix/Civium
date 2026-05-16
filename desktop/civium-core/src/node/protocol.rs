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
    /// Broadcast a signed RCC fraud alert to this node.
    BroadcastAlert { payload_json: String, signature_b58: String },
    /// Propose an inter-network APC (Accord de Partage Civium) to a peer node.
    ///
    /// `signed_request_json` is a serialised [`crate::connection::SignedRequest`].
    ConnectRequest { signed_request_json: String },
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
    /// APC accepted — `apc_json` is a serialised [`crate::connection::ShareAgreement`]
    /// containing both the original request and the acceptance, both signed.
    ConnectAccepted { apc_json: String },
    /// APC rejected by the receiving network.
    ConnectRejected { reason: String },
    Error { message: String },
}
