use clap::{Parser, Subcommand};
use civium_core::{CiviumKeypair, CiviumNode, NodeConfig};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "civium", about = "Civium protocol CLI — Phase 0")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage the local node
    Node {
        #[command(subcommand)]
        action: NodeCommand,
    },
    /// Manage local identity (keypair / CID)
    Identity {
        #[command(subcommand)]
        action: IdentityCommand,
    },
}

#[derive(Subcommand)]
enum NodeCommand {
    /// Start the local Civium node
    Start {
        /// TCP listen address
        #[arg(long, default_value = "/ip4/0.0.0.0/tcp/0")]
        listen_tcp: String,

        /// QUIC listen address
        #[arg(long, default_value = "/ip4/0.0.0.0/udp/0/quic-v1")]
        listen_quic: String,

        /// Bootstrap peer multiaddr (repeatable)
        #[arg(long = "peer")]
        peers: Vec<String>,

        /// Base58 secret key (use the value printed by `identity generate`)
        #[arg(long, env = "CIVIUM_SECRET")]
        secret: Option<String>,
    },
}

#[derive(Subcommand)]
enum IdentityCommand {
    /// Generate a new keypair and print the CID + secret key
    Generate,
    /// Show the CID for an existing secret key
    Show {
        /// Base58 secret key
        #[arg(long, env = "CIVIUM_SECRET")]
        secret: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Respect RUST_LOG; default to info
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Identity { action } => handle_identity(action),
        Command::Node { action } => handle_node(action).await?,
    }

    Ok(())
}

fn handle_identity(action: IdentityCommand) {
    match action {
        IdentityCommand::Generate => {
            let kp = CiviumKeypair::generate().expect("keypair generation failed");
            let cid = kp.cid();
            println!("CID (short) : {}", cid.short());
            println!("CID (full)  : {}", cid.full());
            println!("Secret key  : {}", kp.secret_b58());
            println!();
            println!("Keep the secret key safe — it is your identity.");
            println!("Pass it with --secret or CIVIUM_SECRET env var to start your node.");
        }
        IdentityCommand::Show { secret } => {
            let kp = CiviumKeypair::from_secret_b58(&secret).expect("invalid secret key");
            let cid = kp.cid();
            println!("CID (short) : {}", cid.short());
            println!("CID (full)  : {}", cid.full());
        }
    }
}

async fn handle_node(action: NodeCommand) -> anyhow::Result<()> {
    match action {
        NodeCommand::Start { listen_tcp, listen_quic, peers, secret } => {
            let keypair = match secret {
                Some(s) => CiviumKeypair::from_secret_b58(&s)?,
                None => {
                    let kp = CiviumKeypair::generate()?;
                    eprintln!("No --secret provided — generated ephemeral identity.");
                    eprintln!("CID : {}", kp.cid().short());
                    eprintln!("To reuse this identity, pass: --secret {}", kp.secret_b58());
                    kp
                }
            };

            let cid = keypair.cid();
            let config = NodeConfig {
                listen_tcp,
                listen_quic,
                bootstrap_peers: peers,
            };

            println!("Starting Civium node");
            println!("  CID        : {}", cid.short());
            println!("  CID (full) : {}", cid.full());

            let mut node = CiviumNode::new(keypair, config).await?;
            node.run().await;
        }
    }
    Ok(())
}
