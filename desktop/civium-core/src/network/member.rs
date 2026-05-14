use serde::{Deserialize, Serialize};

/// Access level a member holds within a network.
///
/// Trust is asymmetric between members (each assigns a circle to the other),
/// but in Phase 0 the admin sets a single network-wide circle at admission.
/// Per-relationship asymmetry is implemented in a later phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum TrustCircle {
    /// Annuaire (0): visible in the directory, name + existence only.
    Annuaire = 0,
    /// Connaissance (1): partial profile, basic messaging.
    Connaissance = 1,
    /// Confiance (2): full profile, content sharing, services.
    Confiance = 2,
}

impl TrustCircle {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Annuaire),
            1 => Some(Self::Connaissance),
            2 => Some(Self::Confiance),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Annuaire     => "Annuaire (0)",
            Self::Connaissance => "Connaissance (1)",
            Self::Confiance    => "Confiance (2)",
        }
    }
}

impl std::fmt::Display for TrustCircle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

impl std::str::FromStr for TrustCircle {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" | "annuaire"     => Ok(Self::Annuaire),
            "1" | "connaissance" => Ok(Self::Connaissance),
            "2" | "confiance"    => Ok(Self::Confiance),
            _ => Err(format!("unknown circle '{s}' — use 0, 1 or 2")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberRole {
    Admin,
    Member,
}

impl std::fmt::Display for MemberRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin  => f.write_str("admin"),
            Self::Member => f.write_str("member"),
        }
    }
}

impl std::str::FromStr for MemberRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin"  => Ok(Self::Admin),
            "member" => Ok(Self::Member),
            _ => Err(format!("unknown role '{s}' — use admin or member")),
        }
    }
}

/// A fully admitted member of a network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRecord {
    pub cid_short: String,
    pub cid_full: String,
    pub display_name: String,
    pub circle: TrustCircle,
    pub role: MemberRole,
    pub joined_at: u64,
}

/// A pending join request waiting for admin admission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingRecord {
    pub cid_short: String,
    pub cid_full: String,
    pub display_name: String,
    pub requested_at: u64,
    /// Nonce from the invitation used (to prevent replay).
    pub invite_nonce_b58: String,
}
