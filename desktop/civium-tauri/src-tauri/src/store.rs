//! SQLite store for the Tauri app — same schema as civium-cli.
//! Both apps share the same civium.db file under the data directory.

use anyhow::{Context, Result};
use civium_core::{network::Network, AdminAction, CiviumKeypair, DirectoryEntry, MemberRecord, Message, Proposal, Vote, VoteDelegation};
use rusqlite::{params, Connection};
use std::path::Path;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS identity (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    secret_b58  TEXT    NOT NULL,
    cid_short   TEXT    NOT NULL,
    cid_full    TEXT    NOT NULL
);
CREATE TABLE IF NOT EXISTS networks (
    cid_short   TEXT PRIMARY KEY,
    data_json   TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS connections (
    network_cid     TEXT NOT NULL,
    peer_cid_full   TEXT NOT NULL,
    record_json     TEXT NOT NULL,
    PRIMARY KEY (network_cid, peer_cid_full)
);
CREATE TABLE IF NOT EXISTS messages (
    network_cid     TEXT    NOT NULL,
    message_id      TEXT    NOT NULL,
    message_json    TEXT    NOT NULL,
    in_outbox       INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (network_cid, message_id)
);
CREATE TABLE IF NOT EXISTS proposals (
    network_cid     TEXT NOT NULL,
    proposal_id     TEXT NOT NULL,
    proposal_json   TEXT NOT NULL,
    PRIMARY KEY (network_cid, proposal_id)
);
CREATE TABLE IF NOT EXISTS votes (
    proposal_id     TEXT NOT NULL,
    voter_cid_short TEXT NOT NULL,
    vote_json       TEXT NOT NULL,
    PRIMARY KEY (proposal_id, voter_cid_short)
);
CREATE TABLE IF NOT EXISTS admin_actions (
    network_cid     TEXT NOT NULL,
    action_id       TEXT NOT NULL,
    action_json     TEXT NOT NULL,
    PRIMARY KEY (network_cid, action_id)
);
CREATE TABLE IF NOT EXISTS vote_delegations (
    network_cid         TEXT NOT NULL,
    delegator_cid_short TEXT NOT NULL,
    proposal_id         TEXT,
    delegation_json     TEXT NOT NULL,
    PRIMARY KEY (network_cid, delegator_cid_short, COALESCE(proposal_id, ''))
);
CREATE TABLE IF NOT EXISTS directory_entries (
    directory_cid   TEXT NOT NULL,
    entry_id        TEXT NOT NULL,
    entry_json      TEXT NOT NULL,
    PRIMARY KEY (directory_cid, entry_id)
);
";

pub fn open_db(data_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(data_dir)?;
    let conn = Connection::open(data_dir.join("civium.db"))?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

// ── Identity ──────────────────────────────────────────────────────────────────

pub fn identity_exists(conn: &Connection) -> bool {
    conn.query_row("SELECT COUNT(*) FROM identity", [], |r| r.get::<_, i64>(0))
        .unwrap_or(0)
        > 0
}

pub fn save_identity(conn: &Connection, keypair: &CiviumKeypair) -> Result<()> {
    let cid = keypair.cid();
    conn.execute(
        "INSERT OR REPLACE INTO identity (id, secret_b58, cid_short, cid_full)
         VALUES (1, ?1, ?2, ?3)",
        params![keypair.secret_b58(), cid.short(), cid.full()],
    )?;
    Ok(())
}

pub fn load_identity(conn: &Connection) -> Result<CiviumKeypair> {
    let secret: String = conn
        .query_row("SELECT secret_b58 FROM identity WHERE id = 1", [], |r| r.get(0))
        .context("no identity found")?;
    CiviumKeypair::from_secret_b58(&secret).map_err(|e| anyhow::anyhow!("{e}"))
}

// ── Network ───────────────────────────────────────────────────────────────────

pub fn save_network(conn: &Connection, network: &Network) -> Result<()> {
    let json = serde_json::to_string(&network.data)?;
    conn.execute(
        "INSERT OR REPLACE INTO networks (cid_short, data_json) VALUES (?1, ?2)",
        params![network.cid_short(), json],
    )?;
    Ok(())
}

pub fn list_networks(conn: &Connection) -> Result<Vec<Network>> {
    let mut stmt = conn.prepare("SELECT data_json FROM networks")?;
    let mut rows = stmt.query([])?;
    let mut networks = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let data = serde_json::from_str(&json)?;
        let net = Network::from_data(data).map_err(|e| anyhow::anyhow!("{e}"))?;
        networks.push(net);
    }
    Ok(networks)
}

pub fn load_network(conn: &Connection, cid_short: &str) -> Result<Network> {
    let json: String = conn
        .query_row(
            "SELECT data_json FROM networks WHERE cid_short = ?1",
            params![cid_short],
            |r| r.get(0),
        )
        .with_context(|| format!("network '{cid_short}' not found"))?;
    let data = serde_json::from_str(&json)?;
    Network::from_data(data).map_err(|e| anyhow::anyhow!("{e}"))
}

// ── Members (convenience) ─────────────────────────────────────────────────────

pub fn network_members(network: &Network) -> &[MemberRecord] {
    &network.data.members
}

// ── Sync helpers ──────────────────────────────────────────────────────────────

/// Find a network by its full CID (searches all stored networks).
pub fn find_network_by_full_cid(conn: &Connection, cid_full: &str) -> Result<Network> {
    let networks = list_networks(conn)?;
    networks
        .into_iter()
        .find(|n| n.cid_full() == cid_full)
        .ok_or_else(|| anyhow::anyhow!("network '{}' not found", cid_full))
}

/// Load all inbox messages for a network.
pub fn load_messages(conn: &Connection, network_cid_short: &str) -> Result<Vec<Message>> {
    let mut stmt = conn.prepare(
        "SELECT message_json FROM messages
         WHERE network_cid = ?1 AND in_outbox = 0
         ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut messages = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let msg: Message = serde_json::from_str(&json)?;
        messages.push(msg);
    }
    Ok(messages)
}

/// Persist a single message (thread or direct) into the messages table.
pub fn save_message(conn: &Connection, network_cid_short: &str, msg: &Message) -> Result<()> {
    let json = serde_json::to_string(msg)?;
    conn.execute(
        "INSERT OR IGNORE INTO messages (network_cid, message_id, message_json, in_outbox)
         VALUES (?1, ?2, ?3, 0)",
        params![network_cid_short, &msg.id, json],
    )?;
    Ok(())
}

// ── Governance ────────────────────────────────────────────────────────────────

pub fn save_proposal(conn: &Connection, network_cid_short: &str, proposal: &Proposal) -> Result<()> {
    let json = serde_json::to_string(proposal)?;
    conn.execute(
        "INSERT OR REPLACE INTO proposals (network_cid, proposal_id, proposal_json)
         VALUES (?1, ?2, ?3)",
        params![network_cid_short, &proposal.id, json],
    )?;
    Ok(())
}

pub fn list_proposals(conn: &Connection, network_cid_short: &str) -> Result<Vec<Proposal>> {
    let mut stmt = conn.prepare(
        "SELECT proposal_json FROM proposals WHERE network_cid = ?1 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut proposals = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let p: Proposal = serde_json::from_str(&json)?;
        proposals.push(p);
    }
    Ok(proposals)
}

pub fn save_vote(conn: &Connection, vote: &Vote) -> Result<()> {
    let json = serde_json::to_string(vote)?;
    conn.execute(
        "INSERT OR REPLACE INTO votes (proposal_id, voter_cid_short, vote_json)
         VALUES (?1, ?2, ?3)",
        params![&vote.proposal_id, &vote.voter_cid_short, json],
    )?;
    Ok(())
}

pub fn list_votes(conn: &Connection, proposal_id: &str) -> Result<Vec<Vote>> {
    let mut stmt = conn.prepare(
        "SELECT vote_json FROM votes WHERE proposal_id = ?1",
    )?;
    let mut rows = stmt.query(params![proposal_id])?;
    let mut votes = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let v: Vote = serde_json::from_str(&json)?;
        votes.push(v);
    }
    Ok(votes)
}

