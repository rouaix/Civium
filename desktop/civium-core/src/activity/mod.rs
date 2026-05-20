use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kind of activity that occurred in a network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityKind {
    MemberJoined,
    MemberLeft,
    MessagePosted,
    DirectMessageSent,
    ProposalCreated,
    VoteCast,
    AdminActionTaken,
    AdminActionContested,
    ConnectionEstablished,
    AgendaEventCreated,
    DocumentCreated,
    MessageReported,
}

impl std::fmt::Display for ActivityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::MemberJoined           => "member_joined",
            Self::MemberLeft             => "member_left",
            Self::MessagePosted          => "message_posted",
            Self::DirectMessageSent      => "direct_message_sent",
            Self::ProposalCreated        => "proposal_created",
            Self::VoteCast               => "vote_cast",
            Self::AdminActionTaken       => "admin_action_taken",
            Self::AdminActionContested   => "admin_action_contested",
            Self::ConnectionEstablished  => "connection_established",
            Self::AgendaEventCreated     => "agenda_event_created",
            Self::DocumentCreated        => "document_created",
            Self::MessageReported        => "message_reported",
        };
        f.write_str(s)
    }
}

/// An immutable record of something that happened in a network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub id: String,
    pub network_cid_short: String,
    pub kind: ActivityKind,
    pub actor_cid_short: String,
    pub summary: String,
    pub occurred_at: u64,
}

impl ActivityEvent {
    pub fn new(
        network_cid_short: String,
        kind: ActivityKind,
        actor_cid_short: String,
        summary: String,
    ) -> Self {
        Self {
            id: uuid(),
            network_cid_short,
            kind,
            actor_cid_short,
            summary,
            occurred_at: unix_now(),
        }
    }
}

/// A per-member notification pointing to an ActivityEvent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub network_cid_short: String,
    pub source_event_id: String,
    pub target_cid_short: String,
    pub read: bool,
    pub created_at: u64,
}

impl Notification {
    pub fn new(network_cid_short: String, source_event_id: String, target_cid_short: String) -> Self {
        Self {
            id: uuid(),
            network_cid_short,
            source_event_id,
            target_cid_short,
            read: false,
            created_at: unix_now(),
        }
    }
}

fn unix_now() -> u64 { crate::time::unix_now() }
fn uuid() -> String { Uuid::new_v4().to_string() }
