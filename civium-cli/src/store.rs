//! Phase 0 persistence — plain JSON files.
//! Replaced by SQLCipher in weeks 9-10 (Tauri app).

use anyhow::{Context, Result};
use civium_core::{network::Network, CiviumKeypair, Mailbox};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

// ── Identity ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityFile {
    pub secret_b58: String,
    pub cid_short: String,
    pub cid_full: String,
}

pub fn identity_path(data_dir: &Path) -> PathBuf {
    data_dir.join("identity.json")
}

pub fn save_identity(data_dir: &Path, keypair: &CiviumKeypair) -> Result<()> {
    let cid = keypair.cid();
    let file = IdentityFile {
        secret_b58: keypair.secret_b58(),
        cid_short: cid.short().to_string(),
        cid_full: cid.full().to_string(),
    };
    write_json(data_dir, &identity_path(data_dir), &file)
}

pub fn load_identity(data_dir: &Path) -> Result<CiviumKeypair> {
    let path = identity_path(data_dir);
    let file: IdentityFile = read_json(&path)
        .with_context(|| format!("no identity found at {}", path.display()))?;
    CiviumKeypair::from_secret_b58(&file.secret_b58)
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub fn identity_exists(data_dir: &Path) -> bool {
    identity_path(data_dir).exists()
}

// ── Network ───────────────────────────────────────────────────────────────────

pub fn network_dir(data_dir: &Path, cid_short: &str) -> PathBuf {
    data_dir.join("networks").join(cid_short)
}

pub fn network_path(data_dir: &Path, cid_short: &str) -> PathBuf {
    network_dir(data_dir, cid_short).join("network.json")
}

pub fn save_network(data_dir: &Path, network: &Network) -> Result<()> {
    let path = network_path(data_dir, network.cid_short());
    write_json(data_dir, &path, &network.data)
}

pub fn load_network(data_dir: &Path, cid_short: &str) -> Result<Network> {
    let path = network_path(data_dir, cid_short);
    let data = read_json(&path)
        .with_context(|| format!("no network found at {}", path.display()))?;
    Network::from_data(data).map_err(|e| anyhow::anyhow!("{e}"))
}

/// List all network CID short values stored in data_dir/networks/.
pub fn list_network_cids(data_dir: &Path) -> Vec<String> {
    let networks_dir = data_dir.join("networks");
    if !networks_dir.exists() {
        return vec![];
    }
    fs::read_dir(&networks_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect()
}

// ── Mailbox ───────────────────────────────────────────────────────────────────

pub fn mailbox_path(data_dir: &Path, network_cid_short: &str) -> PathBuf {
    network_dir(data_dir, network_cid_short).join("mailbox.json")
}

pub fn save_mailbox(data_dir: &Path, network_cid_short: &str, mailbox: &Mailbox) -> Result<()> {
    let path = mailbox_path(data_dir, network_cid_short);
    write_json(data_dir, &path, mailbox)
}

pub fn load_mailbox(data_dir: &Path, network_cid_short: &str) -> Result<Mailbox> {
    let path = mailbox_path(data_dir, network_cid_short);
    if !path.exists() {
        return Ok(Mailbox::new());
    }
    read_json(&path).with_context(|| format!("cannot read mailbox at {}", path.display()))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn write_json<T: serde::Serialize>(data_dir: &Path, path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot create directory {}", data_dir.display()))?;
    }
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, json).with_context(|| format!("cannot write {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("cannot read {}", path.display()))?;
    serde_json::from_str(&contents).with_context(|| format!("invalid JSON in {}", path.display()))
}
