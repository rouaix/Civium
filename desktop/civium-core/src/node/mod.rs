mod behaviour;
mod config;
pub mod protocol;

pub use config::NodeConfig;
pub use protocol::{CiviumRequest, CiviumResponse};

use behaviour::CiviumBehaviour;
use crate::{Cid, CiviumError, CiviumKeypair};
use futures::StreamExt;
use libp2p::{
    identify, kad, mdns,
    request_response::{self, InboundRequestId, OutboundRequestId, ResponseChannel},
    noise, tcp, yamux,
    Multiaddr, PeerId, SwarmBuilder,
    swarm::{SwarmEvent, dial_opts::DialOpts},
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};

// ── Public API types ──────────────────────────────────────────────────────────

/// Commands sent to the node from the application layer.
#[derive(Debug)]
pub enum NodeCommand {
    /// Publish our listen addresses for `network_cid_short` to the DHT.
    AnnounceNetwork { network_cid_short: String },
    /// Look up addresses of peers that belong to `network_cid_short`.
    DiscoverPeers { network_cid_short: String },
    /// Send a Civium request to a specific peer.
    SendRequest { peer: PeerId, request: CiviumRequest },
    /// Respond to an inbound request (pair with the matching `request_id`).
    Respond { request_id: InboundRequestId, response: CiviumResponse },
    /// Dial a peer at a known address.
    Dial { addr: Multiaddr },
}

/// Events emitted by the node to the application layer.
#[derive(Debug)]
pub enum NodeEvent {
    /// The node is now listening on `addr`.
    Listening { addr: Multiaddr },
    /// A peer connected.
    PeerConnected { peer_id: PeerId },
    /// DHT lookup returned addresses for `network_cid_short`.
    PeersDiscovered { network_cid_short: String, peer_addrs: Vec<Multiaddr> },
    /// An inbound Civium request arrived — application must reply via `NodeCommand::Respond`.
    InboundRequest { from: PeerId, request_id: InboundRequestId, request: CiviumRequest },
    /// Response to a previously sent request arrived.
    OutboundResponse { request_id: OutboundRequestId, response: CiviumResponse },
}

// ── Node ──────────────────────────────────────────────────────────────────────

pub struct CiviumNode {
    swarm:            libp2p::Swarm<CiviumBehaviour>,
    cid:              Cid,
    listen_addrs:     Vec<Multiaddr>,
    command_rx:       mpsc::Receiver<NodeCommand>,
    event_tx:         mpsc::Sender<NodeEvent>,
    /// Pending response channels, keyed by inbound request ID.
    pending_responses: HashMap<InboundRequestId, ResponseChannel<CiviumResponse>>,
}

/// Handle held by the application — send commands, receive events.
pub struct NodeHandle {
    pub commands: mpsc::Sender<NodeCommand>,
    pub events:   mpsc::Receiver<NodeEvent>,
    pub peer_id:  PeerId,
}

