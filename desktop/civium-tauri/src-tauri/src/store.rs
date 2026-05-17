//! SQLite store for the Tauri app — same schema as civium-cli.
//! Both apps share the same civium.db file under the data directory.

use anyhow::{Context, Result};
use civium_core::{network::Network, ActivityEvent, ActivityKind, AdminAction, AgendaEvent, CiviumKeypair, DirectoryEntry, Document, FederatedDirectory, GuardianLink, MemberRecord, Message, MinorRestrictions, Notification, PairedDevice, PluginManifest, PluginRecord, PluginState, Proposal, RrmEntry, TrustedRrm, Vote, VoteDelegation, preinstalled_plugins};
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
CREATE TABLE IF NOT EXISTS plugins (
    plugin_id   TEXT PRIMARY KEY,
    record_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS agenda_events (
    network_cid TEXT NOT NULL,
    event_id    TEXT NOT NULL,
    event_json  TEXT NOT NULL,
    PRIMARY KEY (network_cid, event_id)
);
CREATE TABLE IF NOT EXISTS activity_feed (
    network_cid TEXT NOT NULL,
    event_id    TEXT NOT NULL,
    event_json  TEXT NOT NULL,
    PRIMARY KEY (network_cid, event_id)
);
CREATE TABLE IF NOT EXISTS notifications (
    notif_id    TEXT PRIMARY KEY,
    notif_json  TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS documents (
    network_cid TEXT NOT NULL,
    doc_id      TEXT NOT NULL,
    doc_json    TEXT NOT NULL,
    PRIMARY KEY (network_cid, doc_id)
);
CREATE TABLE IF NOT EXISTS paired_devices (
    device_id   TEXT PRIMARY KEY,
    device_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS outbox_queue (
    network_cid TEXT    NOT NULL,
    message_id  TEXT    NOT NULL,
    queued_at   INTEGER NOT NULL,
    PRIMARY KEY (network_cid, message_id)
);
CREATE TABLE IF NOT EXISTS ap_followers (
    network_cid     TEXT    NOT NULL,
    actor_url       TEXT    NOT NULL,
    inbox_url       TEXT    NOT NULL,
    shared_inbox    TEXT,
    followed_at     INTEGER NOT NULL,
    PRIMARY KEY (network_cid, actor_url)
);
CREATE TABLE IF NOT EXISTS ap_posts (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    network_cid     TEXT    NOT NULL,
    note_id         TEXT    NOT NULL UNIQUE,
    content         TEXT    NOT NULL,
    ap_activity_id  TEXT,
    posted_at       INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS rcc_registrations (
    network_cid_short TEXT    NOT NULL,
    network_cid_full  TEXT    NOT NULL,
    network_name      TEXT    NOT NULL DEFAULT '',
    admin_email       TEXT    NOT NULL,
    status            TEXT    NOT NULL DEFAULT 'pending',
    attempts          INTEGER NOT NULL DEFAULT 0,
    last_attempt      INTEGER,
    registered_at     INTEGER NOT NULL,
    PRIMARY KEY (network_cid_short)
);
CREATE TABLE IF NOT EXISTS node_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS hub_config (
    network_cid_short TEXT PRIMARY KEY,
    hub_url           TEXT NOT NULL,
    enabled           INTEGER NOT NULL DEFAULT 1,
    last_sync_ts      INTEGER NOT NULL DEFAULT 0
);
";

pub fn open_db(data_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(data_dir)?;
    let conn = Connection::open(data_dir.join("civium.db"))?;
    conn.execute_batch(SCHEMA)?;
    seed_plugins(&conn)?;
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

pub fn delete_network(conn: &Connection, cid_short: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM networks WHERE cid_short = ?1",
        params![cid_short],
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
    conn: &Connection,
    network_cid_short: &str,
    delegator_cid_short: &str,
    proposal_id: Option<&str>,
) -> Result<()> {
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

// ── Directory federations ─────────────────────────────────────────────────────

pub fn save_federation(conn: &Connection, fed: &FederatedDirectory) -> Result<()> {
    let json = serde_json::to_string(fed)?;
    conn.execute(
        "INSERT OR REPLACE INTO directory_federations (host_cid, peer_cid, federation_json)
         VALUES (?1, ?2, ?3)",
        params![&fed.host_cid_short, &fed.peer_cid_short, json],
    )?;
    Ok(())
}

pub fn list_federations(conn: &Connection, host_cid_short: &str) -> Result<Vec<FederatedDirectory>> {
    let mut stmt = conn.prepare(
        "SELECT federation_json FROM directory_federations WHERE host_cid = ?1 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![host_cid_short])?;
    let mut feds = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let f: FederatedDirectory = serde_json::from_str(&json)?;
        feds.push(f);
    }
    Ok(feds)
}

pub fn delete_federation(conn: &Connection, host_cid_short: &str, peer_cid_short: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM directory_federations WHERE host_cid = ?1 AND peer_cid = ?2",
        params![host_cid_short, peer_cid_short],
    )?;
    Ok(())
}

// ── RRM entries ───────────────────────────────────────────────────────────────

pub fn save_rrm_entry(conn: &Connection, entry: &RrmEntry) -> Result<()> {
    let json = serde_json::to_string(entry)?;
    conn.execute(
        "INSERT OR REPLACE INTO rrm_entries (rrm_cid, entry_id, entry_json)
         VALUES (?1, ?2, ?3)",
        params![&entry.rrm_cid_short, &entry.id, json],
    )?;
    Ok(())
}

pub fn list_rrm_entries(conn: &Connection, rrm_cid_short: &str) -> Result<Vec<RrmEntry>> {
    let mut stmt = conn.prepare(
        "SELECT entry_json FROM rrm_entries WHERE rrm_cid = ?1 ORDER BY rowid DESC",
    )?;
    let mut rows = stmt.query(params![rrm_cid_short])?;
    let mut entries = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let e: RrmEntry = serde_json::from_str(&json)?;
        entries.push(e);
    }
    Ok(entries)
}

pub fn delete_rrm_entry(conn: &Connection, rrm_cid_short: &str, entry_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM rrm_entries WHERE rrm_cid = ?1 AND entry_id = ?2",
        params![rrm_cid_short, entry_id],
    )?;
    Ok(())
}

// ── Trusted RRMs ──────────────────────────────────────────────────────────────

pub fn save_trusted_rrm(conn: &Connection, trust: &TrustedRrm) -> Result<()> {
    let json = serde_json::to_string(trust)?;
    conn.execute(
        "INSERT OR REPLACE INTO trusted_rrms (network_cid, rrm_cid, trust_json)
         VALUES (?1, ?2, ?3)",
        params![&trust.network_cid_short, &trust.rrm_cid_short, json],
    )?;
    Ok(())
}

pub fn list_trusted_rrms(conn: &Connection, network_cid_short: &str) -> Result<Vec<TrustedRrm>> {
    let mut stmt = conn.prepare(
        "SELECT trust_json FROM trusted_rrms WHERE network_cid = ?1 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut trusts = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let t: TrustedRrm = serde_json::from_str(&json)?;
        trusts.push(t);
    }
    Ok(trusts)
}

pub fn delete_trusted_rrm(conn: &Connection, network_cid_short: &str, rrm_cid_short: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM trusted_rrms WHERE network_cid = ?1 AND rrm_cid = ?2",
        params![network_cid_short, rrm_cid_short],
    )?;
    Ok(())
}

/// Returns (TrustedRrm, RrmEntry) pairs where `peer_cid_short` is listed in any
/// RRM trusted by `network_cid_short`. Used for connection-time warnings.
pub fn check_rrm_warnings(
    conn: &Connection,
    network_cid_short: &str,
    peer_cid_short: &str,
) -> Result<Vec<(TrustedRrm, RrmEntry)>> {
    let trusts = list_trusted_rrms(conn, network_cid_short)?;
    let mut warnings = Vec::new();
    for trust in trusts {
        let entries = list_rrm_entries(conn, &trust.rrm_cid_short)?;
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

pub fn set_member_minor(conn: &Connection, network_cid_short: &str, member_cid_short: &str, is_minor: bool) -> Result<()> {
    let mut network = load_network(conn, network_cid_short)?;
    let member = network.data.members.iter_mut()
        .find(|m| m.cid_short == member_cid_short)
        .ok_or_else(|| anyhow::anyhow!("member '{}' not found", member_cid_short))?;
    member.is_minor = is_minor;
    save_network(conn, &network)
}

pub fn save_guardian_link(conn: &Connection, link: &GuardianLink) -> Result<()> {
    let json = serde_json::to_string(link)?;
    conn.execute(
        "INSERT OR REPLACE INTO guardian_links (network_cid, minor_cid, guardian_cid, link_json)
         VALUES (?1, ?2, ?3, ?4)",
        params![&link.network_cid_short, &link.minor_cid_short, &link.guardian_cid_short, json],
    )?;
    Ok(())
}

pub fn list_guardians(conn: &Connection, network_cid_short: &str, minor_cid_short: &str) -> Result<Vec<GuardianLink>> {
    let mut stmt = conn.prepare(
        "SELECT link_json FROM guardian_links WHERE network_cid = ?1 AND minor_cid = ?2 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![network_cid_short, minor_cid_short])?;
    let mut links = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let l: GuardianLink = serde_json::from_str(&json)?;
        links.push(l);
    }
    Ok(links)
}

pub fn list_wards(conn: &Connection, network_cid_short: &str, guardian_cid_short: &str) -> Result<Vec<GuardianLink>> {
    let mut stmt = conn.prepare(
        "SELECT link_json FROM guardian_links WHERE network_cid = ?1 AND guardian_cid = ?2 ORDER BY rowid",
    )?;
    let mut rows = stmt.query(params![network_cid_short, guardian_cid_short])?;
    let mut links = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let l: GuardianLink = serde_json::from_str(&json)?;
        links.push(l);
    }
    Ok(links)
}

pub fn delete_guardian_link(conn: &Connection, network_cid_short: &str, minor_cid_short: &str, guardian_cid_short: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM guardian_links WHERE network_cid = ?1 AND minor_cid = ?2 AND guardian_cid = ?3",
        params![network_cid_short, minor_cid_short, guardian_cid_short],
    )?;
    Ok(())
}

pub fn save_minor_restrictions(conn: &Connection, r: &MinorRestrictions) -> Result<()> {
    let json = serde_json::to_string(r)?;
    conn.execute(
        "INSERT OR REPLACE INTO minor_restrictions (network_cid, minor_cid, restrictions_json)
         VALUES (?1, ?2, ?3)",
        params![&r.network_cid_short, &r.minor_cid_short, json],
    )?;
    Ok(())
}

pub fn get_minor_restrictions(conn: &Connection, network_cid_short: &str, minor_cid_short: &str) -> Result<Option<MinorRestrictions>> {
    let result = conn.query_row(
        "SELECT restrictions_json FROM minor_restrictions WHERE network_cid = ?1 AND minor_cid = ?2",
        params![network_cid_short, minor_cid_short],
        |r| r.get::<_, String>(0),
    );
    match result {
        Ok(json) => Ok(Some(serde_json::from_str(&json)?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_minor_restrictions(conn: &Connection, network_cid_short: &str, minor_cid_short: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM minor_restrictions WHERE network_cid = ?1 AND minor_cid = ?2",
        params![network_cid_short, minor_cid_short],
    )?;
    Ok(())
}

/// Check whether `peer_cid_short` is allowed to send a direct message to/from `minor_cid_short`.
/// Returns `Ok(())` if allowed, `Err(...)` with a human-readable reason if blocked.
/// The check is symmetrical — call this for both sides of a direct message (sender and recipient).
pub fn check_minor_interaction(
    conn: &Connection,
    network_cid_short: &str,
    minor_cid_short: &str,
    peer_cid_short: &str,
) -> Result<()> {
    let network = load_network(conn, network_cid_short)?;
    let minor = match network.data.members.iter().find(|m| m.cid_short == minor_cid_short) {
        Some(m) if m.is_minor => m,
        _ => return Ok(()),
    };
    let _ = minor;

    // Guardians bypass all restrictions
    let guardians = list_guardians(conn, network_cid_short, minor_cid_short)?;
    if guardians.iter().any(|g| g.guardian_cid_short == peer_cid_short) {
        return Ok(());
    }

    // Peer's circle in this network (unknown peer → circle 0)
    let peer_circle = network.data.members.iter()
        .find(|m| m.cid_short == peer_cid_short)
        .map(|m| m.circle as u8)
        .unwrap_or(0);

    let allowed = match get_minor_restrictions(conn, network_cid_short, minor_cid_short)? {
        Some(r) => r.allows(peer_cid_short, peer_circle),
        None    => peer_circle <= 1, // default: Annuaire + Connaissance only
    };

    if allowed {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "interaction refusée : '{}' est un compte mineur — seuls les tuteurs et membres au cercle ≤ max_circle peuvent interagir directement",
            minor_cid_short
        ))
    }
}

// ── Plugins ───────────────────────────────────────────────────────────────────

/// Seed pre-installed Civium plugins if they don't exist yet.
fn seed_plugins(conn: &Connection) -> Result<()> {
    for (manifest, enabled) in preinstalled_plugins() {
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM plugins WHERE plugin_id = ?1",
            params![&manifest.id],
            |r| r.get(0),
        ).unwrap_or(0);
        if exists == 0 {
            let record = if enabled {
                PluginRecord::new_enabled(manifest)
            } else {
                PluginRecord::new(manifest)
            };
            let json = serde_json::to_string(&record)?;
            conn.execute(
                "INSERT INTO plugins (plugin_id, record_json) VALUES (?1, ?2)",
                params![&record.manifest.id, json],
            )?;
        }
    }
    Ok(())
}

pub fn install_plugin(conn: &Connection, manifest: PluginManifest) -> Result<PluginRecord> {
    let record = PluginRecord::new(manifest);
    let json = serde_json::to_string(&record)?;
    conn.execute(
        "INSERT OR REPLACE INTO plugins (plugin_id, record_json) VALUES (?1, ?2)",
        params![&record.manifest.id, json],
    )?;
    Ok(record)
}

pub fn list_plugins(conn: &Connection) -> Result<Vec<PluginRecord>> {
    let mut stmt = conn.prepare("SELECT record_json FROM plugins ORDER BY plugin_id")?;
    let mut rows = stmt.query([])?;
    let mut records = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let r: PluginRecord = serde_json::from_str(&json)?;
        records.push(r);
    }
    Ok(records)
}

pub fn get_plugin(conn: &Connection, plugin_id: &str) -> Result<Option<PluginRecord>> {
    let result = conn.query_row(
        "SELECT record_json FROM plugins WHERE plugin_id = ?1",
        params![plugin_id],
        |r| r.get::<_, String>(0),
    );
    match result {
        Ok(json) => Ok(Some(serde_json::from_str(&json)?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set_plugin_state(conn: &Connection, plugin_id: &str, state: PluginState) -> Result<()> {
    let mut record = get_plugin(conn, plugin_id)?
        .ok_or_else(|| anyhow::anyhow!("plugin '{}' not found", plugin_id))?;
    if record.manifest.is_system {
        return Err(anyhow::anyhow!("les plugins système ne peuvent pas être désactivés"));
    }
    record.state = state;
    let json = serde_json::to_string(&record)?;
    conn.execute(
        "UPDATE plugins SET record_json = ?1 WHERE plugin_id = ?2",
        params![json, plugin_id],
    )?;
    Ok(())
}

// ── Agenda ───────────────────────────────────────────────────────────────────

pub fn save_agenda_event(conn: &Connection, event: &AgendaEvent) -> Result<()> {
    let json = serde_json::to_string(event)?;
    conn.execute(
        "INSERT OR REPLACE INTO agenda_events (network_cid, event_id, event_json) VALUES (?1, ?2, ?3)",
        params![&event.network_cid_short, &event.id, json],
    )?;
    Ok(())
}

pub fn list_agenda_events(conn: &Connection, network_cid_short: &str) -> Result<Vec<AgendaEvent>> {
    let mut stmt = conn.prepare(
        "SELECT event_json FROM agenda_events WHERE network_cid = ?1 ORDER BY rowid ASC",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut events = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        events.push(serde_json::from_str(&json).context("invalid agenda event")?);
    }
    Ok(events)
}

pub fn get_agenda_event(conn: &Connection, network_cid_short: &str, event_id: &str) -> Result<Option<AgendaEvent>> {
    let result = conn.query_row(
        "SELECT event_json FROM agenda_events WHERE network_cid = ?1 AND event_id = ?2",
        params![network_cid_short, event_id],
        |r| r.get::<_, String>(0),
    );
    match result {
        Ok(json) => Ok(Some(serde_json::from_str(&json).context("invalid agenda event")?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_agenda_event(conn: &Connection, network_cid_short: &str, event_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM agenda_events WHERE network_cid = ?1 AND event_id = ?2",
        params![network_cid_short, event_id],
    )?;
    Ok(())
}

// ── Documents ────────────────────────────────────────────────────────────────

pub fn save_document(conn: &Connection, doc: &Document) -> Result<()> {
    let json = serde_json::to_string(doc).context("serialize document")?;
    conn.execute(
        "INSERT OR REPLACE INTO documents (network_cid, doc_id, doc_json) VALUES (?1, ?2, ?3)",
        params![doc.network_cid_short, doc.id, json],
    )?;
    Ok(())
}

pub fn list_documents(conn: &Connection, network_cid_short: &str) -> Result<Vec<Document>> {
    let mut stmt = conn.prepare(
        "SELECT doc_json FROM documents WHERE network_cid = ?1 ORDER BY rowid ASC",
    )?;
    let docs = stmt
        .query_map(params![network_cid_short], |row| row.get::<_, String>(0))?
        .map(|r| r.map_err(anyhow::Error::from))
        .map(|r| r.and_then(|json| serde_json::from_str(&json).context("invalid document")))
        .collect::<Result<Vec<Document>>>()?;
    Ok(docs)
}

pub fn get_document(conn: &Connection, network_cid_short: &str, doc_id: &str) -> Result<Option<Document>> {
    let result = conn.query_row(
        "SELECT doc_json FROM documents WHERE network_cid = ?1 AND doc_id = ?2",
        params![network_cid_short, doc_id],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(json) => Ok(Some(serde_json::from_str(&json).context("invalid document")?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_document(conn: &Connection, network_cid_short: &str, doc_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM documents WHERE network_cid = ?1 AND doc_id = ?2",
        params![network_cid_short, doc_id],
    )?;
    Ok(())
}

// ── Paired devices ───────────────────────────────────────────────────────────

pub fn save_paired_device(conn: &Connection, device: &PairedDevice) -> Result<()> {
    let json = serde_json::to_string(device).context("serialize paired device")?;
    conn.execute(
        "INSERT OR REPLACE INTO paired_devices (device_id, device_json) VALUES (?1, ?2)",
        params![device.id, json],
    )?;
    Ok(())
}

pub fn list_paired_devices(conn: &Connection) -> Result<Vec<PairedDevice>> {
    let mut stmt = conn.prepare("SELECT device_json FROM paired_devices ORDER BY rowid ASC")?;
    let devices = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .map(|r| r.map_err(anyhow::Error::from))
        .map(|r| r.and_then(|json| serde_json::from_str(&json).context("invalid paired device")))
        .collect::<Result<Vec<PairedDevice>>>()?;
    Ok(devices)
}

pub fn delete_paired_device(conn: &Connection, device_id: &str) -> Result<()> {
    conn.execute("DELETE FROM paired_devices WHERE device_id = ?1", params![device_id])?;
    Ok(())
}

// ── Activity feed ────────────────────────────────────────────────────────────

/// Insert an activity event and a notification for the local member.
pub fn emit_activity(
    conn: &Connection,
    network_cid_short: &str,
    kind: ActivityKind,
    actor_cid_short: &str,
    summary: &str,
) -> Result<()> {
    let event = ActivityEvent::new(
        network_cid_short.to_string(),
        kind,
        actor_cid_short.to_string(),
        summary.to_string(),
    );
    let event_id = event.id.clone();
    let json = serde_json::to_string(&event)?;
    conn.execute(
        "INSERT OR IGNORE INTO activity_feed (network_cid, event_id, event_json) VALUES (?1, ?2, ?3)",
        params![network_cid_short, &event_id, json],
    )?;

    if let Ok(keypair) = load_identity(conn) {
        let notif = Notification::new(
            network_cid_short.to_string(),
            event_id,
            keypair.cid().short().to_string(),
        );
        let nid = notif.id.clone();
        let njson = serde_json::to_string(&notif)?;
        conn.execute(
            "INSERT OR IGNORE INTO notifications (notif_id, notif_json) VALUES (?1, ?2)",
            params![nid, njson],
        )?;
    }
    Ok(())
}

pub fn list_activity(conn: &Connection, network_cid_short: &str) -> Result<Vec<ActivityEvent>> {
    let mut stmt = conn.prepare(
        "SELECT event_json FROM activity_feed WHERE network_cid = ?1 ORDER BY rowid DESC LIMIT 100",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut events = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        if let Ok(e) = serde_json::from_str(&json) {
            events.push(e);
        }
    }
    Ok(events)
}

pub fn list_activity_all(conn: &Connection) -> Result<Vec<ActivityEvent>> {
    let mut stmt = conn.prepare(
        "SELECT event_json FROM activity_feed ORDER BY rowid DESC LIMIT 200",
    )?;
    let mut rows = stmt.query([])?;
    let mut events = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        if let Ok(e) = serde_json::from_str(&json) {
            events.push(e);
        }
    }
    Ok(events)
}

pub fn list_notifications(conn: &Connection, network_cid_short: &str) -> Result<Vec<Notification>> {
    let mut stmt = conn.prepare(
        "SELECT notif_json FROM notifications WHERE json_extract(notif_json,'$.network_cid_short') = ?1 ORDER BY rowid DESC LIMIT 50",
    )?;
    let mut rows = stmt.query(params![network_cid_short])?;
    let mut notifs = Vec::new();
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        if let Ok(n) = serde_json::from_str(&json) {
            notifs.push(n);
        }
    }
    Ok(notifs)
}

pub fn count_unread_notifications(conn: &Connection, network_cid_short: &str) -> usize {
    conn.query_row(
        "SELECT COUNT(*) FROM notifications WHERE json_extract(notif_json,'$.network_cid_short') = ?1 AND json_extract(notif_json,'$.read') = false",
        params![network_cid_short],
        |r| r.get::<_, i64>(0),
    ).unwrap_or(0) as usize
}

pub fn mark_notification_read(conn: &Connection, notif_id: &str) -> Result<()> {
    let result = conn.query_row(
        "SELECT notif_json FROM notifications WHERE notif_id = ?1",
        params![notif_id],
        |r| r.get::<_, String>(0),
    );
    match result {
        Ok(json) => {
            let mut notif: Notification = serde_json::from_str(&json)?;
            notif.read = true;
            let updated = serde_json::to_string(&notif)?;
            conn.execute(
                "UPDATE notifications SET notif_json = ?1 WHERE notif_id = ?2",
                params![updated, notif_id],
            )?;
            Ok(())
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

// ── Sync ─────────────────────────────────────────────────────────────────────

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

// ── Outbox queue ─────────────────────────────────────────────────────────────

use std::time::{SystemTime, UNIX_EPOCH};

fn unix_now_store() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

/// Record a locally-sent message as pending delivery to peers.
pub fn enqueue_outbox(conn: &Connection, network_cid_short: &str, message_id: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO outbox_queue (network_cid, message_id, queued_at) VALUES (?1, ?2, ?3)",
        params![network_cid_short, message_id, unix_now_store() as i64],
    )?;
    Ok(())
}

/// Clear all pending outbox entries for a network (called after a successful peer sync).
pub fn clear_outbox(conn: &Connection, network_cid_short: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM outbox_queue WHERE network_cid = ?1",
        params![network_cid_short],
    )?;
    Ok(())
}

/// Return the number of pending outbox messages for a network.
pub fn count_outbox(conn: &Connection, network_cid_short: &str) -> u64 {
    conn.query_row(
        "SELECT COUNT(*) FROM outbox_queue WHERE network_cid = ?1",
        params![network_cid_short],
        |r| r.get::<_, i64>(0),
    ).unwrap_or(0) as u64
}

/// Return (network_cid_short, count) for every network with pending outbox messages.
pub fn count_all_outbox(conn: &Connection) -> Vec<(String, u64)> {
    let mut stmt = match conn.prepare(
        "SELECT network_cid, COUNT(*) FROM outbox_queue GROUP BY network_cid",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    });
    match rows {
        Ok(iter) => iter.filter_map(|r| r.ok()).map(|(c, n)| (c, n as u64)).collect(),
        Err(_) => Vec::new(),
    }
}

// ── RCC registrations ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RccRegistration {
    pub network_cid_short: String,
    pub network_cid_full: String,
    pub network_name: String,
    pub admin_email: String,
    pub status: String,
    pub attempts: u32,
    pub last_attempt: Option<u64>,
    pub registered_at: u64,
}

pub fn save_rcc_registration(conn: &Connection, reg: &RccRegistration) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO rcc_registrations
         (network_cid_short, network_cid_full, network_name, admin_email, status, attempts, last_attempt, registered_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            reg.network_cid_short, reg.network_cid_full, reg.network_name, reg.admin_email,
            reg.status, reg.attempts as i64,
            reg.last_attempt.map(|t| t as i64),
            reg.registered_at as i64,
        ],
    )?;
    Ok(())
}

pub fn get_rcc_registration(conn: &Connection, network_cid_short: &str) -> Option<RccRegistration> {
    conn.query_row(
        "SELECT network_cid_short, network_cid_full, network_name, admin_email, status, attempts, last_attempt, registered_at
         FROM rcc_registrations WHERE network_cid_short = ?1",
        params![network_cid_short],
        |r| Ok(RccRegistration {
            network_cid_short: r.get(0)?,
            network_cid_full: r.get(1)?,
            network_name: r.get(2)?,
            admin_email: r.get(3)?,
            status: r.get(4)?,
            attempts: r.get::<_, i64>(5)? as u32,
            last_attempt: r.get::<_, Option<i64>>(6)?.map(|t| t as u64),
            registered_at: r.get::<_, i64>(7)? as u64,
        }),
    ).ok()
}

pub fn list_rcc_registrations(conn: &Connection) -> Vec<RccRegistration> {
    let mut stmt = match conn.prepare(
        "SELECT network_cid_short, network_cid_full, network_name, admin_email, status, attempts, last_attempt, registered_at
         FROM rcc_registrations",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = stmt.query_map([], |r| Ok(RccRegistration {
        network_cid_short: r.get(0)?,
        network_cid_full: r.get(1)?,
        network_name: r.get(2)?,
        admin_email: r.get(3)?,
        status: r.get(4)?,
        attempts: r.get::<_, i64>(5)? as u32,
        last_attempt: r.get::<_, Option<i64>>(6)?.map(|t| t as u64),
        registered_at: r.get::<_, i64>(7)? as u64,
    }));
    match rows {
        Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

pub fn update_rcc_status(
    conn: &Connection,
    network_cid_short: &str,
    status: &str,
    attempts: u32,
    last_attempt: u64,
) -> Result<()> {
    conn.execute(
        "UPDATE rcc_registrations SET status = ?1, attempts = ?2, last_attempt = ?3
         WHERE network_cid_short = ?4",
        params![status, attempts as i64, last_attempt as i64, network_cid_short],
    )?;
    Ok(())
}

// ── ActivityPub ───────────────────────────────────────────────────────────────

use civium_core::{ApFollower, ApPost};

pub fn ap_save_follower(conn: &Connection, network_cid: &str, follower: &ApFollower) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO ap_followers (network_cid, actor_url, inbox_url, shared_inbox, followed_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            network_cid,
            &follower.actor_url,
            &follower.inbox_url,
            &follower.shared_inbox,
            follower.followed_at as i64,
        ],
    )?;
    Ok(())
}

pub fn ap_remove_follower(conn: &Connection, network_cid: &str, actor_url: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM ap_followers WHERE network_cid = ?1 AND actor_url = ?2",
        params![network_cid, actor_url],
    )?;
    Ok(())
}

pub fn ap_list_followers(conn: &Connection, network_cid: &str) -> Result<Vec<ApFollower>> {
    let mut stmt = conn.prepare(
        "SELECT actor_url, inbox_url, shared_inbox, followed_at
         FROM ap_followers WHERE network_cid = ?1 ORDER BY followed_at DESC",
    )?;
    let rows = stmt.query_map(params![network_cid], |r| {
        Ok(ApFollower {
            actor_url:    r.get(0)?,
            inbox_url:    r.get(1)?,
            shared_inbox: r.get(2)?,
            followed_at:  r.get::<_, i64>(3)? as u64,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn ap_save_post(conn: &Connection, post: &ApPost) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO ap_posts (network_cid, note_id, content, ap_activity_id, posted_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            &post.network_cid,
            &post.note_id,
            &post.content,
            &post.ap_activity_id,
            post.posted_at as i64,
        ],
    )?;
    Ok(())
}

pub fn ap_list_posts(conn: &Connection, network_cid: &str, limit: usize) -> Result<Vec<ApPost>> {
    let mut stmt = conn.prepare(
        "SELECT id, network_cid, note_id, content, ap_activity_id, posted_at
         FROM ap_posts WHERE network_cid = ?1 ORDER BY posted_at DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![network_cid, limit as i64], |r| {
        Ok(ApPost {
            id:             r.get(0)?,
            network_cid:    r.get(1)?,
            note_id:        r.get(2)?,
            content:        r.get(3)?,
            ap_activity_id: r.get(4)?,
            posted_at:      r.get::<_, i64>(5)? as u64,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

// ── Inter-network connections ─────────────────────────────────────────────────

pub fn save_connection(
    conn: &Connection,
    network_cid: &str,
    record: &civium_core::ConnectionRecord,
) -> Result<()> {
    let json = serde_json::to_string(record).context("serialize ConnectionRecord")?;
    conn.execute(
        "INSERT OR REPLACE INTO connections (network_cid, peer_cid_full, record_json)
         VALUES (?1, ?2, ?3)",
        params![network_cid, record.peer_cid_full, json],
    )?;
    Ok(())
}

pub fn load_connection(
    conn: &Connection,
    network_cid: &str,
    peer_cid_full: &str,
) -> Result<Option<civium_core::ConnectionRecord>> {
    let result = conn.query_row(
        "SELECT record_json FROM connections WHERE network_cid = ?1 AND peer_cid_full = ?2",
        params![network_cid, peer_cid_full],
        |r| r.get::<_, String>(0),
    );
    match result {
        Ok(json) => Ok(Some(serde_json::from_str(&json).context("deserialize ConnectionRecord")?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn list_connections(
    conn: &Connection,
    network_cid: &str,
) -> Result<Vec<civium_core::ConnectionRecord>> {
    let mut stmt = conn.prepare(
        "SELECT record_json FROM connections WHERE network_cid = ?1",
    )?;
    let rows = stmt.query_map(params![network_cid], |r| r.get::<_, String>(0))?;
    let mut records = Vec::new();
    for row in rows {
        if let Ok(json) = row {
            if let Ok(rec) = serde_json::from_str::<civium_core::ConnectionRecord>(&json) {
                records.push(rec);
            }
        }
    }
    Ok(records)
}

// ── Hub config ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HubConfig {
    pub network_cid_short: String,
    pub hub_url:           String,
    pub enabled:           bool,
    pub last_sync_ts:      u64,
}

pub fn get_hub_config(conn: &Connection, network_cid_short: &str) -> Option<HubConfig> {
    conn.query_row(
        "SELECT hub_url, enabled, last_sync_ts FROM hub_config WHERE network_cid_short = ?1",
        params![network_cid_short],
        |r| Ok(HubConfig {
            network_cid_short: network_cid_short.to_string(),
            hub_url:           r.get(0)?,
            enabled:           r.get::<_, i64>(1)? != 0,
            last_sync_ts:      r.get::<_, i64>(2)? as u64,
        }),
    ).ok()
}

pub fn set_hub_config(conn: &Connection, cfg: &HubConfig) -> Result<()> {
    conn.execute(
        "INSERT INTO hub_config (network_cid_short, hub_url, enabled, last_sync_ts)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(network_cid_short) DO UPDATE SET
             hub_url      = excluded.hub_url,
             enabled      = excluded.enabled,
             last_sync_ts = excluded.last_sync_ts",
        params![cfg.network_cid_short, cfg.hub_url, cfg.enabled as i64, cfg.last_sync_ts as i64],
    )?;
    Ok(())
}

pub fn update_hub_last_sync(conn: &Connection, network_cid_short: &str, ts: u64) -> Result<()> {
    conn.execute(
        "UPDATE hub_config SET last_sync_ts = ?1 WHERE network_cid_short = ?2",
        params![ts as i64, network_cid_short],
    )?;
    Ok(())
}

// ── Node settings ─────────────────────────────────────────────────────────────

pub fn get_node_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM node_settings WHERE key = ?1",
        params![key],
        |r| r.get::<_, String>(0),
    ).ok()
}

pub fn set_node_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO node_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// Retourne la NodeConfig à utiliser au démarrage, construite depuis la base.
pub fn load_node_config(conn: &Connection) -> civium_core::NodeConfig {
    let tcp_port = get_node_setting(conn, "tcp_port")
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(0);
    let ws_port = get_node_setting(conn, "ws_port")
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(0);
    let external_addr = get_node_setting(conn, "external_addr")
        .filter(|v| !v.is_empty());

    civium_core::NodeConfig {
        listen_tcp:  format!("/ip4/0.0.0.0/tcp/{tcp_port}"),
        listen_quic: format!("/ip4/0.0.0.0/udp/{tcp_port}/quic-v1"),
        listen_ws:   Some(format!("/ip4/0.0.0.0/tcp/{ws_port}/ws")),
        external_addr,
        bootstrap_peers:         vec![],
        mcp_port:                None,
        auto_accept_connections: false,
    }
}
