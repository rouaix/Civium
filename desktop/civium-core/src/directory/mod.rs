use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Network,
    Member,
}

impl std::fmt::Display for EntryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network => f.write_str("network"),
            Self::Member  => f.write_str("member"),
        }
    }
}

impl std::str::FromStr for EntryKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "network" => Ok(Self::Network),
            "member"  => Ok(Self::Member),
            _ => Err(format!("unknown kind '{s}' — use network or member")),
        }
    }
}

/// An entry in a Civium directory network — a catalogued network or member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub id: String,
    pub directory_cid_short: String,
    pub kind: EntryKind,
    pub subject_cid_short: String,
    pub subject_name: String,
    pub description: String,
    pub contact_addr: Option<String>,
    pub published_by: String,
    pub published_at: u64,
    pub tags: Vec<String>,
}

impl DirectoryEntry {
    pub fn new(
        directory_cid_short: String,
        kind: EntryKind,
        subject_cid_short: String,
        subject_name: String,
        description: String,
        contact_addr: Option<String>,
        published_by: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            directory_cid_short,
            kind,
            subject_cid_short,
            subject_name,
            description,
            contact_addr,
            published_by,
            published_at: unix_now(),
            tags,
        }
    }

    /// True if this entry matches a free-text query (name, description, CID, tags).
    pub fn matches(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.subject_name.to_lowercase().contains(&q)
            || self.description.to_lowercase().contains(&q)
            || self.subject_cid_short.contains(&q)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&q))
    }
}

/// A federation link between two directory networks.
/// The host directory `host_cid_short` trusts the peer directory
/// `peer_cid_short` and will include its entries in federated searches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedDirectory {
    pub id: String,
    pub host_cid_short: String,
    pub peer_cid_short: String,
    pub peer_name: String,
    /// Optional P2P multiaddr to reach the peer directory for live queries.
    pub peer_addr: Option<String>,
    pub added_by: String,
    pub added_at: u64,
}

impl FederatedDirectory {
    pub fn new(
        host_cid_short: String,
        peer_cid_short: String,
        peer_name: String,
        peer_addr: Option<String>,
        added_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            host_cid_short,
            peer_cid_short,
            peer_name,
            peer_addr,
            added_by,
            added_at: unix_now(),
        }
    }
}

/// An entry in a Registre des Réseaux Malveillants (RRM) — a reported malicious network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RrmEntry {
    pub id: String,
    pub rrm_cid_short: String,
    pub network_cid_short: String,
    pub network_name: String,
    pub reason: String,
    pub evidence_url: Option<String>,
    pub reported_by: String,
    pub reported_at: u64,
}

impl RrmEntry {
    pub fn new(
        rrm_cid_short: String,
        network_cid_short: String,
        network_name: String,
        reason: String,
        evidence_url: Option<String>,
        reported_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            rrm_cid_short,
            network_cid_short,
            network_name,
            reason,
            evidence_url,
            reported_by,
            reported_at: unix_now(),
        }
    }

    pub fn matches(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.network_name.to_lowercase().contains(&q)
            || self.network_cid_short.contains(&q)
            || self.reason.to_lowercase().contains(&q)
    }
}

/// A trust relationship: a standard network that consults a specific RRM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedRrm {
    pub id: String,
    pub network_cid_short: String,
    pub rrm_cid_short: String,
    pub rrm_name: String,
    pub added_by: String,
    pub added_at: u64,
}

impl TrustedRrm {
    pub fn new(
        network_cid_short: String,
        rrm_cid_short: String,
        rrm_name: String,
        added_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            network_cid_short,
            rrm_cid_short,
            rrm_name,
            added_by,
            added_at: unix_now(),
        }
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
