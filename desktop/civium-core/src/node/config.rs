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
    /// External address announced to the DHT so remote peers can reach this node.
    /// Examples:
    ///   "/ip4/203.0.113.42/tcp/4001"           — IP publique + port forwardé
    ///   "/dns4/xyz.trycloudflare.com/tcp/443/wss" — Cloudflare Tunnel WebSocket
    pub external_addr: Option<String>,
    /// Optional bootstrap peer multiaddrs to dial on startup
    pub bootstrap_peers: Vec<String>,
    /// Port for the built-in MCP HTTP server (None = disabled)
    pub mcp_port: Option<u16>,
    /// When `true`, incoming `ConnectRequest`s are automatically accepted without
    /// admin review. Enable this on the root "civium" network server node.
    pub auto_accept_connections: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            listen_tcp:               "/ip4/0.0.0.0/tcp/0".into(),
            listen_quic:              "/ip4/0.0.0.0/udp/0/quic-v1".into(),
            listen_ws:                Some("/ip4/0.0.0.0/tcp/0/ws".into()),
            external_addr:            None,
            bootstrap_peers:          vec![],
            mcp_port:                 None,
            auto_accept_connections:  false,
        }
    }
}
