mod store;

use anyhow::{bail, Result};
use civium_core::{
    connection::ShareAgreement,
    network::{Invitation, Network},
    CiviumKeypair, CiviumNode, ConnectionRecord, ConnectionState, GroupKey, MemberRole,
    MessageKind, NodeConfig, ShareTerms, TrustCircle,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

// ── CLI structure ─────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "civium", about = "Civium protocol CLI — Phase 0")]
struct Cli {
    /// Directory used to store node state (identity, networks).
    #[arg(long, global = true, default_value = "./civium-data", env = "CIVIUM_DATA")]
    data_dir: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage local identity (keypair / CID)
    Identity {
        #[command(subcommand)]
        action: IdentityCmd,
    },
    /// Manage Civium networks
    Network {
        #[command(subcommand)]
        action: NetworkCmd,
    },
    /// Manage members of a network
    Member {
        #[command(subcommand)]
        action: MemberCmd,
    },
    /// Manage the local P2P node
    Node {
        #[command(subcommand)]
        action: NodeCmd,
    },
    /// Send and read encrypted messages
    Msg {
        #[command(subcommand)]
        action: MsgCmd,
    },
    /// Manage connections between Civium networks
    Connect {
        #[command(subcommand)]
        action: ConnectCmd,
    },
}

// ── Identity sub-commands ─────────────────────────────────────────────────────

#[derive(Subcommand)]
enum IdentityCmd {
    /// Generate a new identity and save it to the data directory.
    Init {
        #[arg(long, env = "CIVIUM_SECRET", help = "Restore from an existing secret key (base58)")]
        secret: Option<String>,
    },
    /// Show the CID of the current identity.
    Show,
}

// ── Network sub-commands ──────────────────────────────────────────────────────

#[derive(Subcommand)]
enum NetworkCmd {
    /// Create a new network (generates a fresh network keypair).
    Create {
        #[arg(long)]
        name: String,
        /// Display name for the founding admin in this network.
        #[arg(long, default_value = "admin")]
        display_name: String,
    },
    /// List networks stored locally.
    List,
    /// Show details of a network.
    Info {
        /// Network CID short (e.g. civ1AbCdEfGh).
        network_cid: String,
    },
    /// Generate a signed invitation link for a network.
    Invite {
        /// Network CID short.
        network_cid: String,
        /// Expiry in hours (0 = no expiry).
        #[arg(long, default_value = "0")]
        expires_in: u64,
    },
}

// ── Member sub-commands ───────────────────────────────────────────────────────

#[derive(Subcommand)]
enum MemberCmd {
    /// Submit a join request using an invitation link.
    Join {
        #[arg(long)]
        invite: String,
        /// Your display name in this network (must be unique).
        #[arg(long)]
        name: String,
    },
    /// List members (and pending requests) of a network.
    List {
        network_cid: String,
        /// Also show members of active connected networks (if their APC permits).
        #[arg(long, default_value = "false")]
        connected: bool,
    },
    /// Admit a pending member (admin only).
    Admit {
        network_cid: String,
        member_cid: String,
        /// Trust circle: 0 (annuaire), 1 (connaissance), 2 (confiance).
        #[arg(long, default_value = "1")]
        circle: u8,
        /// Role: member or admin.
        #[arg(long, default_value = "member")]
        role: String,
    },
    /// Reject a pending join request (admin only).
    Reject {
        network_cid: String,
        member_cid: String,
    },
}

// ── Msg sub-commands ──────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum MsgCmd {
    /// Post a message to the network thread or directly to a member.
    Send {
        /// Network CID (short or prefix).
        #[arg(long)]
        network: String,
        /// Recipient CID short — omit for a thread message.
        #[arg(long)]
        to: Option<String>,
        /// Plaintext body (encrypted with the network group key before storage).
        #[arg(long)]
        body: String,
    },
    /// Read messages from the network thread or a direct conversation.
    List {
        /// Network CID (short or prefix).
        #[arg(long)]
        network: String,
        /// Show only direct messages with this member CID short.
        #[arg(long)]
        with: Option<String>,
    },
}

