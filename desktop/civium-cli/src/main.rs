mod store;

use anyhow::{bail, Result};
use civium_core::{
    connection::ShareAgreement,
    network::{Invitation, Network},
    add_contest, compute_result_with_delegations,
    AdminAction, AdminActionKind, AdminActionStatus,
    Cid, CiviumKeypair, CiviumNode, CiviumRequest, CiviumResponse,
    ConnectionRecord, ConnectionState, DirectoryEntry, EntryKind, FederatedDirectory, GroupKey,
    GuardianLink, MemberRole, MinorRestrictions, NetworkKind, MessageKind, Multiaddr,
    NodeCommand, NodeConfig, NodeEvent, PeerId, PluginManifest, PluginState,
    Proposal, RrmEntry, ShareTerms, TrustCircle, TrustedRrm, Vote, VoteDelegation, peer_id_from_multiaddr,
};
use tracing::warn;
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
    /// Governance — proposals and votes
    Governance {
        #[command(subcommand)]
        action: GovernanceCmd,
    },
    /// Manage Civium directory networks
    Directory {
        #[command(subcommand)]
        action: DirectoryCmd,
    },
    /// Manage Registres des Réseaux Malveillants (RRM)
    Rrm {
        #[command(subcommand)]
        action: RrmCmd,
    },
    /// Manage installed plugins
    Plugin {
        #[command(subcommand)]
        action: PluginCmd,
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
    /// Trust an RRM — this network will consult it on connection checks.
    TrustRrm {
        /// Network CID short.
        #[arg(long)]
        network: String,
        /// CID short of the RRM to trust.
        #[arg(long)]
        rrm: String,
        /// Display name for this RRM.
        #[arg(long)]
        name: String,
    },
    /// Stop trusting an RRM.
    UntrustRrm {
        #[arg(long)]
        network: String,
        #[arg(long)]
        rrm: String,
    },
    /// List RRMs trusted by a network.
    TrustedRrms {
        #[arg(long)]
        network: String,
    },
    /// Check if a peer network is listed in any trusted RRM.
    CheckRrm {
        /// Network doing the check.
        #[arg(long)]
        network: String,
        /// CID short of the peer to check.
        #[arg(long)]
        peer: String,
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
    /// Mark a member as a minor (admin only).
    SetMinor {
        #[arg(long)]
        network: String,
        #[arg(long)]
        member: String,
    },
    /// Remove the minor flag from a member (admin only).
    UnsetMinor {
        #[arg(long)]
        network: String,
        #[arg(long)]
        member: String,
    },
    /// Add a guardian for a minor member (admin only).
    SetGuardian {
        #[arg(long)]
        network: String,
        #[arg(long)]
        minor: String,
        #[arg(long)]
        guardian: String,
    },
    /// Remove a guardian–minor link (admin only).
    RemoveGuardian {
        #[arg(long)]
        network: String,
        #[arg(long)]
        minor: String,
        #[arg(long)]
        guardian: String,
    },
    /// List guardians of a minor member.
    Guardians {
        #[arg(long)]
        network: String,
        #[arg(long)]
        minor: String,
    },
    /// List minors for which a member is guardian.
    Wards {
        #[arg(long)]
        network: String,
        #[arg(long)]
        guardian: String,
    },
    /// Set interaction restrictions for a minor member (admin only).
    SetRestrictions {
        #[arg(long)]
        network: String,
        #[arg(long)]
        minor: String,
        /// Max trust circle (0–2) the minor can interact in.
        #[arg(long, default_value = "1")]
        max_circle: u8,
        /// CID shorts explicitly allowed beyond max_circle (comma-separated).
        #[arg(long, default_value = "")]
        allowed: String,
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

// ── Governance sub-commands ───────────────────────────────────────────────────

#[derive(Subcommand)]
enum GovernanceCmd {
    /// Create a new proposal.
    Propose {
        /// Network CID short.
        #[arg(long)]
        network: String,
        /// Proposal title.
        #[arg(long)]
        title: String,
        /// Proposal description (optional).
        #[arg(long, default_value = "")]
        description: String,
        /// Voting options, comma-separated (default: "Pour,Contre,Abstention").
        #[arg(long, default_value = "Pour,Contre,Abstention")]
        options: String,
        /// Duration in hours until the vote closes (0 = open until admin closes it).
        #[arg(long, default_value = "72")]
        hours: u64,
        /// Required participation percentage for the result to be valid (0 = no quorum).
        #[arg(long, default_value = "0")]
        quorum: u8,
    },
    /// List proposals for a network.
    List {
        /// Network CID short.
        #[arg(long)]
        network: String,
    },
    /// Cast a vote on a proposal.
    Vote {
        /// Proposal ID (short prefix accepted).
        proposal_id: String,
        /// Network CID short (needed to find members count).
        #[arg(long)]
        network: String,
        /// Choice index (0 = first option, 1 = second, etc.).
        #[arg(long)]
        choice: usize,
    },
    /// Show vote results for a proposal.
    Results {
        /// Proposal ID (short prefix accepted).
        proposal_id: String,
        /// Network CID short.
        #[arg(long)]
        network: String,
    },
    /// Delegate your vote to another member.
    Delegate {
        #[arg(long)]
        network: String,
        /// CID short of the member to delegate to.
        #[arg(long)]
        to: String,
        /// Restrict delegation to a single proposal ID (omit for network-wide).
        #[arg(long)]
        proposal: Option<String>,
    },
    /// Revoke a vote delegation.
    RevokeDelegation {
        #[arg(long)]
        network: String,
        /// Restrict revocation to a specific proposal (omit to revoke the network-wide one).
        #[arg(long)]
        proposal: Option<String>,
    },
    /// List your current delegations for a network.
    Delegations {
        #[arg(long)]
        network: String,
    },
    /// List recent admin actions (garde-fou).
    Actions {
        #[arg(long)]
        network: String,
    },
    /// Contest an admin action (triggers a vote if majority agrees).
    Contest {
        /// Action ID (short prefix accepted).
        action_id: String,
        #[arg(long)]
        network: String,
    },
}

// ── Directory sub-commands ────────────────────────────────────────────────────

#[derive(Subcommand)]
enum DirectoryCmd {
    /// Create a new directory network (a Civium network of kind=directory).
    Create {
        #[arg(long)]
        name: String,
        /// Display name for the founding admin.
        #[arg(long, default_value = "admin")]
        display_name: String,
    },
    /// List all directory networks in your data directory.
    List,
    /// Publish a network or member to a directory.
    Publish {
        /// Directory network CID short.
        #[arg(long)]
        directory: String,
        /// CID short of the network or member to catalogue.
        #[arg(long)]
        subject: String,
        /// Human-readable name for the subject.
        #[arg(long)]
        name: String,
        /// Kind: network or member.
        #[arg(long, default_value = "network")]
        kind: String,
        /// Optional description.
        #[arg(long, default_value = "")]
        description: String,
        /// Optional P2P multiaddr to contact the subject (e.g. /ip4/1.2.3.4/tcp/4001/p2p/…).
        #[arg(long)]
        addr: Option<String>,
        /// Comma-separated tags (optional).
        #[arg(long, default_value = "")]
        tags: String,
    },
    /// Search entries in a directory.
    Search {
        /// Directory network CID short.
        #[arg(long)]
        directory: String,
        /// Also search entries from all federated directories.
        #[arg(long, default_value = "false")]
        federated: bool,
        /// Free-text query (name, description, CID, tags).
        query: String,
    },
    /// Remove an entry from a directory (admin only — by entry ID prefix).
    Remove {
        #[arg(long)]
        directory: String,
        /// Entry ID short prefix.
        entry_id: String,
    },
    /// Add a federation link to another directory.
    Federate {
        /// Host directory CID short.
        #[arg(long)]
        directory: String,
        /// CID short of the peer directory.
        #[arg(long)]
        peer: String,
        /// Display name of the peer directory.
        #[arg(long)]
        name: String,
        /// Optional P2P multiaddr to contact the peer directory.
        #[arg(long)]
        addr: Option<String>,
    },
    /// Remove a federation link.
    Unfederate {
        #[arg(long)]
        directory: String,
        /// CID short of the peer directory to remove.
        #[arg(long)]
        peer: String,
    },
    /// List federation links for a directory.
    Federations {
        #[arg(long)]
        directory: String,
    },
}

// ── RRM sub-commands ─────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum RrmCmd {
    /// Create a new RRM network (kind = rrm).
    Create {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "admin")]
        display_name: String,
    },
    /// List all RRM networks in your data directory.
    List,
    /// Report a malicious network to an RRM.
    Report {
        /// RRM network CID short.
        #[arg(long)]
        rrm: String,
        /// CID short of the network being reported.
        #[arg(long)]
        network: String,
        /// Human-readable name of the reported network.
        #[arg(long)]
        name: String,
        /// Reason for the report.
        #[arg(long)]
        reason: String,
        /// Optional URL to evidence (article, screenshot, etc.).
        #[arg(long)]
        evidence: Option<String>,
    },
    /// Search reports in an RRM (optional free-text query).
    Search {
        #[arg(long)]
        rrm: String,
        /// Free-text query (name, CID, reason). Omit to list all.
        query: Option<String>,
    },
    /// Remove a report from an RRM (by entry ID prefix).
    Remove {
        #[arg(long)]
        rrm: String,
        entry_id: String,
    },
}

