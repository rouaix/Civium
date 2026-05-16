//! Governance types — proposals and votes.
//!
//! Each Civium network has a built-in governance plugin that cannot be disabled.
//! Proposals are created by members, voted on within a configurable window, and
//! the result is recorded in an immutable audit log.

/// A governance proposal in a Civium network.
#[derive(Debug, Clone)]
pub struct ProposalInfo {
    /// Unique proposal ID.
    pub id: String,
    /// Proposal title.
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// Available vote options (e.g. `["Pour", "Contre", "Abstention"]`).
    pub options: Vec<String>,
    /// Unix timestamp when the proposal was created.
    pub created_at: u64,
    /// Unix timestamp when voting closes (0 = no deadline).
    pub closes_at: u64,
    /// Short CID of the member who created the proposal.
    pub created_by: String,
    /// Current status: `"open"`, `"closed"`, `"suspended"`.
    pub status: String,
}

/// A single vote cast on a proposal.
#[derive(Debug, Clone)]
pub struct VoteInfo {
    /// Proposal this vote belongs to.
    pub proposal_id: String,
    /// Short CID of the voter.
    pub voter_cid_short: String,
    /// Index into the proposal's `options` array.
    pub choice_index: u32,
    /// Unix timestamp when the vote was cast.
    pub cast_at: u64,
}