// ── Connect sub-commands ──────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ConnectCmd {
    /// Request a connection with another network (Phase 0: both in same data dir).
    Request {
        /// Our network CID (short or prefix).
        #[arg(long)]
        network: String,
        /// Peer network CID (short or prefix).
        #[arg(long)]
        to: String,
        /// Do not expose our member directory to the peer.
        #[arg(long, default_value = "false")]
        no_directory: bool,
    },
    /// Accept a pending connection request (admin only).
    Accept {
        #[arg(long)]
        network: String,
        /// Peer network CID short.
        #[arg(long)]
        peer: String,
        /// Do not expose our member directory to the peer.
        #[arg(long, default_value = "false")]
        no_directory: bool,
    },
    /// Refuse a pending connection request (admin only).
    Refuse {
        #[arg(long)]
        network: String,
        #[arg(long)]
        peer: String,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Block a network (refuse + prevent future requests).
    Block {
        #[arg(long)]
        network: String,
        #[arg(long)]
        peer: String,
    },
    /// Revoke an active connection.
    Revoke {
        #[arg(long)]
        network: String,
        #[arg(long)]
        peer: String,
    },
    /// List all connections of a network.
    List {
        #[arg(long)]
        network: String,
    },
    /// Show details of a specific connection (including APC terms).
    Info {
        #[arg(long)]
        network: String,
        #[arg(long)]
        peer: String,
    },
}

// ── Node sub-commands ─────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum NodeCmd {
    /// Start the local P2P node.
    Start {
        #[arg(long, default_value = "/ip4/0.0.0.0/tcp/0")]
        listen_tcp: String,
        #[arg(long, default_value = "/ip4/0.0.0.0/udp/0/quic-v1")]
        listen_quic: String,
        #[arg(long = "peer")]
        peers: Vec<String>,
    },
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();
    let data = &cli.data_dir;

    match cli.command {
        Command::Identity { action } => run_identity(action, data),
        Command::Network { action } => run_network(action, data),
        Command::Member { action } => run_member(action, data),
        Command::Node { action } => run_node(action, data).await,
        Command::Msg { action } => run_msg(action, data),
        Command::Connect { action } => run_connect(action, data),
    }
}

// ── Identity handlers ─────────────────────────────────────────────────────────

fn run_identity(cmd: IdentityCmd, data: &PathBuf) -> Result<()> {
    match cmd {
        IdentityCmd::Init { secret } => {
            if store::identity_exists(data) {
                bail!("identity already exists at {}. Use `identity show`.", data.display());
            }
            let keypair = match secret {
                Some(s) => CiviumKeypair::from_secret_b58(&s)
                    .map_err(|e| anyhow::anyhow!("{e}"))?,
                None => CiviumKeypair::generate()
                    .map_err(|e| anyhow::anyhow!("{e}"))?,
            };
            let cid = keypair.cid();
            store::save_identity(data, &keypair)?;
            println!("Identity created and saved.");
            println!("  CID (short) : {}", cid.short());
            println!("  CID (full)  : {}", cid.full());
            println!("  Secret key  : {}", keypair.secret_b58());
            println!();
            println!("Back up your secret key — it cannot be recovered if lost.");
        }
        IdentityCmd::Show => {
            let keypair = store::load_identity(data)?;
            let cid = keypair.cid();
            println!("CID (short) : {}", cid.short());
            println!("CID (full)  : {}", cid.full());
        }
    }
    Ok(())
}

// ── Network handlers ──────────────────────────────────────────────────────────

