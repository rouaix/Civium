//! Phase 0.5 persistence — SQLite (structured, transactional, query-ready).
//!
//! Upgrade path to full encryption:
//!   1. Add the `rusqlcipher` crate (or `rusqlite` with `sqlcipher` feature).
//!   2. In `open_db`, before running the schema:
//!        conn.execute_batch(&format!("PRAGMA key='{}';", passphrase))?;
//!   The passphrase is provided by the user at login in the Tauri app (weeks 9-10 final).

use anyhow::{Context, Result};
use civium_core::{network::Network, AdminAction, ConnectionRecord, CiviumKeypair, DirectoryEntry, FederatedDirectory, GuardianLink, Mailbox, MemberRecord, Message, MinorRestrictions, Proposal, RrmEntry, TrustedRrm, Vote, VoteDelegation};
use rusqlite::{params, Connection};
use std::path::Path;

// ── Schema ─────────────────────────────────────────────────────────────────────

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
    proposal_id         TEXT NOT NULL DEFAULT '',
    delegation_json     TEXT NOT NULL,
    PRIMARY KEY (network_cid, delegator_cid_short, proposal_id)
);
CREATE TABLE IF NOT EXISTS directory_entries (
    directory_cid   TEXT NOT NULL,
    entry_id        TEXT NOT NULL,
    entry_json      TEXT NOT NULL,
    PRIMARY KEY (directory_cid, entry_id)
);
CREATE TABLE IF NOT EXISTS directory_federations (
    host_cid        TEXT NOT NULL,
    peer_cid        TEXT NOT NULL,
    federation_json TEXT NOT NULL,
    PRIMARY KEY (host_cid, peer_cid)
);
CREATE TABLE IF NOT EXISTS rrm_entries (
    rrm_cid         TEXT NOT NULL,
    entry_id        TEXT NOT NULL,
    entry_json      TEXT NOT NULL,
    PRIMARY KEY (rrm_cid, entry_id)
);
CREATE TABLE IF NOT EXISTS trusted_rrms (
    network_cid     TEXT NOT NULL,
    rrm_cid         TEXT NOT NULL,
    trust_json      TEXT NOT NULL,
    PRIMARY KEY (network_cid, rrm_cid)
);
CREATE TABLE IF NOT EXISTS guardian_links (
    network_cid     TEXT NOT NULL,
    minor_cid       TEXT NOT NULL,
    guardian_cid    TEXT NOT NULL,
    link_json       TEXT NOT NULL,
    PRIMARY KEY (network_cid, minor_cid, guardian_cid)
);
CREATE TABLE IF NOT EXISTS minor_restrictions (
    network_cid         TEXT NOT NULL,
    minor_cid           TEXT NOT NULL,
    restrictions_json   TEXT NOT NULL,
    PRIMARY KEY (network_cid, minor_cid)
);
";

fn open_db(data_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("cannot create data directory {}", data_dir.display()))?;
    let path = data_dir.join("civium.db");
    let conn = Connection::open(&path)
        .with_context(|| format!("cannot open database at {}", path.display()))?;
    conn.execute_batch(SCHEMA).context("cannot initialize database schema")?;
    Ok(conn)
}

// ── Identity ───────────────────────────────────────────────────────────────────

pub fn identity_exists(data_dir: &Path) -> bool {
    let Ok(conn) = open_db(data_dir) else { return false };
    conn.query_row("SELECT COUNT(*) FROM identity", [], |r| r.get::<_, i64>(0))
        .unwrap_or(0)
        > 0
}

pub fn save_identity(data_dir: &Path, keypair: &CiviumKeypair) -> Result<()> {
    let conn = open_db(data_dir)?;
    let cid = keypair.cid();
    conn.execute(
        "INSERT OR REPLACE INTO identity (id, secret_b58, cid_short, cid_full)
         VALUES (1, ?1, ?2, ?3)",
        params![keypair.secret_b58(), cid.short(), cid.full()],
    )?;
    Ok(())
}