// ── Vote delegations ──────────────────────────────────────────────────────────

pub fn save_delegation(conn: &Connection, delegation: &VoteDelegation) -> Result<()> {
    let json = serde_json::to_string(delegation)?;
    conn.execute(
        "INSERT OR REPLACE INTO vote_delegations
             (network_cid, delegator_cid_short, proposal_id, delegation_json)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            &delegation.network_cid_short,
            &delegation.delegator_cid_short,
            &delegation.proposal_id,
            json
        ],
    )?;
    Ok(())
}

pub fn delete_delegation(
    conn: &Connection,
    network_cid_short: &str,
    delegator_cid_short: &str,
    proposal_id: Option<&str>,
) -> Result<()> {
    conn.execute(
        "DELETE FROM vote_delegations
          WHERE network_cid = ?1
            AND delegator_cid_short = ?2
            AND COALESCE(proposal_id, '') = COALESCE(?3, '')",
        params![network_cid_short, delegator_cid_short, proposal_id],
    )?;
    Ok(())
}

pub fn list_delegations(conn: &Connection, network_cid_short: &str) -> Result<Vec<VoteDelegation>> {
    let mut stmt = conn.prepare(
        "SELECT delegation_json FROM vote_delegations WHERE network_cid = ?1",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut delegations = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let d: VoteDelegation = serde_json::from_str(&json)?;
        delegations.push(d);
    }
    Ok(delegations)
}

