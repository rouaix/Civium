use serde::{Deserialize, Serialize};

// ── Garde-fou majoritaire ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminActionKind {
    MemberAdmitted { member_cid_short: String, display_name: String },
    MemberRejected { member_cid_short: String },
    MemberBanned   { member_cid_short: String },
}

impl std::fmt::Display for AdminActionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdminActionKind::MemberAdmitted { member_cid_short, display_name } =>
                write!(f, "Admission de {} ({})", display_name, member_cid_short),
            AdminActionKind::MemberRejected { member_cid_short } =>
                write!(f, "Rejet de {}", member_cid_short),
            AdminActionKind::MemberBanned { member_cid_short } =>
                write!(f, "Bannissement de {}", member_cid_short),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminActionStatus {
    /// Within the contest window, no majority against yet.
    Active,
    /// Contest window expired without majority against.
    Confirmed,
    /// Majority contested → a Proposal has been auto-created.
    Suspended { proposal_id: String },
    /// The auto-vote decided to reverse the action.
    Reversed,
    /// The auto-vote upheld the action.
    Upheld,
}

impl std::fmt::Display for AdminActionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdminActionStatus::Active => write!(f, "active"),
            AdminActionStatus::Confirmed => write!(f, "confirmed"),
            AdminActionStatus::Suspended { .. } => write!(f, "suspended"),
            AdminActionStatus::Reversed => write!(f, "reversed"),
            AdminActionStatus::Upheld => write!(f, "upheld"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAction {
    pub id: String,
    pub network_cid_short: String,
    pub kind: AdminActionKind,
    pub taken_by: String,
    pub taken_at: u64,
    /// Seconds after `taken_at` during which members can contest (0 = 24 h default).
    pub contest_window_secs: u64,
    /// CID shorts of members who have contested this action.
    pub contests: Vec<String>,
    pub status: AdminActionStatus,
}

impl AdminAction {
    pub fn new(
        network_cid_short: String,
        kind: AdminActionKind,
        taken_by: String,
        taken_at: u64,
        contest_window_secs: u64,
    ) -> Self {
        let raw = format!("{network_cid_short}:{taken_by}:{taken_at}:{kind}");
        let hash = blake3::hash(raw.as_bytes());
        let id = bs58::encode(hash.as_bytes()).into_string()[..12].to_string();
        AdminAction {
            id,
            network_cid_short,
            kind,
            taken_by,
            taken_at,
            contest_window_secs: if contest_window_secs == 0 { 86_400 } else { contest_window_secs },
            contests: Vec::new(),
            status: AdminActionStatus::Active,
        }
    }

    pub fn is_window_open(&self, now: u64) -> bool {
        now < self.taken_at + self.contest_window_secs
    }

    /// Returns true if the majority threshold is reached (strictly more than half).
    pub fn majority_contested(&self, total_members: usize) -> bool {
        total_members > 0 && self.contests.len() * 2 > total_members
    }
}

/// Add a contest from `voter_cid_short`. Returns whether the majority threshold
/// was just reached (caller should then create a suspension Proposal).
pub fn add_contest(action: &mut AdminAction, voter_cid_short: &str, total_members: usize) -> bool {
    if action.contests.iter().any(|c| c == voter_cid_short) {
        return false;
    }
    action.contests.push(voter_cid_short.to_string());
    action.majority_contested(total_members)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Open,
    Closed,
    Cancelled,
}

impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Open => write!(f, "open"),
            ProposalStatus::Closed => write!(f, "closed"),
            ProposalStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub network_cid_short: String,
    pub title: String,
    pub description: String,
    pub options: Vec<String>,
    pub created_by: String,
    pub created_at: u64,
    pub closes_at: u64,
    /// Minimum percentage of members that must vote for the result to be valid (0 = no quorum).
    pub quorum_percent: u8,
    pub status: ProposalStatus,
}

impl Proposal {
    pub fn new(
        network_cid_short: String,
        title: String,
        description: String,
        options: Vec<String>,
        created_by: String,
        created_at: u64,
        closes_at: u64,
        quorum_percent: u8,
    ) -> Self {
        let id = {
            let raw = format!("{network_cid_short}:{title}:{created_at}");
            let hash = blake3::hash(raw.as_bytes());
            bs58::encode(hash.as_bytes()).into_string()[..12].to_string()
        };
        Proposal {
            id,
            network_cid_short,
            title,
            description,
            options,
            created_by,
            created_at,
            closes_at,
            quorum_percent,
            status: ProposalStatus::Open,
        }
    }

    pub fn is_expired(&self, now: u64) -> bool {
        self.closes_at > 0 && now >= self.closes_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: String,
    pub voter_cid_short: String,
    pub choice_index: usize,
    pub cast_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionResult {
    pub label: String,
    pub votes: usize,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResult {
    pub proposal_id: String,
    pub total_votes: usize,
    pub total_members: usize,
    pub participation_percent: f64,
    pub quorum_reached: bool,
    pub options: Vec<OptionResult>,
    /// Index of the winning option, or None if tied or quorum not reached.
    pub winner: Option<usize>,
}

pub fn compute_result(proposal: &Proposal, votes: &[Vote], total_members: usize) -> VoteResult {
    let total_votes = votes.len();
    let participation_percent = if total_members > 0 {
        (total_votes as f64 / total_members as f64) * 100.0
    } else {
        0.0
    };
    let quorum_reached = proposal.quorum_percent == 0
        || participation_percent >= proposal.quorum_percent as f64;

    let mut counts = vec![0usize; proposal.options.len()];
    for vote in votes {
        if vote.choice_index < counts.len() {
            counts[vote.choice_index] += 1;
        }
    }

    let options: Vec<OptionResult> = proposal
        .options
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let v = counts[i];
            let percent = if total_votes > 0 {
                (v as f64 / total_votes as f64) * 100.0
            } else {
                0.0
            };
            OptionResult { label: label.clone(), votes: v, percent }
        })
        .collect();

    let winner = if !quorum_reached || total_votes == 0 {
        None
    } else {
        let max = counts.iter().copied().max().unwrap_or(0);
        let winners: Vec<_> = counts.iter().enumerate().filter(|(_, &v)| v == max).collect();
        if winners.len() == 1 { Some(winners[0].0) } else { None }
    };

    VoteResult {
        proposal_id: proposal.id.clone(),
        total_votes,
        total_members,
        participation_percent,
        quorum_reached,
        options,
        winner,
    }
}
