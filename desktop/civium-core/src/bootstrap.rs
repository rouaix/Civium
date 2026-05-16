//! Constants and helpers for the Civium root network.
//!
//! The root network "civium" is the first Civium network. Every newly created
//! Civium network automatically connects to it (permanent, like RCC registration).
//!
//! # Setup
//!
//! The root node runs `civium node start --auto-accept-connections` on the server.
//! Its CID and WebSocket address are hardcoded here so all clients can reach it
//! without a discovery step.
//!
//! **Before production:** generate the root keypair once on the server, derive its CID,
//! replace the placeholder constants below, rebuild all clients.

/// Full CID of the root "civium" network.
///
/// **Placeholder** — must be replaced with the actual CID generated on the server.
pub const CIVIUM_ROOT_NETWORK_CID_FULL: &str = "";

/// Short CID (first 8 chars) of the root "civium" network.
///
/// **Placeholder** — must match `CIVIUM_ROOT_NETWORK_CID_FULL[..8]`.
pub const CIVIUM_ROOT_NETWORK_CID_SHORT: &str = "";

/// Name of the root network as stored in its `NetworkData`.
pub const CIVIUM_ROOT_NETWORK_NAME: &str = "civium";

/// WebSocket multiaddr of the root civium node.
///
/// **Placeholder** — fill in once the server node is running.
/// Example: `"/dns4/www.rouaix.com/tcp/9944/ws/p2p/<PeerId>"`
pub const CIVIUM_ROOT_NODE_ADDR: &str = "";

/// `true` when all root network constants have been configured.
///
/// Checked before attempting auto-connection so clients fail gracefully
/// during development.
pub fn root_configured() -> bool {
    !CIVIUM_ROOT_NETWORK_CID_FULL.is_empty()
        && !CIVIUM_ROOT_NODE_ADDR.is_empty()
}
