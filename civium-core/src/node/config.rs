/// Configuration for a local Civium node.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// TCP listen multiaddr (e.g. "/ip4/0.0.0.0/tcp/0")
    pub listen_tcp: String,
    /// QUIC listen multiaddr (e.g. "/ip4/0.0.0.0/udp/0/quic-v1")
    pub listen_quic: String,
    /// Optional bootstrap peer multiaddrs to dial on startup
    pub bootstrap_peers: Vec<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            listen_tcp: "/ip4/0.0.0.0/tcp/0".into(),
            listen_quic: "/ip4/0.0.0.0/udp/0/quic-v1".into(),
            bootstrap_peers: vec![],
        }
    }
}
