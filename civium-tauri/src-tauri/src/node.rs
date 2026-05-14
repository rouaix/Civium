//! P2P node lifecycle for the Tauri app.
//!
//! `start_node` is called once at startup (inside a background task) if an identity
//! exists.  It runs the libp2p event loop, dispatches NodeEvents to the app, and
//! periodically triggers DHT peer-discovery + state sync for every local network.

use std::path::PathBuf;
use std::sync::Mutex;

use civium_core::{
    node::{CiviumNode, NodeCommand, NodeConfig, NodeEvent, NodeHandle},
    CiviumKeypair, CiviumRequest, CiviumResponse,
};
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{interval, Duration};

use crate::store;

// ── Shared state ──────────────────────────────────────────────────────────────

pub struct AppState {
    /// Sender half of the node's command channel — None if the node isn't running.
    pub node_tx: Mutex<Option<tokio::sync::mpsc::Sender<NodeCommand>>>,
    /// Multiaddrs the node is currently listening on.
    pub listen_addrs: Mutex<Vec<String>>,
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Start the P2P node in the background.  Never returns while the node is alive;
/// call from inside `tauri::async_runtime::spawn`.
pub async fn start_node(app_handle: AppHandle, keypair: CiviumKeypair, data_dir: PathBuf) {
    let config = NodeConfig::default();
    let (node, handle) = match CiviumNode::new(keypair, config).await {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("[civium] P2P node start failed: {e}");
            return;
        }
    };

    // Store the command sender so Tauri commands and the periodic task can reach the node.
    {
        let state = app_handle.state::<AppState>();
        *state.node_tx.lock().unwrap() = Some(handle.commands.clone());
    }

    // Clone command sender for the event loop before handle is moved into it.
    let cmd_tx_event = handle.commands.clone();

    // Spawn the event-processing loop (consumes the NodeHandle).
    let app2 = app_handle.clone();
    let data2 = data_dir.clone();
    tauri::async_runtime::spawn(async move {
        run_event_loop(handle, cmd_tx_event, app2, data2).await;
    });

    // Spawn the periodic sync ticker (reads cmd_tx from AppState every 60 s).
    let app3 = app_handle.clone();
    let data3 = data_dir.clone();
    tauri::async_runtime::spawn(async move {
        run_periodic_sync(app3, data3).await;
    });

    node.run().await;
}

// ── Event loop ────────────────────────────────────────────────────────────────