// ── Plugin sub-commands ───────────────────────────────────────────────────────

#[derive(Subcommand)]
enum PluginCmd {
    /// List installed plugins and their status.
    List,
    /// Show details for a plugin.
    Info {
        /// Plugin ID (e.g. civium.messagerie)
        id: String,
    },
    /// Enable a plugin.
    Enable {
        id: String,
    },
    /// Disable a plugin (system plugins cannot be disabled).
    Disable {
        id: String,
    },
    /// Install a plugin from a JSON manifest file.
    Install {
        /// Path to the manifest JSON file.
        path: String,
    },
}

// ── Node sub-commands ─────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum NodeCmd {
    /// Start the local P2P node (event loop — press Ctrl-C to stop).
    Start {
        #[arg(long, default_value = "/ip4/0.0.0.0/tcp/0")]
        listen_tcp: String,
        #[arg(long, default_value = "/ip4/0.0.0.0/udp/0/quic-v1")]
        listen_quic: String,
        #[arg(long = "peer")]
        peers: Vec<String>,
        /// Network CID(s) to announce to the DHT after connecting.
        #[arg(long = "announce")]
        announce: Vec<String>,
    },
    /// Sync state with a remote peer for a specific network (one-shot).
    Sync {
        /// Network CID short or prefix.
        #[arg(long)]
        network: String,
        /// Multiaddr of a peer to sync with (must include /p2p/<PeerId>).
        #[arg(long)]
        via: String,
        #[arg(long, default_value = "/ip4/0.0.0.0/tcp/0")]
        listen_tcp: String,
        #[arg(long, default_value = "/ip4/0.0.0.0/udp/0/quic-v1")]
        listen_quic: String,
    },
    /// Join a network over P2P using an invitation link (no shared DB needed).
    JoinP2p {
        /// The civium-invite:… link.
        invite_link: String,
        /// Your display name in the network.
        #[arg(long)]
        name: String,
        /// Multiaddr of a known peer in the target network (e.g. /ip4/1.2.3.4/tcp/4001/p2p/12D3…).
        #[arg(long)]
        via: String,
        #[arg(long, default_value = "/ip4/0.0.0.0/tcp/0")]
        listen_tcp: String,
        #[arg(long, default_value = "/ip4/0.0.0.0/udp/0/quic-v1")]
        listen_quic: String,
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
        Command::Governance { action } => run_governance(action, data),
        Command::Directory { action } => run_directory(action, data),
        Command::Rrm { action } => run_rrm(action, data),
        Command::Plugin { action } => run_plugin(action, data),
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

        NetworkCmd::TrustRrm { network, rrm, name } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let net = load_network_fuzzy(data, &network)?;
            let rrm_net = load_network_fuzzy(data, &rrm)?;
            if rrm_net.data.kind != NetworkKind::Rrm {
                bail!("Network '{}' is not an RRM — create one with `rrm create`.", rrm);
            }
            let trust = TrustedRrm::new(
                net.cid_short().to_string(),
                rrm_net.cid_short().to_string(),
                name.clone(),
                keypair.cid().short().to_string(),
            );
            store::save_trusted_rrm(data, &trust)?;
            println!("Network '{}' now trusts RRM '{name}' ({}).", net.name(), rrm_net.cid_short());
        }

        NetworkCmd::UntrustRrm { network, rrm } => {
            let net = load_network_fuzzy(data, &network)?;
            store::delete_trusted_rrm(data, net.cid_short(), &rrm)?;
            println!("Network '{}' no longer trusts RRM '{rrm}'.", net.name());
        }

        NetworkCmd::TrustedRrms { network } => {
            let net = load_network_fuzzy(data, &network)?;
            let trusts = store::list_trusted_rrms(data, net.cid_short())?;
            if trusts.is_empty() {
                println!("No trusted RRMs for network '{}'.", net.name());
                return Ok(());
            }
            println!("Trusted RRMs for '{}':", net.name());
            for t in &trusts {
                println!("  {} — {}", t.rrm_name, t.rrm_cid_short);
            }
        }

        NetworkCmd::CheckRrm { network, peer } => {
            let net = load_network_fuzzy(data, &network)?;
            let warnings = store::check_rrm_warnings(data, net.cid_short(), &peer)?;
            if warnings.is_empty() {
                println!("✓ Peer '{peer}' is not listed in any trusted RRM for '{}'.", net.name());
            } else {
                println!("⚠ Peer '{peer}' is listed in {} RRM(s):", warnings.len());
                for (trust, entry) in &warnings {
                    println!("  RRM: {} ({})", trust.rrm_name, trust.rrm_cid_short);
                    println!("  Reported network: {} ({})", entry.network_name, entry.network_cid_short);
                    println!("  Reason: {}", entry.reason);
                    if let Some(url) = &entry.evidence_url {
                        println!("  Evidence: {url}");
                    }
                    println!("  Reported: {}", fmt_ts(entry.reported_at));
                }
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
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let admin_cid = keypair.cid();

            let circle = TrustCircle::from_u8(circle)
                .ok_or_else(|| anyhow::anyhow!("invalid circle {circle} — use 0, 1, or 2"))?;
            let role: MemberRole = role.parse().map_err(|e: String| anyhow::anyhow!("{e}"))?;

            let mut network = load_network_fuzzy(data, &network_cid)?;
            let record = network
                .admit(&member_cid, circle, role)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            store::save_network(data, &network)?;

            // Record admin action for the garde-fou
            let now = unix_now_cli();
            let action = AdminAction::new(
                network.cid_short().to_string(),
                AdminActionKind::MemberAdmitted {
                    member_cid_short: record.cid_short.clone(),
                    display_name: record.display_name.clone(),
                },
                admin_cid.short().to_string(),
                now,
                0,
            );
            store::save_admin_action(data, network.cid_short(), &action)?;

            println!(
                "Admitted {} as '{}' — circle: {}, role: {}",
                record.cid_short, record.display_name, record.circle, record.role
            );
            println!("Network address: {}@{}", record.cid_short, network.cid_short());
            println!("Action ID (garde-fou): {}", action.id);
        }

        MemberCmd::Reject { network_cid, member_cid } => {
            let mut network = load_network_fuzzy(data, &network_cid)?;
            network.reject(&member_cid).map_err(|e| anyhow::anyhow!("{e}"))?;
            store::save_network(data, &network)?;
            println!("Join request from {member_cid} rejected.");
        }

        MemberCmd::SetMinor { network, member } => {
            store::set_member_minor(data, &network, &member, true)?;
            println!("{member} marked as minor in network {network}.");
        }

        MemberCmd::UnsetMinor { network, member } => {
            store::set_member_minor(data, &network, &member, false)?;
            println!("{member} minor flag removed in network {network}.");
        }

        MemberCmd::SetGuardian { network, minor, guardian } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let link = GuardianLink::new(
                network.clone(),
                minor.clone(),
                guardian.clone(),
                keypair.cid().short().to_string(),
            );
            store::save_guardian_link(data, &link)?;
            println!("{guardian} added as guardian of {minor} in network {network}.");
        }

        MemberCmd::RemoveGuardian { network, minor, guardian } => {
            store::delete_guardian_link(data, &network, &minor, &guardian)?;
            println!("Guardian link {guardian}→{minor} removed from network {network}.");
        }

        MemberCmd::Guardians { network, minor } => {
            let links = store::list_guardians(data, &network, &minor)?;
            if links.is_empty() {
                println!("No guardians for {minor} in network {network}.");
            } else {
                println!("Guardians of {minor} in network {network}:");
                for l in &links {
                    println!("  {} (added by {} at {})", l.guardian_cid_short, l.added_by, l.added_at);
                }
            }
        }

        MemberCmd::Wards { network, guardian } => {
            let links = store::list_wards(data, &network, &guardian)?;
            if links.is_empty() {
                println!("No minors under guardianship of {guardian} in network {network}.");
            } else {
                println!("Minors under {guardian} in network {network}:");
                for l in &links {
                    println!("  {} (added by {} at {})", l.minor_cid_short, l.added_by, l.added_at);
                }
            }
        }

        MemberCmd::SetRestrictions { network, minor, max_circle, allowed } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let allowed_cids: Vec<String> = if allowed.is_empty() {
                vec![]
            } else {
                allowed.split(',').map(|s| s.trim().to_string()).collect()
            };
            let r = MinorRestrictions::new(
                network.clone(),
                minor.clone(),
                max_circle,
                allowed_cids,
                keypair.cid().short().to_string(),
            );
            store::save_minor_restrictions(data, &r)?;
            println!("Restrictions set for {minor} in network {network}: max_circle={}, allowed={:?}", r.max_circle, r.allowed_cid_shorts);
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
        // ── Start ─────────────────────────────────────────────────────────────
        NodeCmd::Start { listen_tcp, listen_quic, peers, announce } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let cid = keypair.cid();

            let config = NodeConfig { listen_tcp, listen_quic, bootstrap_peers: peers };

            println!("Starting Civium node");
            println!("  CID        : {}", cid.short());
            println!("  Peer ID    : {}", {
                let kp = keypair.libp2p_keypair();
                kp.public().to_peer_id()
            });

            for cid_short in store::list_network_cids(data) {
                if let Ok(n) = store::load_network(data, &cid_short) {
                    println!("  Network    : {} ({})", n.name(), n.cid_short());
                }
            }

            let (node, mut handle) = CiviumNode::new(keypair, config).await
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            // Announce requested networks once we've heard our first listen address
            let announce_list = announce.clone();
            let data2 = data.clone();

            tokio::spawn(async move {
                // Wait for the first Listening event before announcing
                let mut announced = false;
                loop {
                    match handle.events.recv().await {
                        Some(NodeEvent::Listening { addr }) => {
                            println!("Listening on {addr}");
                            if !announced {
                                for cid_short in &announce_list {
                                    let _ = handle.commands.send(
                                        NodeCommand::AnnounceNetwork {
                                            network_cid_short: cid_short.clone(),
                                        }
                                    ).await;
                                    println!("Announced network {cid_short} to DHT");
                                }
                                announced = true;
                            }
                        }
                        Some(NodeEvent::PeerConnected { peer_id }) => {
                            println!("Peer connected: {peer_id}");
                        }
                        Some(NodeEvent::PeersDiscovered { network_cid_short, peer_addrs }) => {
                            println!("Peers for network {network_cid_short}:");
                            for a in &peer_addrs { println!("  {a}"); }
                        }
                        Some(NodeEvent::InboundRequest { from, request_id, request }) => {
                            let response = handle_inbound_request(&data2, from, &request);
                            let _ = handle.commands.send(
                                NodeCommand::Respond { request_id, response }
                            ).await;
                        }
                        Some(NodeEvent::OutboundResponse { response, .. }) => {
                            println!("Response: {response:?}");
                        }
                        None => break,
                    }
                }
            });

            node.run().await;
        }

        // ── Sync ─────────────────────────────────────────────────────────────
        NodeCmd::Sync { network, via, listen_tcp, listen_quic } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let net = load_network_fuzzy(data, &network)?;

            println!("Syncing '{}' via {via}…", net.name());

            let via_addr: Multiaddr = via.parse()
                .map_err(|e| anyhow::anyhow!("invalid multiaddr: {e}"))?;
            let peer_id = peer_id_from_multiaddr(&via_addr)
                .ok_or_else(|| anyhow::anyhow!("--via must include /p2p/<PeerId>"))?;

            let network_cid_full = net.cid_full().to_string();
            let network_cid_short = net.cid_short().to_string();
            let config = NodeConfig { listen_tcp, listen_quic, bootstrap_peers: vec![via.clone()] };

            let (node, mut handle) = CiviumNode::new(keypair, config).await
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            let data2 = data.clone();
            tokio::spawn(async move {
                loop {
                    match handle.events.recv().await {
                        Some(NodeEvent::PeerConnected { peer_id: connected }) if connected == peer_id => {
                            let _ = handle.commands.send(NodeCommand::SendRequest {
                                peer: peer_id,
                                request: CiviumRequest::Sync {
                                    network_cid_full: network_cid_full.clone(),
                                    since_ts: 0,
                                },
                            }).await;
                        }
                        Some(NodeEvent::OutboundResponse { response: CiviumResponse::SyncData { members, messages, .. }, .. }) => {
                            match store::merge_sync_data(&data2, &network_cid_short, &members, &messages) {
                                Ok(()) => println!("Synced: {} members, {} messages", members.len(), messages.len()),
                                Err(e) => eprintln!("Sync error: {e}"),
                            }
                            std::process::exit(0);
                        }
                        None => break,
                        _ => {}
                    }
                }
            });

            node.run().await;
        }

        // ── JoinP2p ───────────────────────────────────────────────────────────
        NodeCmd::JoinP2p { invite_link, name, via, listen_tcp, listen_quic } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let invitation = Invitation::from_link(&invite_link)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            invitation.verify().map_err(|e| anyhow::anyhow!("{e}"))?;

            println!("Joining '{}' over P2P…", invitation.network_name());
            println!("  As: {name}");
            println!("  Via: {via}");

            let member_cid = keypair.cid();
            let config = NodeConfig {
                listen_tcp,
                listen_quic,
                bootstrap_peers: vec![via.clone()],
            };

            let (node, mut handle) = CiviumNode::new(keypair, config).await
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            // Parse the via address to extract PeerId
            let via_addr: Multiaddr = via.parse()
                .map_err(|e| anyhow::anyhow!("invalid multiaddr: {e}"))?;
            let peer_id = peer_id_from_multiaddr(&via_addr)
                .ok_or_else(|| anyhow::anyhow!("--via address must include /p2p/<PeerId>"))?;

            let request = civium_core::CiviumRequest::Join {
                invite_link: invite_link.clone(),
                member_cid_full: member_cid.full().to_string(),
                display_name: name.clone(),
            };

            let data2 = data.clone();
            tokio::spawn(async move {
                // Wait until connected, then send the join request
                loop {
                    match handle.events.recv().await {
                        Some(NodeEvent::PeerConnected { peer_id: connected_id }) => {
                            if connected_id == peer_id {
                                let _ = handle.commands.send(NodeCommand::SendRequest {
                                    peer: peer_id,
                                    request: request.clone(),
                                }).await;
                            }
                        }
                        Some(NodeEvent::OutboundResponse { response, .. }) => {
                            match response {
                                CiviumResponse::JoinAccepted { network_data } => {
                                    match Network::from_data(network_data) {
                                        Ok(network) => {
                                            match store::save_network(&data2, &network) {
                                                Ok(()) => println!(
                                                    "Joined '{}' — saved to local store.",
                                                    network.name()
                                                ),
                                                Err(e) => eprintln!("Failed to save: {e}"),
                                            }
                                        }
                                        Err(e) => eprintln!("Invalid network data: {e}"),
                                    }
                                    std::process::exit(0);
                                }
                                CiviumResponse::JoinRejected { reason } => {
                                    eprintln!("Join rejected: {reason}");
                                    std::process::exit(1);
                                }
                                other => eprintln!("Unexpected response: {other:?}"),
                            }
                        }
                        None => break,
                        _ => {}
                    }
                }
            });

            node.run().await;
        }
    }
    Ok(())
}

