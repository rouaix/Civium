//! Minimal synchronous SQLite store for the mobile FFI layer.
//! Uses the same civium.db schema as civium-tauri so a paired device can read
//! a database that was seeded by the desktop app.

use anyhow::{Context, Result};
use civium_core::{network::Network, CiviumKeypair, Message};
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
        .context("aucune identité trouvée")?;
    CiviumKeypair::from_secret_b58(&secret).map_err(|e| anyhow::anyhow!("{e}"))
}

pub fn load_identity_row(conn: &Connection) -> Result<(String, String, String)> {
    conn.query_row(
        "SELECT secret_b58, cid_short, cid_full FROM identity WHERE id = 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
    .context("aucune identité trouvée")
}

// ── Networks ──────────────────────────────────────────────────────────────────

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

pub fn save_network(conn: &Connection, network: &Network) -> Result<()> {
    let json = serde_json::to_string(&network.data)?;
    conn.execute(
        "INSERT OR REPLACE INTO networks (cid_short, data_json) VALUES (?1, ?2)",
        params![network.cid_short(), json],
    )?;
    Ok(())
}

// ── Messages ──────────────────────────────────────────────────────────────────

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

pub fn save_message(conn: &Connection, network_cid_short: &str, msg: &Message) -> Result<()> {
    let json = serde_json::to_string(msg)?;
    conn.execute(
        "INSERT OR REPLACE INTO messages (network_cid, message_id, message_json, in_outbox)
         VALUES (?1, ?2, ?3, 1)",
        params![network_cid_short, msg.id, json],
    )?;
    Ok(())
}
