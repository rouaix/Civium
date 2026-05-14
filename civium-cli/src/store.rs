//! Phase 0.5 persistence — SQLite (structured, transactional, query-ready).
//!
//! Upgrade path to full encryption:
//!   1. Add the `rusqlcipher` crate (or `rusqlite` with `sqlcipher` feature).
//!   2. In `open_db`, before running the schema:
//!        conn.execute_batch(&format!("PRAGMA key='{}';", passphrase))?;
//!   The passphrase is provided by the user at login in the Tauri app (weeks 9-10 final).

use anyhow::{Context, Result};
use civium_core::{network::Network, ConnectionRecord, CiviumKeypair, Mailbox, Message};
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
