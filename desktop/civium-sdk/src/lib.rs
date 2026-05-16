//! # Civium SDK
//!
//! High-level interface for building applications on the Civium protocol.
//!
//! This crate wraps [`civium-core`] and exposes a clean, stable API for
//! third-party integrators who want to embed Civium functionality in their
//! applications without depending on civium-core internals directly.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use civium_sdk::{CiviumClient, ClientConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = ClientConfig::builder()
//!         .data_dir("/path/to/civium-data")
//!         .build();
//!
//!     let mut client = CiviumClient::open(config).await?;
//!
//!     // Create identity if none exists, otherwise load it.
//!     let identity = match client.identity().await? {
//!         Some(id) => id,
//!         None => client.identity_create().await?,
//!     };
//!     println!("CID: {}", identity.cid_short);
//!
//!     // List networks
//!     let networks = client.networks().await?;
//!     for net in &networks {
//!         println!("Network: {} ({})", net.name, net.cid_short);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Modules
//!
//! | Module | Contents |
//! |---|---|
//! | [`client`] | [`CiviumClient`] and [`ClientConfig`] — main entry points |
//! | [`identity`] | [`IdentityInfo`] — CID, public key |
//! | [`network`] | [`NetworkInfo`] — network summary |
//! | [`messaging`] | [`MessageInfo`] — decrypted message |
//! | [`governance`] | [`ProposalInfo`], [`VoteInfo`] — proposals and votes |
//! | [`prelude`] | Glob re-export of all public types |
//!
//! ## Data directory layout
//!
//! The SDK stores all persistent state under the `data_dir` passed to
//! [`ClientConfig::builder`]:
//!
//! ```text
//! data_dir/
//!   identity.key        ← base58 Ed25519 secret key (created by identity_create)
//!   networks/
//!     <cid_short>.json  ← one JSON file per joined network
//! ```
//!
//! ## Security notes
//!
//! - The secret key file (`identity.key`) must be protected at rest; the SDK
//!   does not encrypt it — encryption at the filesystem level is the
//!   integrator's responsibility.
//! - [`CiviumClient::identity_import`] overwrites any existing key without
//!   prompting. Check [`CiviumClient::has_identity`] first if needed.
//! - [`FraudAlert`]s received from the network are verified against the RCC
//!   public key before being surfaced. The `RCC_PUBLIC_KEY_B58` constant in
//!   `civium-core` **must** be set to the real key before a production build.

pub mod client;
pub mod identity;
pub mod network;
pub mod messaging;
pub mod governance;
pub mod prelude;

pub use client::{CiviumClient, ClientConfig, ClientConfigBuilder};
pub use identity::IdentityInfo;
pub use network::NetworkInfo;
pub use messaging::MessageInfo;
pub use governance::{ProposalInfo, VoteInfo};

// Re-export key civium-core types for integrators who need them.
pub use civium_core::{
    CiviumError,
    CiviumKeypair,
    Cid,
    GroupKey,
    TrustCircle,
    CertificationLevel,
    PluginManifest,
    PluginPermission,
    FraudAlert,
    RCC_URL,
};
