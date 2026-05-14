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

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
