use serde::{Deserialize, Serialize};

use super::agreement::ShareAgreement;

/// Lifecycle state of a connection between two Civium networks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConnectionState {
    /// We sent a request; waiting for the peer admin to review.
    Requested,
    /// The peer sent a request; our admin is reviewing.
    Validating,
    /// APC signed by both sides; connection is live.
    Active,
    /// Request rejected. The reason is optionally included.
    Refused { reason: Option<String> },
    /// We blocked this network (hides from their view).
    Blocked,
    /// Was Active; unilaterally revoked by one side.
    Revoked,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Requested       => f.write_str("Requested"),
            Self::Validating      => f.write_str("Validating"),
            Self::Active          => f.write_str("Active"),
            Self::Refused { .. }  => f.write_str("Refused"),
            Self::Blocked         => f.write_str("Blocked"),
            Self::Revoked         => f.write_str("Revoked"),
        }
    }
}

/// What a network agrees to expose to its connection peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareTerms {
    /// Whether to expose the member directory to the connected network.
    pub expose_member_directory: bool,
    /// When `true`, this network is registered in the civium root directory but
    /// hidden from public listings (visible only to the root admin).
    #[serde(default)]
    pub privacy: bool,
}

impl Default for ShareTerms {
    fn default() -> Self {
        Self { expose_member_directory: true, privacy: false }
    }
}

/// A connection request payload, signed by the requesting network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPayload {
    pub v: u8,
    /// Unique per request — links request to acceptance in the APC.
    pub nonce_b58: String,
    pub from_cid_full: String,
    pub from_pubkey_b58: String,
    pub from_name: String,
    pub from_terms: ShareTerms,
    pub to_cid_full: String,
    pub created_at: u64,
}

/// A signed connection request (stored on the receiver's side during Validating).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedRequest {
    pub payload: RequestPayload,
    pub sig_b58: String,
}

/// An acceptance payload, signed by the accepting network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptPayload {
    pub v: u8,
    /// Must match the nonce in the original request.
    pub request_nonce_b58: String,
    pub from_cid_full: String,
    pub from_pubkey_b58: String,
    pub from_name: String,
    pub from_terms: ShareTerms,
    pub accepted_at: u64,
}

/// Persistent record of a network-to-network connection, stored per network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRecord {
    pub peer_cid_full: String,
    pub peer_cid_short: String,
    pub peer_name: String,
    pub peer_pubkey_b58: String,
    pub state: ConnectionState,
    pub initiated_at: u64,
    pub updated_at: u64,
    /// What this network exposes to the peer.
    pub our_terms: ShareTerms,
    /// What the peer exposes to this network (known once Active).
    pub their_terms: Option<ShareTerms>,
    /// Stored while state == Validating (the requester's signed request).
    pub incoming_request: Option<SignedRequest>,
    /// The signed Accord de Partage Civium — present once Active.
    pub apc: Option<ShareAgreement>,
}