pub fn load_identity(data_dir: &Path) -> Result<CiviumKeypair> {
    let conn = open_db(data_dir)?;
    let secret: String = conn
        .query_row("SELECT secret_b58 FROM identity WHERE id = 1", [], |r| r.get(0))
        .context("no identity found — run `identity init` first")?;
    CiviumKeypair::from_secret_b58(&secret).map_err(|e| anyhow::anyhow!("{e}"))
}

// ── Network ────────────────────────────────────────────────────────────────────

pub fn save_network(data_dir: &Path, network: &Network) -> Result<()> {
    let conn = open_db(data_dir)?;
    let json = serde_json::to_string(&network.data)?;
    conn.execute(
        "INSERT OR REPLACE INTO networks (cid_short, data_json) VALUES (?1, ?2)",
        params![network.cid_short(), json],
    )?;
    Ok(())
}

pub fn load_network(data_dir: &Path, cid_short: &str) -> Result<Network> {
    let conn = open_db(data_dir)?;
    let json: String = conn
        .query_row(
            "SELECT data_json FROM networks WHERE cid_short = ?1",
            params![cid_short],
            |r| r.get(0),
        )
        .with_context(|| format!("no network found for '{cid_short}'"))?;
    let data = serde_json::from_str(&json)?;
    Network::from_data(data).map_err(|e| anyhow::anyhow!("{e}"))
}

pub fn list_network_cids(data_dir: &Path) -> Vec<String> {
    let Ok(conn) = open_db(data_dir) else { return vec![] };
    let Ok(mut stmt) = conn.prepare("SELECT cid_short FROM networks") else { return vec![] };
    let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) else { return vec![] };
    rows.flatten().collect()
}

// ── Connections ────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct ConnectionStore {
    pub connections: Vec<ConnectionRecord>,
}

pub fn save_connections(
    data_dir: &Path,
    network_cid_short: &str,
    store: &ConnectionStore,
) -> Result<()> {
    let mut conn = open_db(data_dir)?;
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM connections WHERE network_cid = ?1",
        params![network_cid_short],
    )?;
    for rec in &store.connections {
        let json = serde_json::to_string(rec)?;
        tx.execute(
            "INSERT INTO connections (network_cid, peer_cid_full, record_json)
             VALUES (?1, ?2, ?3)",
            params![network_cid_short, &rec.peer_cid_full, json],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn load_connections(data_dir: &Path, network_cid_short: &str) -> Result<ConnectionStore> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT record_json FROM connections WHERE network_cid = ?1",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut connections = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let rec: ConnectionRecord =
            serde_json::from_str(&json).context("invalid connection record in database")?;
        connections.push(rec);
    }
    Ok(ConnectionStore { connections })
}

// ── Mailbox ────────────────────────────────────────────────────────────────────

pub fn save_mailbox(data_dir: &Path, network_cid_short: &str, mailbox: &Mailbox) -> Result<()> {
    let mut conn = open_db(data_dir)?;
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM messages WHERE network_cid = ?1",
        params![network_cid_short],
    )?;
    for msg in &mailbox.messages {
        let json = serde_json::to_string(msg)?;
        tx.execute(
            "INSERT INTO messages (network_cid, message_id, message_json, in_outbox)
             VALUES (?1, ?2, ?3, 0)",
            params![network_cid_short, &msg.id, json],
        )?;
    }
    for msg in &mailbox.outbox {
        let json = serde_json::to_string(msg)?;
        tx.execute(
            "INSERT INTO messages (network_cid, message_id, message_json, in_outbox)
             VALUES (?1, ?2, ?3, 1)",
            params![network_cid_short, &msg.id, json],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn load_mailbox(data_dir: &Path, network_cid_short: &str) -> Result<Mailbox> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT message_json, in_outbox FROM messages
         WHERE network_cid = ?1 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut messages: Vec<Message> = Vec::new();
    let mut outbox: Vec<Message> = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let in_outbox: i64 = row.get(1)?;
        let msg: Message =
            serde_json::from_str(&json).context("invalid message in database")?;
        if in_outbox == 1 {
            outbox.push(msg);
        } else {
            messages.push(msg);
        }
    }
    Ok(Mailbox { messages, outbox })
}

