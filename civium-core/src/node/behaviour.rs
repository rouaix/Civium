use libp2p::{
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

#[derive(NetworkBehaviour)]
pub struct CiviumBehaviour {
    pub kademlia:         kad::Behaviour<kad::store::MemoryStore>,
    pub identify:         identify::Behaviour,
    pub mdns:             mdns::tokio::Behaviour,
    pub request_response: ReqResBehaviour,
}

impl CiviumBehaviour {
    pub fn new(peer_id: PeerId, local_public_key: PublicKey) -> Result<Self, std::io::Error> {
        let kademlia = {
            let store = kad::store::MemoryStore::new(peer_id);
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

        Ok(Self { kademlia, identify, mdns, request_response })
    }
}
