use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A guardian–minor link within a network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianLink {
    pub id: String,
    pub network_cid_short: String,
    pub minor_cid_short: String,
    pub guardian_cid_short: String,
    pub added_by: String,
    pub added_at: u64,
}

impl GuardianLink {
    pub fn new(
        network_cid_short: String,
        minor_cid_short: String,
        guardian_cid_short: String,
        added_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            network_cid_short,
            minor_cid_short,
            guardian_cid_short,
            added_by,
            added_at: unix_now(),
        }
    }
}

/// Configurable interaction restrictions for a minor member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinorRestrictions {
    pub network_cid_short: String,
    pub minor_cid_short: String,
    /// Maximum trust circle the minor can be involved in (0–2).
    pub max_circle: u8,
    /// CID shorts explicitly allowed to interact beyond max_circle
    /// (e.g., guardians already get full access; add others here).
    /// Empty = only guardians.
    #[serde(default)]
    pub allowed_cid_shorts: Vec<String>,
    pub updated_by: String,
    pub updated_at: u64,
}

impl MinorRestrictions {
    pub fn new(
        network_cid_short: String,
        minor_cid_short: String,
        max_circle: u8,
        allowed_cid_shorts: Vec<String>,
        updated_by: String,
    ) -> Self {
        Self {
            network_cid_short,
            minor_cid_short,
            max_circle: max_circle.min(2),
            allowed_cid_shorts,
            updated_by,
            updated_at: unix_now(),
        }
    }

    /// Returns true if `peer_cid_short` is allowed to interact with this minor
    /// at the given circle level (ignoring guardian links — those are checked separately).
    pub fn allows(&self, peer_cid_short: &str, circle: u8) -> bool {
        if circle <= self.max_circle {
            return true;
        }
        self.allowed_cid_shorts.iter().any(|c| c == peer_cid_short)
    }
}

fn unix_now() -> u64 { crate::time::unix_now() }