// ── Governance ────────────────────────────────────────────────────────────────

pub fn save_proposal(data_dir: &Path, network_cid_short: &str, proposal: &Proposal) -> Result<()> {
    let conn = open_db(data_dir)?;
    let json = serde_json::to_string(proposal)?;
    conn.execute(
        "INSERT OR REPLACE INTO proposals (network_cid, proposal_id, proposal_json)
         VALUES (?1, ?2, ?3)",
        params![network_cid_short, &proposal.id, json],
    )?;
    Ok(())
}

pub fn list_proposals(data_dir: &Path, network_cid_short: &str) -> Result<Vec<Proposal>> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT proposal_json FROM proposals WHERE network_cid = ?1 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut proposals = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let p: Proposal = serde_json::from_str(&json).context("invalid proposal in database")?;
        proposals.push(p);
    }
    Ok(proposals)
}

pub fn save_vote(data_dir: &Path, vote: &Vote) -> Result<()> {
    let conn = open_db(data_dir)?;
    let json = serde_json::to_string(vote)?;
    conn.execute(
        "INSERT OR REPLACE INTO votes (proposal_id, voter_cid_short, vote_json)
         VALUES (?1, ?2, ?3)",
        params![&vote.proposal_id, &vote.voter_cid_short, json],
    )?;
    Ok(())
}

pub fn list_votes(data_dir: &Path, proposal_id: &str) -> Result<Vec<Vote>> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT vote_json FROM votes WHERE proposal_id = ?1",
    )?;
    let mut rows = stmt.query(params![proposal_id])?;
    let mut votes = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let v: Vote = serde_json::from_str(&json).context("invalid vote in database")?;
        votes.push(v);
    }
    Ok(votes)
}

// ── Vote delegations ──────────────────────────────────────────────────────────

pub fn save_delegation(data_dir: &Path, delegation: &VoteDelegation) -> Result<()> {
    let conn = open_db(data_dir)?;
    let json = serde_json::to_string(delegation)?;
    let pid = delegation.proposal_id.as_deref().unwrap_or("");
    conn.execute(
        "INSERT OR REPLACE INTO vote_delegations
             (network_cid, delegator_cid_short, proposal_id, delegation_json)
         VALUES (?1, ?2, ?3, ?4)",
        params![&delegation.network_cid_short, &delegation.delegator_cid_short, pid, json],
    )?;
    Ok(())
}

pub fn delete_delegation(
    data_dir: &Path,
    network_cid_short: &str,
    delegator_cid_short: &str,
    proposal_id: Option<&str>,
) -> Result<()> {
    let conn = open_db(data_dir)?;
    let pid = proposal_id.unwrap_or("");
    conn.execute(
        "DELETE FROM vote_delegations
          WHERE network_cid = ?1
            AND delegator_cid_short = ?2
            AND proposal_id = ?3",
        params![network_cid_short, delegator_cid_short, pid],
    )?;
    Ok(())
}

pub fn list_delegations(data_dir: &Path, network_cid_short: &str) -> Result<Vec<VoteDelegation>> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT delegation_json FROM vote_delegations WHERE network_cid = ?1",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut delegations = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let d: VoteDelegation = serde_json::from_str(&json).context("invalid delegation in database")?;
        delegations.push(d);
    }
    Ok(delegations)
}

// ── Admin actions ─────────────────────────────────────────────────────────────

pub fn save_admin_action(data_dir: &Path, network_cid_short: &str, action: &AdminAction) -> Result<()> {
    let conn = open_db(data_dir)?;
    let json = serde_json::to_string(action)?;
    conn.execute(
        "INSERT OR REPLACE INTO admin_actions (network_cid, action_id, action_json)
         VALUES (?1, ?2, ?3)",
        params![network_cid_short, &action.id, json],
    )?;
    Ok(())
}