// ── Admin actions (garde-fou majoritaire) ─────────────────────────────────────

pub fn save_admin_action(conn: &Connection, network_cid_short: &str, action: &AdminAction) -> Result<()> {
    let json = serde_json::to_string(action)?;
    conn.execute(
        "INSERT OR REPLACE INTO admin_actions (network_cid, action_id, action_json)
         VALUES (?1, ?2, ?3)",
        params![network_cid_short, &action.id, json],
    )?;
    Ok(())
}

pub fn list_admin_actions(conn: &Connection, network_cid_short: &str) -> Result<Vec<AdminAction>> {
    let mut stmt = conn.prepare(
        "SELECT action_json FROM admin_actions WHERE network_cid = ?1 ORDER BY rowid DESC",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut actions = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let a: AdminAction = serde_json::from_str(&json)?;
        actions.push(a);
    }
    Ok(actions)
}

// ── Directory ─────────────────────────────────────────────────────────────────

pub fn save_directory_entry(conn: &Connection, entry: &DirectoryEntry) -> Result<()> {
    let json = serde_json::to_string(entry)?;
    conn.execute(
        "INSERT OR REPLACE INTO directory_entries (directory_cid, entry_id, entry_json)
         VALUES (?1, ?2, ?3)",
        params![&entry.directory_cid_short, &entry.id, json],
    )?;
    Ok(())
}

pub fn list_directory_entries(conn: &Connection, directory_cid_short: &str) -> Result<Vec<DirectoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT entry_json FROM directory_entries WHERE directory_cid = ?1 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![directory_cid_short])?;
    let mut entries = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let e: DirectoryEntry = serde_json::from_str(&json)?;
        entries.push(e);
    }
    Ok(entries)
}

pub fn search_directory_entries(conn: &Connection, directory_cid_short: &str, query: &str) -> Result<Vec<DirectoryEntry>> {
    let entries = list_directory_entries(conn, directory_cid_short)?;
    Ok(entries.into_iter().filter(|e| e.matches(query)).collect())
}

pub fn delete_directory_entry(conn: &Connection, directory_cid_short: &str, entry_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM directory_entries WHERE directory_cid = ?1 AND entry_id = ?2",
        params![directory_cid_short, entry_id],
    )?;
    Ok(())
}

/// Merge members and messages received via P2P sync.
/// Members already present (by cid_full) are skipped.
/// Messages use INSERT OR IGNORE to avoid duplicates.
pub fn merge_sync_data(
    conn: &Connection,
    network_cid_short: &str,
    members: Vec<MemberRecord>,
    messages: Vec<Message>,
) -> Result<()> {
    let mut network = load_network(conn, network_cid_short)?;
    for member in members {
        if !network.data.members.iter().any(|m| m.cid_full == member.cid_full) {
            network.data.members.push(member);
        }
    }
    save_network(conn, &network)?;
    for msg in &messages {
        let json = serde_json::to_string(msg)?;
        conn.execute(
            "INSERT OR IGNORE INTO messages (network_cid, message_id, message_json, in_outbox)
             VALUES (?1, ?2, ?3, 0)",
            params![network_cid_short, &msg.id, json],
        )?;
    }
    Ok(())
}