fn run_network(cmd: NetworkCmd, data: &PathBuf) -> Result<()> {
    match cmd {
        NetworkCmd::Create { name, display_name } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let admin_cid = keypair.cid();
            let network = Network::create(name.clone(), &admin_cid, display_name)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            store::save_network(data, &network)?;
            println!("Network '{}' created.", name);
            println!("  Network CID (short) : {}", network.cid_short());
            println!("  Network CID (full)  : {}", network.cid_full());
            let addr = network.address_for(&admin_cid);
            println!("  Your address        : {addr}");
        }

        NetworkCmd::List => {
            let cids = store::list_network_cids(data);
            if cids.is_empty() {
                println!("No networks found. Create one with `network create --name <name>`.");
                return Ok(());
            }
            for cid_short in cids {
                if let Ok(n) = store::load_network(data, &cid_short) {
                    println!("{} — {} ({} members)", n.cid_short(), n.name(), n.data.members.len());
                }
            }
        }

        NetworkCmd::Info { network_cid } => {
            let network = load_network_fuzzy(data, &network_cid)?;
            println!("Network    : {}", network.name());
            println!("CID short  : {}", network.cid_short());
            println!("CID full   : {}", network.cid_full());
            println!("Members    : {}", network.data.members.len());
            println!("Pending    : {}", network.data.pending.len());
            for m in &network.data.members {
                println!("  {} ({}) — {} [{}]", m.display_name, m.cid_short, m.circle, m.role);
            }
        }

        NetworkCmd::Invite { network_cid, expires_in } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let inviter_cid = keypair.cid();
            let network = load_network_fuzzy(data, &network_cid)?;
            let link = network
                .create_invitation(&inviter_cid, expires_in)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Invitation link for '{}':", network.name());
            println!();
            println!("{link}");
            println!();
            if expires_in == 0 {
                println!("This invitation does not expire.");
            } else {
                println!("Expires in {expires_in}h.");
            }
        }
    }
    Ok(())
}

// ── Member handlers ───────────────────────────────────────────────────────────

fn run_member(cmd: MemberCmd, data: &PathBuf) -> Result<()> {
    match cmd {
        MemberCmd::Join { invite, name } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let member_cid = keypair.cid();

            let invitation = Invitation::from_link(&invite)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            invitation.verify().map_err(|e| anyhow::anyhow!("{e}"))?;

            let network_cid_full = invitation.network_cid_full().to_string();
            let cid_short = &network_cid_full[..12.min(network_cid_full.len())];

            // Load existing network record if we already have it, otherwise create a stub
            let mut network = if let Ok(n) = load_network_fuzzy(data, cid_short) {
                n
            } else {
                bail!(
                    "Network '{}' ({}) is not in your data directory.\n\
                     In Phase 0, admin and member share the same data directory.\n\
                     Inter-network P2P join is implemented in weeks 7-8.",
                    invitation.network_name(),
                    cid_short
                );
            };

            network
                .submit_join_request(&member_cid, name.clone(), &invitation)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            store::save_network(data, &network)?;
            println!(
                "Join request submitted to '{}' as '{name}'.",
                network.name()
            );
            println!("Ask an admin to run: civium member admit {} {}", network.cid_short(), member_cid.short());
        }

        MemberCmd::List { network_cid, connected } => {
            let network = load_network_fuzzy(data, &network_cid)?;
            println!("=== {} ===", network.name());
            println!();
            println!("Members ({}):", network.data.members.len());
            for m in &network.data.members {
                println!("  {} — {} ({}) [{}]", m.display_name, m.cid_short, m.circle, m.role);
            }
            if !network.data.pending.is_empty() {
                println!();
                println!("Pending ({}):", network.data.pending.len());
                for p in &network.data.pending {
                    println!("  {} — {} (waiting)", p.display_name, p.cid_short);
                }
            }
            if connected {
                let conn_store = store::load_connections(data, network.cid_short())?;
                let active: Vec<_> = conn_store
                    .connections
                    .iter()
                    .filter(|c| {
                        c.state == ConnectionState::Active
                            && c.their_terms
                                .as_ref()
                                .map(|t| t.expose_member_directory)
                                .unwrap_or(false)
                    })
                    .collect();

                if active.is_empty() {
                    println!();
                    println!("No connected networks sharing their directory.");
                } else {
                    for conn in active {
                        if let Ok(peer_net) = load_network_fuzzy(data, &conn.peer_cid_short) {
                            println!();
                            println!("From {} (connected):", peer_net.name());
                            for m in &peer_net.data.members {
                                println!(
                                    "  {} — {}@{} ({}) [{}]",
                                    m.display_name,
                                    m.cid_short,
                                    peer_net.cid_short(),
                                    m.circle,
                                    m.role
                                );
                            }
                        }
                    }
                }
            }
        }

        MemberCmd::Admit { network_cid, member_cid, circle, role } => {
            let circle = TrustCircle::from_u8(circle)
                .ok_or_else(|| anyhow::anyhow!("invalid circle {circle} — use 0, 1, or 2"))?;
            let role: MemberRole = role.parse().map_err(|e: String| anyhow::anyhow!("{e}"))?;

            let mut network = load_network_fuzzy(data, &network_cid)?;
            let record = network
                .admit(&member_cid, circle, role)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            store::save_network(data, &network)?;
            println!(
                "Admitted {} as '{}' — circle: {}, role: {}",
                record.cid_short, record.display_name, record.circle, record.role
            );
            println!("Network address: {}@{}", record.cid_short, network.cid_short());
        }

        MemberCmd::Reject { network_cid, member_cid } => {
            let mut network = load_network_fuzzy(data, &network_cid)?;
            network.reject(&member_cid).map_err(|e| anyhow::anyhow!("{e}"))?;
            store::save_network(data, &network)?;
            println!("Join request from {member_cid} rejected.");
        }
    }
    Ok(())
}

