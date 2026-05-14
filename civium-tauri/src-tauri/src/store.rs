//! SQLite store for the Tauri app — same schema as civium-cli.
//! Both apps share the same civium.db file under the data directory.

use anyhow::{Context, Result};
use civium_core::{network::Network, CiviumKeypair, MemberRecord, Message};
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