pub fn list_admin_actions(data_dir: &Path, network_cid_short: &str) -> Result<Vec<AdminAction>> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT action_json FROM admin_actions WHERE network_cid = ?1 ORDER BY rowid DESC",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut actions = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let a: AdminAction = serde_json::from_str(&json).context("invalid admin action in database")?;
        actions.push(a);
    }
    Ok(actions)
}

// ── Directory ─────────────────────────────────────────────────────────────────

pub fn save_directory_entry(data_dir: &Path, entry: &DirectoryEntry) -> Result<()> {
    let conn = open_db(data_dir)?;
    let json = serde_json::to_string(entry)?;
    conn.execute(
        "INSERT OR REPLACE INTO directory_entries (directory_cid, entry_id, entry_json)
         VALUES (?1, ?2, ?3)",
        params![&entry.directory_cid_short, &entry.id, json],
    )?;
    Ok(())
}

pub fn list_directory_entries(data_dir: &Path, directory_cid_short: &str) -> Result<Vec<DirectoryEntry>> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT entry_json FROM directory_entries WHERE directory_cid = ?1 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![directory_cid_short])?;
    let mut entries = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let e: DirectoryEntry = serde_json::from_str(&json).context("invalid directory entry in database")?;
        entries.push(e);
    }
    Ok(entries)
}

pub fn search_directory_entries(data_dir: &Path, directory_cid_short: &str, query: &str) -> Result<Vec<DirectoryEntry>> {
    let entries = list_directory_entries(data_dir, directory_cid_short)?;
    Ok(entries.into_iter().filter(|e| e.matches(query)).collect())
}

pub fn delete_directory_entry(data_dir: &Path, directory_cid_short: &str, entry_id: &str) -> Result<()> {
    let conn = open_db(data_dir)?;
    conn.execute(
        "DELETE FROM directory_entries WHERE directory_cid = ?1 AND entry_id = ?2",
        params![directory_cid_short, entry_id],
    )?;
    Ok(())
}

// ── Directory federations ─────────────────────────────────────────────────────

pub fn save_federation(data_dir: &Path, fed: &FederatedDirectory) -> Result<()> {
    let conn = open_db(data_dir)?;
    let json = serde_json::to_string(fed)?;
    conn.execute(
        "INSERT OR REPLACE INTO directory_federations (host_cid, peer_cid, federation_json)
         VALUES (?1, ?2, ?3)",
        params![&fed.host_cid_short, &fed.peer_cid_short, json],
    )?;
    Ok(())
}

pub fn list_federations(data_dir: &Path, host_cid_short: &str) -> Result<Vec<FederatedDirectory>> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT federation_json FROM directory_federations WHERE host_cid = ?1 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![host_cid_short])?;
    let mut feds = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let f: FederatedDirectory = serde_json::from_str(&json).context("invalid federation in database")?;
        feds.push(f);
    }
    Ok(feds)
}

pub fn delete_federation(data_dir: &Path, host_cid_short: &str, peer_cid_short: &str) -> Result<()> {
    let conn = open_db(data_dir)?;
    conn.execute(
        "DELETE FROM directory_federations WHERE host_cid = ?1 AND peer_cid = ?2",
        params![host_cid_short, peer_cid_short],
    )?;
    Ok(())
}

// ── RRM entries ───────────────────────────────────────────────────────────────

pub fn save_rrm_entry(data_dir: &Path, entry: &RrmEntry) -> Result<()> {
    let conn = open_db(data_dir)?;
    let json = serde_json::to_string(entry)?;
    conn.execute(
        "INSERT OR REPLACE INTO rrm_entries (rrm_cid, entry_id, entry_json)
         VALUES (?1, ?2, ?3)",
        params![&entry.rrm_cid_short, &entry.id, json],
    )?;
    Ok(())
}

pub fn list_rrm_entries(data_dir: &Path, rrm_cid_short: &str) -> Result<Vec<RrmEntry>> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT entry_json FROM rrm_entries WHERE rrm_cid = ?1 ORDER BY rowid DESC",
    )?;
    let mut rows = stmt.query(params![rrm_cid_short])?;
    let mut entries = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let e: RrmEntry = serde_json::from_str(&json).context("invalid RRM entry in database")?;
        entries.push(e);
    }
    Ok(entries)
}

