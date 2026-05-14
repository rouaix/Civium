mod behaviour;
mod config;

pub use config::NodeConfig;

use behaviour::CiviumBehaviour;
use crate::{Cid, CiviumError, CiviumKeypair};
use futures::StreamExt;
use libp2p::{
    identify, mdns,
    noise, tcp, yamux,
    Multiaddr, SwarmBuilder,
    swarm::{SwarmEvent, dial_opts::DialOpts},
};
use std::time::Duration;
use tracing::{info, warn, debug};

pub struct CiviumNode {
    swarm: libp2p::Swarm<CiviumBehaviour>,
    cid: Cid,
}

impl CiviumNode {
    pub async fn new(keypair: CiviumKeypair, config: NodeConfig) -> Result<Self, CiviumError> {
        let cid = keypair.cid();
        let libp2p_keypair = keypair.libp2p_keypair().clone();

        // Pre-create the behaviour (needs peer_id + public key, which we can derive here)
        let peer_id = libp2p_keypair.public().to_peer_id();
        let local_pub_key = libp2p_keypair.public().clone();
        let behaviour = CiviumBehaviour::new(peer_id, local_pub_key)
            .map_err(|e| CiviumError::Node(e.to_string()))?;

        let mut swarm = SwarmBuilder::with_existing_identity(libp2p_keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default().nodelay(true),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| CiviumError::Node(e.to_string()))?
            .with_quic()
            .with_behaviour(|_| behaviour)
            .map_err(|e| CiviumError::Node(e.to_string()))?
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(Duration::from_secs(60))
            })
            .build();

        let tcp_addr: Multiaddr = config.listen_tcp.parse()
            .map_err(|e: libp2p::multiaddr::Error| CiviumError::Node(e.to_string()))?;
        let quic_addr: Multiaddr = config.listen_quic.parse()
            .map_err(|e: libp2p::multiaddr::Error| CiviumError::Node(e.to_string()))?;

        swarm.listen_on(tcp_addr)
            .map_err(|e| CiviumError::Node(e.to_string()))?;
        swarm.listen_on(quic_addr)
            .map_err(|e| CiviumError::Node(e.to_string()))?;

        for addr_str in &config.bootstrap_peers {
            let addr: Multiaddr = addr_str.parse()
                .map_err(|e: libp2p::multiaddr::Error| CiviumError::Node(e.to_string()))?;
            swarm
                .dial(DialOpts::unknown_peer_id().address(addr).build())
                .map_err(|e| CiviumError::Node(e.to_string()))?;
        }

        Ok(Self { swarm, cid })
    }

    pub fn cid(&self) -> &Cid {
        &self.cid
    }

    pub fn local_peer_id(&self) -> &libp2p::PeerId {
        self.swarm.local_peer_id()
    }

    /// Run the node event loop — blocks until the future is dropped.
    pub async fn run(&mut self) {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!(cid = %self.cid, addr = %address, "listening");
                }
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    info!(%peer_id, addr = %endpoint.get_remote_address(), "connected");
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    debug!(%peer_id, ?cause, "disconnected");
                }
                SwarmEvent::Behaviour(event) => self.handle_behaviour(event),
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    warn!(?peer_id, %error, "outgoing connection failed");
                }
                _ => {}
            }
        }
    }

    fn handle_behaviour(&mut self, event: behaviour::CiviumBehaviourEvent) {
        match event {
            behaviour::CiviumBehaviourEvent::Identify(identify::Event::Received {
                peer_id,
                info,
                ..
            }) => {
                for addr in info.listen_addrs {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr);
                }
            }
            behaviour::CiviumBehaviourEvent::Mdns(mdns::Event::Discovered(peers)) => {
                for (peer_id, addr) in peers {
                    info!(%peer_id, %addr, "mdns: peer discovered");
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr.clone());
                    let _ = self.swarm.dial(
                        DialOpts::peer_id(peer_id).addresses(vec![addr]).build()
                    );
                }
            }
            behaviour::CiviumBehaviourEvent::Mdns(mdns::Event::Expired(peers)) => {
                for (peer_id, addr) in peers {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .remove_address(&peer_id, &addr);
                }
            }
            behaviour::CiviumBehaviourEvent::Kademlia(e) => {
                debug!(?e, "kademlia event");
            }
            _ => {}
        }
    }
}