async fn run_event_loop(
    mut handle: NodeHandle,
    cmd_tx: tokio::sync::mpsc::Sender<NodeCommand>,
    app_handle: AppHandle,
    data_dir: PathBuf,
) {
    // Networks we want to sync on next peer connection: (cid_short, cid_full).
    let mut pending_sync: Vec<(String, String)> = Vec::new();

    while let Some(event) = handle.events.recv().await {
        match event {
            NodeEvent::Listening { addr } => {
                eprintln!("[civium] Listening on {addr}");
                app_handle
                    .state::<AppState>()
                    .listen_addrs
                    .lock()
                    .unwrap()
                    .push(addr.to_string());

                // Auto-announce every local network to the DHT.
                if let Ok(conn) = store::open_db(&data_dir) {
                    if let Ok(networks) = store::list_networks(&conn) {
                        for net in networks {
                            let _ = cmd_tx
                                .send(NodeCommand::AnnounceNetwork {
                                    network_cid_short: net.cid_short().to_string(),
                                })
                                .await;
                        }
                    }
                }
            }

            NodeEvent::PeersDiscovered { network_cid_short, peer_addrs } => {
                if peer_addrs.is_empty() {
                    continue;
                }
                // Queue a sync for this network once a peer connects.
                if let Ok(conn) = store::open_db(&data_dir) {
                    if let Ok(net) = store::load_network(&conn, &network_cid_short) {
                        let cid_full = net.cid_full().to_string();
                        if !pending_sync.iter().any(|(s, _)| s == &network_cid_short) {
                            pending_sync.push((network_cid_short, cid_full));
                        }
                    }
                }
                for addr in peer_addrs {
                    let _ = cmd_tx.send(NodeCommand::Dial { addr }).await;
                }
            }

            NodeEvent::PeerConnected { peer_id } => {
                // Send a SyncRequest for every pending network to this peer.
                for (_, cid_full) in &pending_sync {
                    let _ = cmd_tx
                        .send(NodeCommand::SendRequest {
                            peer: peer_id,
                            request: CiviumRequest::Sync {
                                network_cid_full: cid_full.clone(),
                                since_ts: 0,
                            },
                        })
                        .await;
                }
            }

            NodeEvent::InboundRequest { request_id, request, .. } => {
                let response = handle_inbound(&data_dir, &request);
                let _ = cmd_tx
                    .send(NodeCommand::Respond { request_id, response })
                    .await;
            }

            NodeEvent::OutboundResponse { response, .. } => {
                if let CiviumResponse::SyncData { network_cid_full, members, messages } = response {
                    if let Ok(conn) = store::open_db(&data_dir) {
                        if let Ok(net) = store::find_network_by_full_cid(&conn, &network_cid_full) {
                            let cid_short = net.cid_short().to_string();
                            drop(conn);
                            if let Ok(conn2) = store::open_db(&data_dir) {
                                if let Err(e) =
                                    store::merge_sync_data(&conn2, &cid_short, members, messages)
                                {
                                    eprintln!("[civium] Sync merge error: {e}");
                                } else {
                                    eprintln!("[civium] Synced network {cid_short}");
                                    let _ = app_handle.emit("civium://sync-completed", &cid_short);
                                }
                            }
                            pending_sync.retain(|(s, _)| s != &cid_short);
                        }
                    }
                }
            }
        }
    }
}

// ── Periodic sync ─────────────────────────────────────────────────────────────

async fn run_periodic_sync(app_handle: AppHandle, data_dir: PathBuf) {
    let mut ticker = interval(Duration::from_secs(60));
    ticker.tick().await; // skip the immediate first tick
    loop {
        ticker.tick().await;
        let cmd_tx = app_handle
            .state::<AppState>()
            .node_tx
            .lock()
            .unwrap()
            .clone();
        let Some(cmd_tx) = cmd_tx else { continue };

        if let Ok(conn) = store::open_db(&data_dir) {
            if let Ok(networks) = store::list_networks(&conn) {
                for net in networks {
                    let _ = cmd_tx
                        .send(NodeCommand::DiscoverPeers {
                            network_cid_short: net.cid_short().to_string(),
                        })
                        .await;
                }
            }
        }
    }
}

// ── Inbound request handler ───────────────────────────────────────────────────

fn handle_inbound(data_dir: &PathBuf, request: &CiviumRequest) -> CiviumResponse {
    match request {
        CiviumRequest::Ping => CiviumResponse::Pong,

        CiviumRequest::Sync { network_cid_full, since_ts } => {
            let result = (|| -> anyhow::Result<CiviumResponse> {
                let conn = store::open_db(data_dir)?;
                let net = store::find_network_by_full_cid(&conn, network_cid_full)?;
                let messages = store::load_messages(&conn, net.cid_short())?;

                let members = net
                    .data
                    .members
                    .into_iter()
                    .filter(|m| m.joined_at >= *since_ts)
                    .collect();
                let messages = messages
                    .into_iter()
                    .filter(|m| m.sent_at >= *since_ts)
                    .collect();

                Ok(CiviumResponse::SyncData {
                    network_cid_full: network_cid_full.clone(),
                    members,
                    messages,
                })
            })();
            result.unwrap_or_else(|e| CiviumResponse::Error { message: e.to_string() })
        }

        CiviumRequest::Join { .. } => CiviumResponse::JoinRejected {
            reason: "Use the Civium app to accept join requests".into(),
        },
    }
}