pub fn search_rrm_entries(data_dir: &Path, rrm_cid_short: &str, query: &str) -> Result<Vec<RrmEntry>> {
    let entries = list_rrm_entries(data_dir, rrm_cid_short)?;
    Ok(entries.into_iter().filter(|e| e.matches(query)).collect())
}

pub fn delete_rrm_entry(data_dir: &Path, rrm_cid_short: &str, entry_id: &str) -> Result<()> {
    let conn = open_db(data_dir)?;
    conn.execute(
        "DELETE FROM rrm_entries WHERE rrm_cid = ?1 AND entry_id = ?2",
        params![rrm_cid_short, entry_id],
    )?;
    Ok(())
}

// ── Trusted RRMs ──────────────────────────────────────────────────────────────

pub fn save_trusted_rrm(data_dir: &Path, trust: &TrustedRrm) -> Result<()> {
    let conn = open_db(data_dir)?;
    let json = serde_json::to_string(trust)?;
    conn.execute(
        "INSERT OR REPLACE INTO trusted_rrms (network_cid, rrm_cid, trust_json)
         VALUES (?1, ?2, ?3)",
        params![&trust.network_cid_short, &trust.rrm_cid_short, json],
    )?;
    Ok(())
}

pub fn list_trusted_rrms(data_dir: &Path, network_cid_short: &str) -> Result<Vec<TrustedRrm>> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT trust_json FROM trusted_rrms WHERE network_cid = ?1 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut trusts = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let t: TrustedRrm = serde_json::from_str(&json).context("invalid trusted RRM in database")?;
        trusts.push(t);
    }
    Ok(trusts)
}

pub fn delete_trusted_rrm(data_dir: &Path, network_cid_short: &str, rrm_cid_short: &str) -> Result<()> {
    let conn = open_db(data_dir)?;
    conn.execute(
        "DELETE FROM trusted_rrms WHERE network_cid = ?1 AND rrm_cid = ?2",
        params![network_cid_short, rrm_cid_short],
    )?;
    Ok(())
}

/// Returns the (TrustedRrm, RrmEntry) pairs where `peer_cid_short` is listed in any
/// RRM trusted by `network_cid_short`. Used for connection-time warnings.
pub fn check_rrm_warnings(
    data_dir: &Path,
    network_cid_short: &str,
    peer_cid_short: &str,
) -> Result<Vec<(TrustedRrm, RrmEntry)>> {
    let trusts = list_trusted_rrms(data_dir, network_cid_short)?;
    let mut warnings = Vec::new();
    for trust in trusts {
        let entries = list_rrm_entries(data_dir, &trust.rrm_cid_short)?;
        for entry in entries {
            if entry.network_cid_short == peer_cid_short {
                warnings.push((trust.clone(), entry));
                break;
            }
        }
    }
    Ok(warnings)
}

// ── Minor / Guardian ─────────────────────────────────────────────────────────

pub fn set_member_minor(data_dir: &Path, network_cid_short: &str, member_cid_short: &str, is_minor: bool) -> Result<()> {
    let mut network = load_network(data_dir, network_cid_short)?;
    let member = network.data.members.iter_mut()
        .find(|m| m.cid_short == member_cid_short)
        .ok_or_else(|| anyhow::anyhow!("member '{}' not found", member_cid_short))?;
    member.is_minor = is_minor;
    save_network(data_dir, &network)
}

pub fn save_guardian_link(data_dir: &Path, link: &GuardianLink) -> Result<()> {
    let conn = open_db(data_dir)?;
    let json = serde_json::to_string(link)?;
    conn.execute(
        "INSERT OR REPLACE INTO guardian_links (network_cid, minor_cid, guardian_cid, link_json)
         VALUES (?1, ?2, ?3, ?4)",
        params![&link.network_cid_short, &link.minor_cid_short, &link.guardian_cid_short, json],
    )?;
    Ok(())
}