impl CiviumNode {
    pub async fn new(keypair: CiviumKeypair, config: NodeConfig)
        -> Result<(Self, NodeHandle), CiviumError>
    {
        let cid = keypair.cid();
        let libp2p_keypair = keypair.libp2p_keypair().clone();
        let peer_id = libp2p_keypair.public().to_peer_id();
        let local_pub_key = libp2p_keypair.public().clone();

        let behaviour = CiviumBehaviour::new(peer_id, local_pub_key)
            .map_err(|e| CiviumError::Node(e.to_string()))?;

        // libp2p 0.55: with_websocket() is only available on TcpPhase (before with_quic()).
        // TCP + QUIC + WS as a three-transport combo is not supported by the builder API.
        // QUIC is kept in NodeConfig for future use; for now we use TCP + WebSocket.
        let mut swarm = SwarmBuilder::with_existing_identity(libp2p_keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default().nodelay(true),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| CiviumError::Node(e.to_string()))?
            .with_websocket(noise::Config::new, yamux::Config::default)
            .await
            .map_err(|e| CiviumError::Node(e.to_string()))?
            .with_behaviour(|_| behaviour)
            .map_err(|e| CiviumError::Node(e.to_string()))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        let tcp_addr: Multiaddr = config.listen_tcp.parse()
            .map_err(|e: libp2p::multiaddr::Error| CiviumError::Node(e.to_string()))?;

        swarm.listen_on(tcp_addr).map_err(|e: libp2p::TransportError<std::io::Error>| CiviumError::Node(e.to_string()))?;

        if let Some(ws_str) = &config.listen_ws {
            let ws_addr: Multiaddr = ws_str.parse()
                .map_err(|e: libp2p::multiaddr::Error| CiviumError::Node(e.to_string()))?;
            swarm.listen_on(ws_addr).map_err(|e: libp2p::TransportError<std::io::Error>| CiviumError::Node(e.to_string()))?;
        }

        for addr_str in &config.bootstrap_peers {
            let addr: Multiaddr = addr_str.parse()
                .map_err(|e: libp2p::multiaddr::Error| CiviumError::Node(e.to_string()))?;
            swarm.dial(DialOpts::unknown_peer_id().address(addr).build())
                .map_err(|e: libp2p::swarm::DialError| CiviumError::Node(e.to_string()))?;
        }

        let (cmd_tx, cmd_rx) = mpsc::channel(64);
        let (evt_tx, evt_rx) = mpsc::channel(64);

        let handle = NodeHandle {
            commands: cmd_tx,
            events:   evt_rx,
            peer_id,
        };

        let node = Self {
            swarm,
            cid,
            listen_addrs:     Vec::new(),
            command_rx:       cmd_rx,
            event_tx:         evt_tx,
            pending_responses: HashMap::new(),
        };

        Ok((node, handle))
    }

    pub fn cid(&self) -> &Cid { &self.cid }
}

/// Extract a PeerId from a multiaddr containing a `/p2p/<peer_id>` component.
pub fn peer_id_from_multiaddr(addr: &Multiaddr) -> Option<PeerId> {
    use libp2p::multiaddr::Protocol;
    addr.iter().find_map(|p| {
        if let Protocol::P2p(peer_id) = p { Some(peer_id) } else { None }
    })
}