/// Respond to an inbound Civium request using the local store.
fn handle_inbound_request(
    data: &PathBuf,
    from: PeerId,
    request: &CiviumRequest,
) -> CiviumResponse {
    match request {
        CiviumRequest::Ping => CiviumResponse::Pong,

        CiviumRequest::Join { invite_link, member_cid_full, display_name } => {
            let result = (|| -> anyhow::Result<CiviumResponse> {
                let invitation = Invitation::from_link(invite_link)?;
                invitation.verify()?;

                let network_cid_full = invitation.network_cid_full().to_string();
                let cid_short = store::list_network_cids(data)
                    .into_iter()
                    .find(|c| {
                        store::load_network(data, c)
                            .map(|n| n.cid_full() == network_cid_full)
                            .unwrap_or(false)
                    })
                    .ok_or_else(|| anyhow::anyhow!("network '{}' not found locally", invitation.network_name()))?;

                let mut network = store::load_network(data, &cid_short)?;
                let member_cid = Cid::from_full(member_cid_full)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;

                network.submit_join_request(&member_cid, display_name.clone(), &invitation)?;
                network.admit(member_cid.short(), TrustCircle::Connaissance, MemberRole::Member)?;
                store::save_network(data, &network)?;

                println!("[P2P] Admitted '{}' to '{}'", display_name, network.name());
                Ok(CiviumResponse::JoinAccepted { network_data: network.data })
            })();

            match result {
                Ok(r) => r,
                Err(e) => {
                    warn!(%from, err = %e, "join request failed");
                    CiviumResponse::JoinRejected { reason: e.to_string() }
                }
            }
        }

        CiviumRequest::Sync { network_cid_full, since_ts } => {
            let result = (|| -> anyhow::Result<CiviumResponse> {
                let cid_short = store::list_network_cids(data)
                    .into_iter()
                    .find(|c| {
                        store::load_network(data, c)
                            .map(|n| n.cid_full() == *network_cid_full)
                            .unwrap_or(false)
                    })
                    .ok_or_else(|| anyhow::anyhow!("network not found"))?;

                let network = store::load_network(data, &cid_short)?;
                let mailbox = store::load_mailbox(data, &cid_short)?;

                let members = network.data.members.into_iter()
                    .filter(|m| m.joined_at >= *since_ts)
                    .collect();
                let messages = mailbox.messages.into_iter()
                    .filter(|m| m.sent_at >= *since_ts)
                    .collect();

                Ok(CiviumResponse::SyncData { network_cid_full: network_cid_full.clone(), members, messages })
            })();

            result.unwrap_or_else(|e| CiviumResponse::Error { message: e.to_string() })
        }
    }
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
                    let author_cid_short = net.data.members.iter()
                        .find(|m| m.cid_full == author_cid.full())
                        .map(|m| m.cid_short.clone())
                        .unwrap_or_default();
                    // Enforce minor restrictions in both directions
                    store::check_minor_interaction(data, net.cid_short(), &peer_cid_short, &author_cid_short)
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    store::check_minor_interaction(data, net.cid_short(), &author_cid_short, &peer_cid_short)
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
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

// ── Governance handler ────────────────────────────────────────────────────────

fn run_governance(cmd: GovernanceCmd, data: &PathBuf) -> Result<()> {
    match cmd {
        GovernanceCmd::Propose { network, title, description, options, hours, quorum } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let author_cid = keypair.cid();

            let net = load_network_fuzzy(data, &network)?;
            if !net.data.members.iter().any(|m| m.cid_full == author_cid.full()) {
                bail!("you are not a member of '{}'", net.name());
            }

            let now = unix_now_cli();
            let closes_at = if hours == 0 { 0 } else { now + hours * 3600 };
            let opts: Vec<String> = options.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            if opts.len() < 2 {
                bail!("at least 2 options are required");
            }

            let proposal = Proposal::new(
                net.cid_short().to_string(),
                title.clone(),
                description,
                opts,
                author_cid.short().to_string(),
                now,
                closes_at,
                quorum,
            );

            store::save_proposal(data, net.cid_short(), &proposal)?;
            println!("Proposal '{}' created.", title);
            println!("  ID      : {}", proposal.id);
            println!("  Options : {}", proposal.options.join(", "));
            if closes_at > 0 {
                println!("  Closes  : +{hours}h");
            } else {
                println!("  Closes  : open-ended");
            }
        }

        GovernanceCmd::List { network } => {
            let net = load_network_fuzzy(data, &network)?;
            let proposals = store::list_proposals(data, net.cid_short())?;
            if proposals.is_empty() {
                println!("No proposals for '{}'.", net.name());
                return Ok(());
            }
            println!("=== Proposals: {} ===", net.name());
            println!();
            for p in &proposals {
                println!("  [{}] {} — {} ({})", p.id, p.title, p.status, p.options.join(" / "));
                if !p.description.is_empty() {
                    println!("       {}", p.description);
                }
            }
        }

        GovernanceCmd::Vote { proposal_id, network, choice } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let voter_cid = keypair.cid();

            let net = load_network_fuzzy(data, &network)?;
            if !net.data.members.iter().any(|m| m.cid_full == voter_cid.full()) {
                bail!("you are not a member of '{}'", net.name());
            }

            let proposals = store::list_proposals(data, net.cid_short())?;
            let proposal = proposals
                .iter()
                .find(|p| p.id.starts_with(&proposal_id))
                .ok_or_else(|| anyhow::anyhow!("proposal '{}' not found", proposal_id))?;

            if proposal.status != civium_core::ProposalStatus::Open {
                bail!("proposal '{}' is not open", proposal.id);
            }
            if choice >= proposal.options.len() {
                bail!("choice {} out of range (0–{})", choice, proposal.options.len() - 1);
            }

            let now = unix_now_cli();
            if proposal.is_expired(now) {
                bail!("proposal '{}' has expired", proposal.id);
            }

            let vote = Vote {
                proposal_id: proposal.id.clone(),
                voter_cid_short: voter_cid.short().to_string(),
                choice_index: choice,
                cast_at: now,
            };
            store::save_vote(data, &vote)?;
            println!(
                "Vote cast: '{}' on proposal '{}'.",
                proposal.options[choice], proposal.id
            );
        }

        GovernanceCmd::Delegate { network, to, proposal } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let delegator_cid = keypair.cid();
            let net = load_network_fuzzy(data, &network)?;

            if !net.data.members.iter().any(|m| m.cid_full == delegator_cid.full()) {
                bail!("you are not a member of '{}'", net.name());
            }
            if delegator_cid.short() == to.as_str() {
                bail!("cannot delegate to yourself");
            }
            if !net.data.members.iter().any(|m| m.cid_short.starts_with(&to)) {
                bail!("member '{}' not found in '{}'", to, net.name());
            }
            let delegate_cid_short = net.data.members.iter()
                .find(|m| m.cid_short.starts_with(&to))
                .map(|m| m.cid_short.clone()).unwrap();

            let now = unix_now_cli();
            let delegation = VoteDelegation {
                delegator_cid_short: delegator_cid.short().to_string(),
                delegate_cid_short: delegate_cid_short.clone(),
                network_cid_short: net.cid_short().to_string(),
                proposal_id: proposal.clone(),
                created_at: now,
            };
            store::save_delegation(data, &delegation)?;
            match &proposal {
                Some(pid) => println!("Vote delegated to {} for proposal {}.", delegate_cid_short, pid),
                None => println!("Vote delegated to {} for all proposals in '{}'.", delegate_cid_short, net.name()),
            }
        }

        GovernanceCmd::RevokeDelegation { network, proposal } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let delegator_cid = keypair.cid();
            let net = load_network_fuzzy(data, &network)?;
            store::delete_delegation(data, net.cid_short(), delegator_cid.short(), proposal.as_deref())?;
            println!("Delegation revoked.");
        }

        GovernanceCmd::Delegations { network } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let my_cid = keypair.cid();
            let net = load_network_fuzzy(data, &network)?;
            let all = store::list_delegations(data, net.cid_short())?;
            let mine: Vec<_> = all.iter().filter(|d| d.delegator_cid_short == my_cid.short()).collect();
            if mine.is_empty() {
                println!("No active delegations for '{}'.", net.name());
                return Ok(());
            }
            for d in mine {
                match &d.proposal_id {
                    None => println!("  → {} (network-wide)", d.delegate_cid_short),
                    Some(pid) => println!("  → {} (proposal {})", d.delegate_cid_short, pid),
                }
            }
        }

        GovernanceCmd::Actions { network } => {
            let net = load_network_fuzzy(data, &network)?;
            let actions = store::list_admin_actions(data, net.cid_short())?;
            if actions.is_empty() {
                println!("No admin actions recorded for '{}'.", net.name());
                return Ok(());
            }
            println!("=== Admin actions: {} ===", net.name());
            println!();
            let now = unix_now_cli();
            for a in &actions {
                let window = if a.is_window_open(now) {
                    let remaining = (a.taken_at + a.contest_window_secs).saturating_sub(now);
                    format!("({}h restantes pour contester)", remaining / 3600)
                } else {
                    String::new()
                };
                println!("  [{}] {} — {} — {} conteste(s) {}",
                    a.id, a.kind, a.status, a.contests.len(), window);
            }
        }

        GovernanceCmd::Contest { action_id, network } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let voter_cid = keypair.cid();

            let net = load_network_fuzzy(data, &network)?;
            if !net.data.members.iter().any(|m| m.cid_full == voter_cid.full()) {
                bail!("you are not a member of '{}'", net.name());
            }

            let mut actions = store::list_admin_actions(data, net.cid_short())?;
            let action = actions
                .iter_mut()
                .find(|a| a.id.starts_with(&action_id))
                .ok_or_else(|| anyhow::anyhow!("action '{}' not found", action_id))?;

            if action.status != AdminActionStatus::Active {
                bail!("action '{}' is no longer contestable (status: {})", action.id, action.status);
            }
            let now = unix_now_cli();
            if !action.is_window_open(now) {
                action.status = AdminActionStatus::Confirmed;
                store::save_admin_action(data, net.cid_short(), action)?;
                bail!("contest window has closed for action '{}'", action.id);
            }

            let total_members = net.data.members.len();
            let threshold_reached = add_contest(action, voter_cid.short(), total_members);
            println!("Contest recorded. ({}/{} membres ont contesté)",
                action.contests.len(), total_members);

            if threshold_reached {
                // Auto-create a suspension proposal
                let prop_now = unix_now_cli();
                let proposal = Proposal::new(
                    net.cid_short().to_string(),
                    format!("Garde-fou : {}", action.kind),
                    format!("La majorité a contesté une action de l'admin. Que décide le réseau ?"),
                    vec!["Maintenir l'action".into(), "Annuler l'action".into()],
                    "système".into(),
                    prop_now,
                    prop_now + 72 * 3600,
                    0,
                );
                store::save_proposal(data, net.cid_short(), &proposal)?;
                action.status = AdminActionStatus::Suspended { proposal_id: proposal.id.clone() };
                println!("Seuil majoritaire atteint — action SUSPENDUE.");
                println!("Vote automatique créé : {} ({})", proposal.title, proposal.id);
            }
            store::save_admin_action(data, net.cid_short(), action)?;
        }

        GovernanceCmd::Results { proposal_id, network } => {
            let net = load_network_fuzzy(data, &network)?;
            let proposals = store::list_proposals(data, net.cid_short())?;
            let proposal = proposals
                .iter()
                .find(|p| p.id.starts_with(&proposal_id))
                .ok_or_else(|| anyhow::anyhow!("proposal '{}' not found", proposal_id))?;

            let votes = store::list_votes(data, &proposal.id)?;
            let delegations = store::list_delegations(data, net.cid_short())?;
            let total_members = net.data.members.len();
            let result = compute_result_with_delegations(proposal, &votes, &delegations, total_members);

            println!("=== Results: {} ===", proposal.title);
            println!("  Status        : {}", proposal.status);
            println!("  Votes cast    : {}/{} ({:.1}%)", result.total_votes, total_members, result.participation_percent);
            println!("  Quorum        : {}", if result.quorum_reached { "reached" } else { "NOT reached" });
            println!();
            for (i, opt) in result.options.iter().enumerate() {
                let marker = result.winner.map(|w| if w == i { " ← WINNER" } else { "" }).unwrap_or("");
                println!("  [{}] {} — {} votes ({:.1}%){}", i, opt.label, opt.votes, opt.percent, marker);
            }
        }
    }
    Ok(())
}