// ── Connect handler ───────────────────────────────────────────────────────────

fn run_connect(cmd: ConnectCmd, data: &PathBuf) -> Result<()> {
    match cmd {
        // ── Request ───────────────────────────────────────────────────────────
        ConnectCmd::Request { network, to, no_directory } => {
            let our_net = load_network_fuzzy(data, &network)?;
            let peer_net = load_network_fuzzy(data, &to).map_err(|_| {
                anyhow::anyhow!(
                    "network '{to}' not found in {}.\n\
                     In Phase 0, both networks must share the same data directory.\n\
                     P2P handshake is implemented in weeks 9-10.",
                    data.display()
                )
            })?;

            // Guard: no self-connections
            if our_net.cid_full() == peer_net.cid_full() {
                bail!("a network cannot connect to itself");
            }

            let our_terms = ShareTerms { expose_member_directory: !no_directory };
            let peer_pubkey_b58 = peer_net.pubkey_b58();

            // Build and sign the connection request
            let signed_req = ShareAgreement::build_request(
                our_net.keypair(),
                our_net.name(),
                our_terms.clone(),
                peer_net.cid_full(),
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;

            let now = unix_now_cli();

            // Our record: Requested
            let our_record = ConnectionRecord {
                peer_cid_full:    peer_net.cid_full().to_string(),
                peer_cid_short:   peer_net.cid_short().to_string(),
                peer_name:        peer_net.name().to_string(),
                peer_pubkey_b58:  peer_pubkey_b58,
                state:            ConnectionState::Requested,
                initiated_at:     now,
                updated_at:       now,
                our_terms,
                their_terms:      None,
                incoming_request: None,
                apc:              None,
            };

            // Peer record: Validating (holds our signed request)
            let peer_our_pubkey_b58 = our_net.pubkey_b58();
            let peer_record = ConnectionRecord {
                peer_cid_full:    our_net.cid_full().to_string(),
                peer_cid_short:   our_net.cid_short().to_string(),
                peer_name:        our_net.name().to_string(),
                peer_pubkey_b58:  peer_our_pubkey_b58,
                state:            ConnectionState::Validating,
                initiated_at:     now,
                updated_at:       now,
                our_terms:        ShareTerms::default(), // peer admin sets at accept
                their_terms:      Some(signed_req.payload.from_terms.clone()),
                incoming_request: Some(signed_req),
                apc:              None,
            };

            // Persist
            let mut our_conns = store::load_connections(data, our_net.cid_short())?;
            guard_no_duplicate(&our_conns, peer_net.cid_full())?;
            our_conns.connections.push(our_record);
            store::save_connections(data, our_net.cid_short(), &our_conns)?;

            let mut peer_conns = store::load_connections(data, peer_net.cid_short())?;
            peer_conns.connections.push(peer_record);
            store::save_connections(data, peer_net.cid_short(), &peer_conns)?;

            println!(
                "Connection request sent from '{}' to '{}'.",
                our_net.name(),
                peer_net.name()
            );
            println!(
                "Ask an admin of '{}' to run:",
                peer_net.name()
            );
            println!(
                "  civium connect accept --network {} --peer {}",
                peer_net.cid_short(),
                our_net.cid_short()
            );
        }

        // ── Accept ────────────────────────────────────────────────────────────
        ConnectCmd::Accept { network, peer, no_directory } => {
            let our_net = load_network_fuzzy(data, &network)?;
            let peer_net = load_network_fuzzy(data, &peer).map_err(|_| {
                anyhow::anyhow!("network '{peer}' not found — both must share the same data dir")
            })?;

            let our_terms = ShareTerms { expose_member_directory: !no_directory };

            // Load our (B's) connection record
            let mut our_conns = store::load_connections(data, our_net.cid_short())?;
            let b_rec = find_conn_mut(&mut our_conns.connections, peer_net.cid_full())
                .ok_or_else(|| anyhow::anyhow!("no connection request from '{peer}'"))?;

            if b_rec.state != ConnectionState::Validating {
                bail!(
                    "connection with '{}' is in state '{}', expected Validating",
                    peer_net.name(),
                    b_rec.state
                );
            }

            let signed_req = b_rec
                .incoming_request
                .clone()
                .ok_or_else(|| anyhow::anyhow!("missing request payload in connection record"))?;

            // Build the APC (verifies A's sig, B signs acceptance)
            let apc = ShareAgreement::build_from_acceptance(
                &signed_req,
                our_net.keypair(),
                our_net.name(),
                our_terms.clone(),
            )
            .map_err(|e| anyhow::anyhow!("APC error: {e}"))?;

            apc.verify().map_err(|e| anyhow::anyhow!("{e}"))?;

            let now = unix_now_cli();

            // Update B's record → Active
            b_rec.state = ConnectionState::Active;
            b_rec.our_terms = our_terms.clone();
            b_rec.their_terms = Some(apc.request.from_terms.clone());
            b_rec.incoming_request = None;
            b_rec.apc = Some(apc.clone());
            b_rec.updated_at = now;
            store::save_connections(data, our_net.cid_short(), &our_conns)?;

            // Update A's record → Active
            let mut peer_conns = store::load_connections(data, peer_net.cid_short())?;
            let a_rec = find_conn_mut(&mut peer_conns.connections, our_net.cid_full())
                .ok_or_else(|| anyhow::anyhow!("no pending record on requester side"))?;
            a_rec.state = ConnectionState::Active;
            a_rec.their_terms = Some(our_terms);
            a_rec.apc = Some(apc);
            a_rec.updated_at = now;
            store::save_connections(data, peer_net.cid_short(), &peer_conns)?;

            println!(
                "Connection between '{}' and '{}' is now Active.",
                our_net.name(),
                peer_net.name()
            );
            println!("APC signed by both networks.");
        }

        // ── Refuse ────────────────────────────────────────────────────────────
        ConnectCmd::Refuse { network, peer, reason } => {
            let our_net = load_network_fuzzy(data, &network)?;
            let peer_net = load_network_fuzzy(data, &peer).map_err(|_| {
                anyhow::anyhow!("network '{peer}' not found")
            })?;

            let now = unix_now_cli();

            let mut our_conns = store::load_connections(data, our_net.cid_short())?;
            let b_rec = find_conn_mut(&mut our_conns.connections, peer_net.cid_full())
                .ok_or_else(|| anyhow::anyhow!("no connection request from '{peer}'"))?;
            b_rec.state = ConnectionState::Refused { reason: reason.clone() };
            b_rec.incoming_request = None;
            b_rec.updated_at = now;
            store::save_connections(data, our_net.cid_short(), &our_conns)?;

            let mut peer_conns = store::load_connections(data, peer_net.cid_short())?;
            if let Some(a_rec) = find_conn_mut(&mut peer_conns.connections, our_net.cid_full()) {
                a_rec.state = ConnectionState::Refused { reason: reason.clone() };
                a_rec.updated_at = now;
                store::save_connections(data, peer_net.cid_short(), &peer_conns)?;
            }

            println!(
                "Connection request from '{}' refused.",
                peer_net.name()
            );
            if let Some(r) = reason {
                println!("Reason: {r}");
            }
        }

        // ── Block ─────────────────────────────────────────────────────────────
        ConnectCmd::Block { network, peer } => {
            let our_net = load_network_fuzzy(data, &network)?;
            let peer_net = load_network_fuzzy(data, &peer).map_err(|_| {
                anyhow::anyhow!("network '{peer}' not found")
            })?;

            let now = unix_now_cli();

            let mut our_conns = store::load_connections(data, our_net.cid_short())?;
            let rec = find_conn_mut(&mut our_conns.connections, peer_net.cid_full())
                .ok_or_else(|| anyhow::anyhow!("no connection record for '{peer}'"))?;
            rec.state = ConnectionState::Blocked;
            rec.incoming_request = None;
            rec.apc = None;
            rec.updated_at = now;
            store::save_connections(data, our_net.cid_short(), &our_conns)?;

            // Peer sees Refused — they don't learn they were blocked
            let mut peer_conns = store::load_connections(data, peer_net.cid_short())?;
            if let Some(peer_rec) = find_conn_mut(&mut peer_conns.connections, our_net.cid_full()) {
                peer_rec.state = ConnectionState::Refused { reason: None };
                peer_rec.updated_at = now;
                store::save_connections(data, peer_net.cid_short(), &peer_conns)?;
            }

            println!("'{}' is now blocked.", peer_net.name());
        }

        // ── Revoke ────────────────────────────────────────────────────────────
        ConnectCmd::Revoke { network, peer } => {
            let our_net = load_network_fuzzy(data, &network)?;
            let peer_net = load_network_fuzzy(data, &peer).map_err(|_| {
                anyhow::anyhow!("network '{peer}' not found")
            })?;

            let now = unix_now_cli();

            let mut our_conns = store::load_connections(data, our_net.cid_short())?;
            let rec = find_conn_mut(&mut our_conns.connections, peer_net.cid_full())
                .ok_or_else(|| anyhow::anyhow!("no connection with '{peer}'"))?;
            if rec.state != ConnectionState::Active {
                bail!("connection with '{}' is not Active (current: {})", peer_net.name(), rec.state);
            }
            rec.state = ConnectionState::Revoked;
            rec.apc = None;
            rec.updated_at = now;
            store::save_connections(data, our_net.cid_short(), &our_conns)?;

            let mut peer_conns = store::load_connections(data, peer_net.cid_short())?;
            if let Some(peer_rec) = find_conn_mut(&mut peer_conns.connections, our_net.cid_full()) {
                peer_rec.state = ConnectionState::Revoked;
                peer_rec.apc = None;
                peer_rec.updated_at = now;
                store::save_connections(data, peer_net.cid_short(), &peer_conns)?;
            }

            println!("Connection with '{}' revoked.", peer_net.name());
        }

        // ── List ──────────────────────────────────────────────────────────────
        ConnectCmd::List { network } => {
            let net = load_network_fuzzy(data, &network)?;
            let store = store::load_connections(data, net.cid_short())?;

            if store.connections.is_empty() {
                println!("No connections for '{}'.", net.name());
                return Ok(());
            }

            println!("=== Connections: {} ===", net.name());
            println!();
            for c in &store.connections {
                println!(
                    "  {} ({}) — {}",
                    c.peer_name,
                    c.peer_cid_short,
                    c.state
                );
            }
        }

        // ── Info ──────────────────────────────────────────────────────────────
        ConnectCmd::Info { network, peer } => {
            let net = load_network_fuzzy(data, &network)?;
            let store = store::load_connections(data, net.cid_short())?;

            let conn = store
                .connections
                .iter()
                .find(|c| c.peer_cid_short.starts_with(&peer) || c.peer_cid_full.starts_with(&peer))
                .ok_or_else(|| anyhow::anyhow!("no connection record for '{peer}'"))?;

            println!("Connection: {} → {}", net.name(), conn.peer_name);
            println!("  Peer CID  : {}", conn.peer_cid_short);
            println!("  State     : {}", conn.state);
            println!("  Since     : {}", fmt_ts(conn.initiated_at));
            println!("  Our terms : directory={}", conn.our_terms.expose_member_directory);
            if let Some(t) = &conn.their_terms {
                println!("  Their terms: directory={}", t.expose_member_directory);
            }
            if let Some(apc) = &conn.apc {
                println!();
                println!("  APC — Accord de Partage Civium");
                println!("    Nonce       : {}", apc.request.nonce_b58);
                println!("    Created     : {}", fmt_ts(apc.request.created_at));
                println!("    Accepted    : {}", fmt_ts(apc.acceptance.accepted_at));
                println!("    Requester   : {} ({})", apc.request.from_name, &apc.request.from_cid_full[..12]);
                println!("    Acceptor    : {} ({})", apc.acceptance.from_name, &apc.acceptance.from_cid_full[..12]);
                match apc.verify() {
                    Ok(()) => println!("    Signatures  : valid"),
                    Err(e) => println!("    Signatures  : INVALID — {e}"),
                }
            }
        }
    }
    Ok(())
}

fn unix_now_cli() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn guard_no_duplicate(store: &store::ConnectionStore, peer_cid_full: &str) -> Result<()> {
    if store.connections.iter().any(|c| c.peer_cid_full == peer_cid_full) {
        bail!("a connection record for this network already exists");
    }
    Ok(())
}

fn find_conn_mut<'a>(
    conns: &'a mut Vec<ConnectionRecord>,
    peer_cid_full: &str,
) -> Option<&'a mut ConnectionRecord> {
    conns.iter_mut().find(|c| c.peer_cid_full == peer_cid_full)
}

