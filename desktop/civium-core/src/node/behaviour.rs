use libp2p::{
    autonat,
    connection_limits,
    identify, kad, mdns,
    request_response,
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
    identity::PublicKey,
};
use std::time::Duration;

use super::protocol::{CiviumRequest, CiviumResponse};

pub type ReqResBehaviour =
    request_response::cbor::Behaviour<CiviumRequest, CiviumResponse>;

/// Max simultaneous connections from a single peer.
const MAX_CONNECTIONS_PER_PEER: u32 = 8;
/// Max total established connections (in + out).
const MAX_CONNECTIONS_TOTAL: u32 = 256;

#[derive(NetworkBehaviour)]
pub struct CiviumBehaviour {
    pub kademlia:          kad::Behaviour<kad::store::MemoryStore>,
    pub identify:          identify::Behaviour,
    pub mdns:              mdns::tokio::Behaviour,
    pub request_response:  ReqResBehaviour,
    pub connection_limits: connection_limits::Behaviour,
    /// AutoNAT — probes whether this node is reachable from the public internet.
    /// Circuit relay (libp2p-relay feature) requires a separate transport that is
    /// incompatible with the WebSocket transport in libp2p 0.55's builder API.
    /// Workaround: use Cloudflare Tunnel via `external_addr` until native relay is wired.
    pub autonat:           autonat::Behaviour,
}

impl CiviumBehaviour {
    pub fn new(peer_id: PeerId, local_public_key: PublicKey) -> Result<Self, std::io::Error> {
        let kademlia = {
            // Bound the DHT store to prevent memory exhaustion from malicious peers.
            let store_config = kad::store::MemoryStoreConfig {
                max_records: 4096,
                max_provided_keys: 1024,
                ..Default::default()
            };
            let store = kad::store::MemoryStore::with_config(peer_id, store_config);
            let protocol = StreamProtocol::new("/civium/kad/1.0.0");
            let config = kad::Config::new(protocol);
            kad::Behaviour::with_config(peer_id, store, config)
        };

        let identify = identify::Behaviour::new(
            identify::Config::new("/civium/identify/1.0.0".to_string(), local_public_key)
                .with_push_listen_addr_updates(true),
        );

        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        let request_response = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/civium/1.0.0"),
                request_response::ProtocolSupport::Full,
            )],
            request_response::Config::default()
                .with_request_timeout(Duration::from_secs(30)),
        );

        let connection_limits = connection_limits::Behaviour::new(
            connection_limits::ConnectionLimits::default()
                .with_max_established_per_peer(Some(MAX_CONNECTIONS_PER_PEER))
                .with_max_established(Some(MAX_CONNECTIONS_TOTAL)),
        );

        let autonat = autonat::Behaviour::new(peer_id, autonat::Config {
            timeout: Duration::from_secs(30),
            ..Default::default()
        });

        Ok(Self { kademlia, identify, mdns, request_response, connection_limits, autonat })
    }
}