// ── Directory handlers ────────────────────────────────────────────────────────

fn run_directory(cmd: DirectoryCmd, data: &PathBuf) -> Result<()> {
    match cmd {
        DirectoryCmd::Create { name, display_name } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let admin_cid = keypair.cid();
            let mut network = Network::create(name.clone(), &admin_cid, display_name)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            network.data.kind = NetworkKind::Directory;
            store::save_network(data, &network)?;
            println!("Directory network '{}' created.", name);
            println!("  CID (short) : {}", network.cid_short());
            println!("  CID (full)  : {}", network.cid_full());
        }

        DirectoryCmd::List => {
            let cids = store::list_network_cids(data);
            let dirs: Vec<_> = cids
                .iter()
                .filter_map(|cid| store::load_network(data, cid).ok())
                .filter(|n| n.data.kind == NetworkKind::Directory)
                .collect();
            if dirs.is_empty() {
                println!("No directory networks. Create one with `directory create --name <name>`.");
                return Ok(());
            }
            for n in &dirs {
                let entries = store::list_directory_entries(data, n.cid_short()).unwrap_or_default();
                println!("{} — {} ({} entries)", n.cid_short(), n.name(), entries.len());
            }
        }

        DirectoryCmd::Publish { directory, subject, name, kind, description, addr, tags } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let publisher_cid = keypair.cid().short().to_string();

            let dir_net = load_network_fuzzy(data, &directory)?;
            if dir_net.data.kind != NetworkKind::Directory {
                bail!("Network '{}' is not a directory — create one with `directory create`.", directory);
            }

            let entry_kind: EntryKind = kind.parse().map_err(|e: String| anyhow::anyhow!("{e}"))?;
            let tag_list: Vec<String> = tags
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            let entry = DirectoryEntry::new(
                dir_net.cid_short().to_string(),
                entry_kind,
                subject.clone(),
                name.clone(),
                description,
                addr,
                publisher_cid,
                tag_list,
            );
            store::save_directory_entry(data, &entry)?;
            println!("Published '{}' ({}) to directory '{}'.", name, subject, dir_net.name());
            println!("  Entry ID : {}", entry.id);
        }

        DirectoryCmd::Search { directory, federated, query } => {
            let dir_net = load_network_fuzzy(data, &directory)?;
            let mut results = store::search_directory_entries(data, dir_net.cid_short(), &query)?;
            let mut sources: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            for e in &results {
                sources.entry(e.id.clone()).or_insert_with(|| dir_net.name().to_string());
            }

            if federated {
                let feds = store::list_federations(data, dir_net.cid_short()).unwrap_or_default();
                for fed in &feds {
                    let peer_entries = store::search_directory_entries(data, &fed.peer_cid_short, &query)
                        .unwrap_or_default();
                    for e in &peer_entries {
                        sources.entry(e.id.clone()).or_insert_with(|| fed.peer_name.clone());
                    }
                    results.extend(peer_entries);
                }
            }

            if results.is_empty() {
                println!("No entries matching '{query}'.");
                return Ok(());
            }
            println!("Results in '{}'{} for '{query}':",
                dir_net.name(),
                if federated { " (+ fédérés)" } else { "" });
            for e in &results {
                let src = sources.get(&e.id).map(|s| s.as_str()).unwrap_or("?");
                println!("  [{}] {} — {} ({}){}", e.kind, e.subject_name, e.subject_cid_short, e.id,
                    if src != dir_net.name() { format!(" [via {src}]") } else { String::new() });
                if !e.description.is_empty() {
                    println!("      {}", e.description);
                }
                if let Some(addr) = &e.contact_addr {
                    println!("      addr: {addr}");
                }
                if !e.tags.is_empty() {
                    println!("      tags: {}", e.tags.join(", "));
                }
            }
        }

        DirectoryCmd::Remove { directory, entry_id } => {
            let dir_net = load_network_fuzzy(data, &directory)?;
            let entries = store::list_directory_entries(data, dir_net.cid_short())?;
            let entry = entries
                .iter()
                .find(|e| e.id.starts_with(&entry_id))
                .ok_or_else(|| anyhow::anyhow!("no entry with ID starting with '{entry_id}'"))?;
            let full_id = entry.id.clone();
            store::delete_directory_entry(data, dir_net.cid_short(), &full_id)?;
            println!("Entry '{full_id}' removed from directory '{}'.", dir_net.name());
        }

        DirectoryCmd::Federate { directory, peer, name, addr } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let publisher = keypair.cid().short().to_string();
            let dir_net = load_network_fuzzy(data, &directory)?;
            if dir_net.data.kind != NetworkKind::Directory {
                bail!("Network '{}' is not a directory.", directory);
            }
            let fed = FederatedDirectory::new(
                dir_net.cid_short().to_string(),
                peer.clone(),
                name.clone(),
                addr,
                publisher,
            );
            store::save_federation(data, &fed)?;
            println!("Directory '{}' now federates with '{name}' ({peer}).", dir_net.name());
        }

        DirectoryCmd::Unfederate { directory, peer } => {
            let dir_net = load_network_fuzzy(data, &directory)?;
            store::delete_federation(data, dir_net.cid_short(), &peer)?;
            println!("Federation with '{peer}' removed from directory '{}'.", dir_net.name());
        }

        DirectoryCmd::Federations { directory } => {
            let dir_net = load_network_fuzzy(data, &directory)?;
            let feds = store::list_federations(data, dir_net.cid_short())?;
            if feds.is_empty() {
                println!("No federations for directory '{}'.", dir_net.name());
                return Ok(());
            }
            println!("Federations for '{}':", dir_net.name());
            for f in &feds {
                println!("  {} — {}", f.peer_name, f.peer_cid_short);
                if let Some(addr) = &f.peer_addr {
                    println!("    addr: {addr}");
                }
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

// ── RRM handlers ──────────────────────────────────────────────────────────────

fn run_rrm(cmd: RrmCmd, data: &PathBuf) -> Result<()> {
    match cmd {
        RrmCmd::Create { name, display_name } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let admin_cid = keypair.cid();
            let mut network = Network::create(name.clone(), &admin_cid, display_name)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            network.data.kind = NetworkKind::Rrm;
            store::save_network(data, &network)?;
            println!("RRM network '{}' created.", name);
            println!("  CID (short) : {}", network.cid_short());
            println!("  CID (full)  : {}", network.cid_full());
        }

        RrmCmd::List => {
            let cids = store::list_network_cids(data);
            let rrms: Vec<_> = cids
                .iter()
                .filter_map(|cid| store::load_network(data, cid).ok())
                .filter(|n| n.data.kind == NetworkKind::Rrm)
                .collect();
            if rrms.is_empty() {
                println!("No RRM networks. Create one with `rrm create --name <name>`.");
                return Ok(());
            }
            for n in &rrms {
                let entries = store::list_rrm_entries(data, n.cid_short()).unwrap_or_default();
                println!("{} — {} ({} reports)", n.cid_short(), n.name(), entries.len());
            }
        }

        RrmCmd::Report { rrm, network, name, reason, evidence } => {
            let keypair = store::load_identity(data)
                .map_err(|_| anyhow::anyhow!("no identity found — run `identity init` first"))?;
            let reporter = keypair.cid().short().to_string();
            let rrm_net = load_network_fuzzy(data, &rrm)?;
            if rrm_net.data.kind != NetworkKind::Rrm {
                bail!("Network '{}' is not an RRM.", rrm);
            }
            let entry = RrmEntry::new(
                rrm_net.cid_short().to_string(),
                network.clone(),
                name.clone(),
                reason.clone(),
                evidence,
                reporter,
            );
            store::save_rrm_entry(data, &entry)?;
            println!("Network '{}' ({}) reported to RRM '{}'.", name, network, rrm_net.name());
            println!("  Entry ID : {}", entry.id);
        }

        RrmCmd::Search { rrm, query } => {
            let rrm_net = load_network_fuzzy(data, &rrm)?;
            let entries = match &query {
                Some(q) => store::search_rrm_entries(data, rrm_net.cid_short(), q)?,
                None    => store::list_rrm_entries(data, rrm_net.cid_short())?,
            };
            if entries.is_empty() {
                println!("No reports in RRM '{}'{}.",
                    rrm_net.name(),
                    query.as_deref().map(|q| format!(" for '{q}'")).unwrap_or_default());
                return Ok(());
            }
            println!("Reports in '{}'{}:",
                rrm_net.name(),
                query.as_deref().map(|q| format!(" matching '{q}'")).unwrap_or_default());
            for e in &entries {
                println!("  [{}] {} — {}", e.id, e.network_name, e.network_cid_short);
                println!("      Reason: {}", e.reason);
                if let Some(url) = &e.evidence_url {
                    println!("      Evidence: {url}");
                }
                println!("      Reported: {} by {}", fmt_ts(e.reported_at), e.reported_by);
            }
        }

        RrmCmd::Remove { rrm, entry_id } => {
            let rrm_net = load_network_fuzzy(data, &rrm)?;
            let entries = store::list_rrm_entries(data, rrm_net.cid_short())?;
            let entry = entries
                .iter()
                .find(|e| e.id.starts_with(&entry_id))
                .ok_or_else(|| anyhow::anyhow!("no entry with ID starting with '{entry_id}'"))?;
            let full_id = entry.id.clone();
            store::delete_rrm_entry(data, rrm_net.cid_short(), &full_id)?;
            println!("Entry '{full_id}' removed from RRM '{}'.", rrm_net.name());
        }
    }
    Ok(())
}

// ── Plugin handler ────────────────────────────────────────────────────────────

fn run_plugin(cmd: PluginCmd, data: &PathBuf) -> Result<()> {
    match cmd {
        PluginCmd::List => {
            let plugins = store::list_plugins(data)?;
            if plugins.is_empty() {
                println!("No plugins installed.");
            } else {
                println!("{:<30} {:<10} {:<8} {}", "ID", "VERSION", "STATUS", "NAME");
                println!("{}", "-".repeat(70));
                for p in &plugins {
                    let lock = if p.manifest.is_system { " [system]" } else { "" };
                    println!("{:<30} {:<10} {:<8} {}{}",
                        p.manifest.id, p.manifest.version,
                        p.state.to_string(), p.manifest.name, lock);
                }
            }
        }

        PluginCmd::Info { id } => {
            let record = store::get_plugin(data, &id)?
                .ok_or_else(|| anyhow::anyhow!("plugin '{}' not found", id))?;
            println!("ID      : {}", record.manifest.id);
            println!("Name    : {}", record.manifest.name);
            println!("Version : {}", record.manifest.version);
            println!("Author  : {}", record.manifest.author);
            println!("Status  : {}{}", record.state, if record.manifest.is_system { " (system)" } else { "" });
            println!("Desc    : {}", record.manifest.description);
            println!("Perms   : {}", record.manifest.permissions.iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", "));
        }

        PluginCmd::Enable { id } => {
            store::set_plugin_state(data, &id, PluginState::Enabled)?;
            println!("Plugin '{id}' enabled.");
        }

        PluginCmd::Disable { id } => {
            store::set_plugin_state(data, &id, PluginState::Disabled)?;
            println!("Plugin '{id}' disabled.");
        }

        PluginCmd::Install { path } => {
            let json = std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("cannot read '{}': {e}", path))?;
            let manifest: PluginManifest = serde_json::from_str(&json)
                .map_err(|e| anyhow::anyhow!("invalid manifest JSON: {e}"))?;
            let id = manifest.id.clone();
            let record = store::install_plugin(data, manifest)?;
            println!("Plugin '{id}' installed (status: {}).", record.state);
            println!("Run `civium plugin enable {id}` to activate it.");
        }
    }
    Ok(())
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
