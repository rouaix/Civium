mod store;

use anyhow::{bail, Result};
use civium_core::{
    network::{Invitation, Network},
    CiviumKeypair, CiviumNode, MemberRole, NodeConfig, TrustCircle,
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

        MemberCmd::List { network_cid } => {
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

// ── Helpers ───────────────────────────────────────────────────────────────────

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
