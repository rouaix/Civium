/// Configuration for a local Civium node.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// TCP listen multiaddr (e.g. "/ip4/0.0.0.0/tcp/0")
    pub listen_tcp: String,
    /// QUIC listen multiaddr (e.g. "/ip4/0.0.0.0/udp/0/quic-v1")
    pub listen_quic: String,
    /// WebSocket listen multiaddr (e.g. "/ip4/0.0.0.0/tcp/0/ws").
    /// None = WebSocket transport not bound (transport still compiled in, just not listening).
    pub listen_ws: Option<String>,
    /// Optional bootstrap peer multiaddrs to dial on startup
    pub bootstrap_peers: Vec<String>,
    /// Port for the built-in MCP HTTP server (None = disabled)
    pub mcp_port: Option<u16>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            listen_tcp:      "/ip4/0.0.0.0/tcp/0".into(),
            listen_quic:     "/ip4/0.0.0.0/udp/0/quic-v1".into(),
            listen_ws:       Some("/ip4/0.0.0.0/tcp/0/ws".into()),
            bootstrap_peers: vec![],
            mcp_port:        None,
        }
    }
}