// ── Node handler ──────────────────────────────────────────────────────────────

async fn run_node(cmd: NodeCmd, data: &PathBuf) -> Result<()> {
    match cmd {
        NodeCmd::Start { listen_tcp, listen_quic, peers } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let cid = keypair.cid();

            let config = NodeConfig { listen_tcp, listen_quic, bootstrap_peers: peers };

            println!("Starting Civium node");
            println!("  CID        : {}", cid.short());
            println!("  CID (full) : {}", cid.full());

            // Show network addresses
            for cid_short in store::list_network_cids(data) {
                if let Ok(n) = store::load_network(data, &cid_short) {
                    let addr = n.address_for(&cid);
                    println!("  Network    : {} → {addr}", n.name());
                }
            }

            let mut node = CiviumNode::new(keypair, config).await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            node.run().await;
        }
    }
    Ok(())
}

// ── Msg handlers ─────────────────────────────────────────────────────────────

fn run_msg(cmd: MsgCmd, data: &PathBuf) -> Result<()> {
    match cmd {
        MsgCmd::Send { network, to, body } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let author_cid = keypair.cid();

            let net = load_network_fuzzy(data, &network)?;
            let group_key = GroupKey::from_b58(&net.data.group_key_b58)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            // Check that the author is a member of this network
            if !net.data.members.iter().any(|m| m.cid_full == author_cid.full()) {
                bail!("you are not a member of '{}'", net.name());
            }

            let kind = match to {
                Some(ref peer) => {
                    // Validate peer exists in the network
                    if !net.data.members.iter().any(|m| m.cid_short.starts_with(peer.as_str())) {
                        bail!("member '{peer}' not found in '{}'", net.name());
                    }
                    // Resolve to full cid_short
                    let peer_cid_short = net
                        .data
                        .members
                        .iter()
                        .find(|m| m.cid_short.starts_with(peer.as_str()))
                        .map(|m| m.cid_short.clone())
                        .unwrap();
                    MessageKind::Direct { to_cid_short: peer_cid_short }
                }
                None => MessageKind::Thread,
            };

            let mut mailbox = store::load_mailbox(data, net.cid_short())?;
            mailbox
                .post(author_cid.short().to_string(), kind.clone(), &body, &group_key)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            // Grab the id before the save (last message is what we just posted)
            let msg_id = mailbox.messages.last().unwrap().id.clone();
            store::save_mailbox(data, net.cid_short(), &mailbox)?;

            let label = match kind {
                MessageKind::Thread => "thread".to_string(),
                MessageKind::Direct { to_cid_short } => format!("DM → {to_cid_short}"),
            };
            println!("Message sent ({label}) — id: {}", &msg_id[..8.min(msg_id.len())]);
        }

        MsgCmd::List { network, with } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let local_cid = keypair.cid();

            let net = load_network_fuzzy(data, &network)?;
            let group_key = GroupKey::from_b58(&net.data.group_key_b58)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            let mailbox = store::load_mailbox(data, net.cid_short())?;

            if mailbox.messages.is_empty() {
                println!("No messages in '{}'.", net.name());
                return Ok(());
            }

            let messages_to_show: Vec<_> = match &with {
                Some(peer) => {
                    mailbox
                        .direct_messages(local_cid.short(), peer)
                        .collect()
                }
                None => mailbox.thread_messages().collect(),
            };

            if messages_to_show.is_empty() {
                if let Some(peer) = &with {
                    println!("No direct messages with {peer} in '{}'.", net.name());
                } else {
                    println!("No thread messages in '{}'.", net.name());
                }
                return Ok(());
            }

            let header = match &with {
                Some(peer) => format!("=== DM: {} ↔ {} ===", local_cid.short(), peer),
                None => format!("=== {} — thread ===", net.name()),
            };
            println!("{header}");
            println!();

            for msg in messages_to_show {
                let body = mailbox
                    .decrypt_body(msg, &group_key)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                let author_name = net
                    .data
                    .members
                    .iter()
                    .find(|m| m.cid_short == msg.author_cid_short)
                    .map(|m| m.display_name.as_str())
                    .unwrap_or(&msg.author_cid_short);
                println!("[{}] {} : {}", fmt_ts(msg.sent_at), author_name, body);
            }
        }
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmt_ts(unix: u64) -> String {
    // Simple formatting without external deps: YYYY-MM-DD HH:MM
    use std::time::{Duration, UNIX_EPOCH};
    let d = UNIX_EPOCH + Duration::from_secs(unix);
    // Approximate: good enough for CLI display in Phase 0.
    let secs = unix;
    let mins  = secs / 60;
    let hours = mins / 60;
    let days  = hours / 24;
    let year  = 1970 + days / 365;
    let rem   = days % 365;
    let month = rem / 30 + 1;
    let day   = rem % 30 + 1;
    let _ = d; // suppress unused warning
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        year, month.min(12), day.min(31),
        hours % 24, mins % 60
    )
}

/// Load a network by CID prefix — accepts short form or full CID.
fn load_network_fuzzy(data: &PathBuf, cid: &str) -> Result<Network> {
    // Direct match first
    if let Ok(n) = store::load_network(data, cid) {
        return Ok(n);
    }
    // Prefix scan among stored networks
    let cids = store::list_network_cids(data);
    let matches: Vec<_> = cids.iter().filter(|c| c.starts_with(cid)).collect();
    match matches.len() {
        0 => bail!("network '{cid}' not found in {}", data.display()),
        1 => store::load_network(data, matches[0]),
        _ => bail!("ambiguous network prefix '{cid}' — be more specific"),
    }
}
