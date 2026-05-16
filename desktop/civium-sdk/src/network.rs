//! Network discovery and metadata.
//!
//! Networks are stored as JSON files under `data_dir/networks/` and loaded
//! by [`CiviumClient::networks`](crate::CiviumClient::networks).

/// Summary of a Civium network stored in the local data directory.
///
/// # Example
///
/// ```rust,no_run
/// # use civium_sdk::{CiviumClient, ClientConfig};
/// # async fn example() -> anyhow::Result<()> {
/// # let client = CiviumClient::open(ClientConfig::builder().data_dir("/tmp/c").build()).await?;
/// let networks = client.networks().await?;
/// for net in &networks {
///     println!("[{}] {} — {} membres", net.cid_short, net.name, net.member_count);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    /// Short CID (first 8 chars) — used for UI display and DHT lookups.
    pub cid_short: String,
    /// Full CID — unique identifier derived from the creator's keypair + timestamp.
    pub cid_full: String,
    /// Human-readable display name chosen by the network creator.
    pub name: String,
    /// Number of members currently recorded in the local network snapshot.
    pub member_count: u32,
}
