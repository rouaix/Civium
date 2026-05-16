//! `CiviumClient` — main entry point for SDK users.

use std::path::{Path, PathBuf};

use civium_core::CiviumKeypair;
use anyhow::Context;

use crate::{IdentityInfo, NetworkInfo};

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration for a [`CiviumClient`].
pub struct ClientConfig {
    pub(crate) data_dir: PathBuf,
}

impl ClientConfig {
    /// Start building a configuration.
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::default()
    }
}

/// Builder for [`ClientConfig`].
#[derive(Default)]
pub struct ClientConfigBuilder {
    data_dir: Option<PathBuf>,
}

impl ClientConfigBuilder {
    /// Set the directory where identity and network data are stored.
    pub fn data_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.data_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Build the configuration. Panics if `data_dir` was not set.
    pub fn build(self) -> ClientConfig {
        ClientConfig {
            data_dir: self.data_dir.expect("data_dir is required"),
        }
    }
}

// ── Client ────────────────────────────────────────────────────────────────────

/// Main entry point for SDK users.
///
/// Wraps the Civium data store and provides high-level operations for
/// identity, network, messaging, and governance.
pub struct CiviumClient {
    data_dir: PathBuf,
    keypair: Option<CiviumKeypair>,
}

impl CiviumClient {
    /// Open a Civium client backed by the given data directory.
    ///
    /// Loads the identity from the store if one exists. Does not start a P2P
    /// node — call [`CiviumClient::start_node`] separately if needed.
    pub async fn open(config: ClientConfig) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&config.data_dir)
            .with_context(|| format!("cannot create data dir: {}", config.data_dir.display()))?;

        let keypair = Self::try_load_keypair(&config.data_dir);

        Ok(Self { data_dir: config.data_dir, keypair })
    }

    fn try_load_keypair(data_dir: &Path) -> Option<CiviumKeypair> {
        let key_path = data_dir.join("identity.key");
        let bytes = std::fs::read(&key_path).ok()?;
        let b58 = String::from_utf8(bytes).ok()?;
        CiviumKeypair::from_secret_b58(b58.trim()).ok()
    }

    // ── Identity ──────────────────────────────────────────────────────────────

    /// Return the current identity, or `None` if no identity exists yet.
    pub async fn identity(&self) -> anyhow::Result<Option<IdentityInfo>> {
        Ok(self.keypair.as_ref().map(IdentityInfo::from))
    }

    /// Generate a new Ed25519 identity and persist it to the data directory.
    ///
    /// Returns an error if an identity already exists (use [`CiviumClient::identity`]
    /// first to check).
    pub async fn identity_create(&mut self) -> anyhow::Result<IdentityInfo> {
        if self.keypair.is_some() {
            anyhow::bail!("identity already exists in {}", self.data_dir.display());
        }
        let kp = CiviumKeypair::generate()
            .map_err(|e| anyhow::anyhow!("key generation failed: {e}"))?;
        let key_path = self.data_dir.join("identity.key");
        std::fs::write(&key_path, kp.secret_b58())
            .with_context(|| format!("cannot write identity key to {}", key_path.display()))?;
        let info = IdentityInfo::from(&kp);
        self.keypair = Some(kp);
        Ok(info)
    }

    /// Import an existing identity from a base58 secret key.
    pub async fn identity_import(&mut self, secret_b58: &str) -> anyhow::Result<IdentityInfo> {
        let kp = CiviumKeypair::from_secret_b58(secret_b58)
            .map_err(|e| anyhow::anyhow!("invalid secret key: {e}"))?;
        let key_path = self.data_dir.join("identity.key");
        std::fs::write(&key_path, kp.secret_b58())
            .with_context(|| format!("cannot write identity key to {}", key_path.display()))?;
        let info = IdentityInfo::from(&kp);
        self.keypair = Some(kp);
        Ok(info)
    }

    // ── Networks ──────────────────────────────────────────────────────────────

    /// List all networks stored in the data directory.
    ///
    /// Networks are stored as JSON files under `data_dir/networks/`.
    pub async fn networks(&self) -> anyhow::Result<Vec<NetworkInfo>> {
        let dir = self.data_dir.join("networks");
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let json = std::fs::read_to_string(entry.path())?;
            if let Ok(net) = serde_json::from_str::<civium_core::network::NetworkData>(&json) {
                result.push(NetworkInfo {
                    cid_short: net.cid_short.clone(),
                    cid_full:  net.cid_full.clone(),
                    name:      net.name.clone(),
                    member_count: net.members.len() as u32,
                });
            }
        }
        Ok(result)
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// Path to the data directory.
    pub fn data_dir(&self) -> &Path { &self.data_dir }

    /// `true` if an identity is loaded.
    pub fn has_identity(&self) -> bool { self.keypair.is_some() }
}