impl CiviumNode {
    /// Run the event loop — blocks until dropped. Spawn via `tokio::spawn`.
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // ── Swarm events ──────────────────────────────────────────────
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                }
                // ── Application commands ──────────────────────────────────────
                Some(cmd) = self.command_rx.recv() => {
                    self.handle_command(cmd).await;
                }
            }
        }
    }

    // ── Swarm event handler ───────────────────────────────────────────────────

    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<behaviour::CiviumBehaviourEvent>,
    ) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!(cid = %self.cid, addr = %address, "listening");
                self.listen_addrs.push(address.clone());
                let _ = self.event_tx.send(NodeEvent::Listening { addr: address }).await;
            }

            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                info!(%peer_id, addr = %endpoint.get_remote_address(), "connected");
                let _ = self.event_tx.send(NodeEvent::PeerConnected { peer_id }).await;
            }

            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                debug!(%peer_id, ?cause, "disconnected");
            }

            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                warn!(?peer_id, %error, "outgoing connection failed");
            }

            SwarmEvent::Behaviour(ev) => self.handle_behaviour_event(ev).await,

            _ => {}
        }
    }

    async fn handle_behaviour_event(&mut self, event: behaviour::CiviumBehaviourEvent) {
        match event {
            // ── Identify: add new addresses to Kademlia ───────────────────────
            behaviour::CiviumBehaviourEvent::Identify(identify::Event::Received {
                peer_id, info, ..
            }) => {
                for addr in info.listen_addrs {
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }

            // ── mDNS: local peer discovery ────────────────────────────────────
            behaviour::CiviumBehaviourEvent::Mdns(mdns::Event::Discovered(peers)) => {
                for (peer_id, addr) in peers {
                    info!(%peer_id, %addr, "mdns: discovered");
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                    let _ = self.swarm.dial(
                        DialOpts::peer_id(peer_id).addresses(vec![addr]).build()
                    );
                }
            }

            behaviour::CiviumBehaviourEvent::Mdns(mdns::Event::Expired(peers)) => {
                for (peer_id, addr) in peers {
                    self.swarm.behaviour_mut().kademlia.remove_address(&peer_id, &addr);
                }
            }

            // ── Kademlia: handle get_record results ───────────────────────────
            behaviour::CiviumBehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    result: kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(r))),
                    ..
                }
            ) => {
                if let Ok(key_str) = std::str::from_utf8(r.record.key.as_ref()) {
                    if let Some(cid_short) = key_str.strip_prefix("/civium/net/") {
                        if let Ok(addrs_json) = std::str::from_utf8(&r.record.value) {
                            if let Ok(addrs) = serde_json::from_str::<Vec<String>>(addrs_json) {
                                let peer_addrs: Vec<Multiaddr> = addrs
                                    .iter()
                                    .filter_map(|s| s.parse().ok())
                                    .collect();
                                let _ = self.event_tx.send(NodeEvent::PeersDiscovered {
                                    network_cid_short: cid_short.to_string(),
                                    peer_addrs,
                                }).await;
                            }
                        }
                    }
                }
            }

            // ── Request-response: inbound request ─────────────────────────────
            behaviour::CiviumBehaviourEvent::RequestResponse(
                request_response::Event::Message {
                    peer,
                    message: request_response::Message::Request { request_id, request, channel, .. },
                    ..
                }
            ) => {
                debug!(%peer, "inbound Civium request");
                self.pending_responses.insert(request_id, channel);
                let _ = self.event_tx.send(NodeEvent::InboundRequest {
                    from: peer,
                    request_id,
                    request,
                }).await;
            }

            // ── Request-response: outbound response ───────────────────────────
            behaviour::CiviumBehaviourEvent::RequestResponse(
                request_response::Event::Message {
                    message: request_response::Message::Response { request_id, response },
                    ..
                }
            ) => {
                let _ = self.event_tx.send(NodeEvent::OutboundResponse {
                    request_id,
                    response,
                }).await;
            }

            behaviour::CiviumBehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure { peer, error, .. }
            ) => {
                warn!(%peer, ?error, "request failed");
            }

            _ => {}
        }
    }

    // ── Command handler ───────────────────────────────────────────────────────

    async fn handle_command(&mut self, cmd: NodeCommand) {
        match cmd {
            NodeCommand::AnnounceNetwork { network_cid_short } => {
                let key = format!("/civium/net/{network_cid_short}");
                let addrs: Vec<String> = self.listen_addrs
                    .iter()
                    .map(|a| a.to_string())
                    .collect();
                if let Ok(value) = serde_json::to_vec(&addrs) {
                    let record = kad::Record::new(key.into_bytes(), value);
                    let _ = self.swarm.behaviour_mut().kademlia
                        .put_record(record, kad::Quorum::One);
                    info!(network = %network_cid_short, "announced to DHT");
                }
            }

            NodeCommand::DiscoverPeers { network_cid_short } => {
                let key = format!("/civium/net/{network_cid_short}");
                self.swarm.behaviour_mut().kademlia
                    .get_record(key.into_bytes().into());
            }

            NodeCommand::SendRequest { peer, request } => {
                self.swarm.behaviour_mut().request_response
                    .send_request(&peer, request);
            }

            NodeCommand::Respond { request_id, response } => {
                if let Some(channel) = self.pending_responses.remove(&request_id) {
                    let _ = self.swarm.behaviour_mut().request_response
                        .send_response(channel, response);
                } else {
                    warn!(?request_id, "no pending channel for Respond command");
                }
            }

            NodeCommand::Dial { addr } => {
                let _ = self.swarm.dial(
                    DialOpts::unknown_peer_id().address(addr).build()
                );
            }
        }
    }
}