pub fn list_guardians(data_dir: &Path, network_cid_short: &str, minor_cid_short: &str) -> Result<Vec<GuardianLink>> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT link_json FROM guardian_links WHERE network_cid = ?1 AND minor_cid = ?2 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![network_cid_short, minor_cid_short])?;
    let mut links = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let l: GuardianLink = serde_json::from_str(&json).context("invalid guardian link")?;
        links.push(l);
    }
    Ok(links)
}

pub fn list_wards(data_dir: &Path, network_cid_short: &str, guardian_cid_short: &str) -> Result<Vec<GuardianLink>> {
    let conn = open_db(data_dir)?;
    let mut stmt = conn.prepare(
        "SELECT link_json FROM guardian_links WHERE network_cid = ?1 AND guardian_cid = ?2 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![network_cid_short, guardian_cid_short])?;
    let mut links = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let l: GuardianLink = serde_json::from_str(&json).context("invalid guardian link")?;
        links.push(l);
    }
    Ok(links)
}

pub fn delete_guardian_link(data_dir: &Path, network_cid_short: &str, minor_cid_short: &str, guardian_cid_short: &str) -> Result<()> {
    let conn = open_db(data_dir)?;
    conn.execute(
        "DELETE FROM guardian_links WHERE network_cid = ?1 AND minor_cid = ?2 AND guardian_cid = ?3",
        params![network_cid_short, minor_cid_short, guardian_cid_short],
    )?;
    Ok(())
}

pub fn save_minor_restrictions(data_dir: &Path, r: &MinorRestrictions) -> Result<()> {
    let conn = open_db(data_dir)?;
    let json = serde_json::to_string(r)?;
    conn.execute(
        "INSERT OR REPLACE INTO minor_restrictions (network_cid, minor_cid, restrictions_json)
         VALUES (?1, ?2, ?3)",
        params![&r.network_cid_short, &r.minor_cid_short, json],
    )?;
    Ok(())
}

pub fn get_minor_restrictions(data_dir: &Path, network_cid_short: &str, minor_cid_short: &str) -> Result<Option<MinorRestrictions>> {
    let conn = open_db(data_dir)?;
    let result = conn.query_row(
        "SELECT restrictions_json FROM minor_restrictions WHERE network_cid = ?1 AND minor_cid = ?2",
        params![network_cid_short, minor_cid_short],
        |r| r.get::<_, String>(0),
    );
    match result {
        Ok(json) => Ok(Some(serde_json::from_str(&json).context("invalid minor restrictions")?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_minor_restrictions(data_dir: &Path, network_cid_short: &str, minor_cid_short: &str) -> Result<()> {
    let conn = open_db(data_dir)?;
    conn.execute(
        "DELETE FROM minor_restrictions WHERE network_cid = ?1 AND minor_cid = ?2",
        params![network_cid_short, minor_cid_short],
    )?;
    Ok(())
}

// ── Sync ──────────────────────────────────────────────────────────────────────

/// Merge members and messages received via P2P sync into the local store.
/// Members already present (by cid_full) are skipped. Messages use INSERT OR IGNORE.
pub fn merge_sync_data(
    data_dir: &Path,
    network_cid_short: &str,
    members: &[MemberRecord],
    messages: &[Message],
) -> Result<()> {
    let mut network = load_network(data_dir, network_cid_short)?;
    for member in members {
        if !network.data.members.iter().any(|m| m.cid_full == member.cid_full) {
            network.data.members.push(member.clone());
        }
    }
    save_network(data_dir, &network)?;
    let conn = open_db(data_dir)?;
    for msg in messages {
        let json = serde_json::to_string(msg)?;
        conn.execute(
            "INSERT OR IGNORE INTO messages (network_cid, message_id, message_json, in_outbox)
             VALUES (?1, ?2, ?3, 0)",
            params![network_cid_short, &msg.id, json],
        )?;
    }
    Ok(())
}
