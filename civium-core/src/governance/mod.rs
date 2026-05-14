use serde::{Deserialize, Serialize};

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
