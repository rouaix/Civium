pub mod activitypub;
pub mod activity;
pub mod agenda;
pub mod document;
pub mod e2e;
pub mod pairing;
pub mod cil;
pub mod connection;
pub mod crypto;
pub mod directory;
pub mod error;
pub mod governance;
pub mod identity;
pub mod messaging;
pub mod minor;
pub mod network;
pub(crate) mod time;
// node uses Tokio + native libp2p transports — not available on wasm32
#[cfg(not(target_arch = "wasm32"))]
pub mod node;
pub mod plugin;
pub mod bootstrap;
pub mod rcc;
pub mod revocation;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use connection::{
    AcceptPayload, ConnectionRecord, ConnectionState, RequestPayload, ShareAgreement, ShareTerms,
    SignedRequest,
};
pub use crypto::GroupKey;
pub use error::CiviumError;
pub use identity::{Cid, CiviumKeypair};
pub use messaging::{Mailbox, Message, MessageKind, EncryptedChunk};
pub use directory::{DirectoryEntry, EntryKind, FederatedDirectory, RrmEntry, TrustedRrm};
pub use activity::{ActivityEvent, ActivityKind, Notification};
pub use agenda::AgendaEvent;
pub use document::Document;
pub use e2e::PairKey;
pub use pairing::{PairedDevice, PairingInit, complete_pairing, init_pairing};
pub use minor::{GuardianLink, MinorRestrictions};
pub use network::{Invitation, MemberRecord, MemberRole, NetworkAddress, NetworkKind, PendingRecord, TrustCircle};
pub use governance::{
    add_contest, compute_result, compute_result_with_delegations,
    AdminAction, AdminActionKind, AdminActionStatus,
    Proposal, ProposalStatus, Vote, VoteDelegation, VoteResult,
};
pub use plugin::{CertificationLevel, PluginManifest, PluginPermission, PluginRecord, PluginState, preinstalled_plugins};
pub use cil::{CilAction, check_cil};
pub use bootstrap::{
    CIVIUM_ROOT_NETWORK_CID_FULL, CIVIUM_ROOT_NETWORK_CID_SHORT,
    CIVIUM_ROOT_NETWORK_NAME, CIVIUM_ROOT_NODE_ADDR, root_configured,
};
pub use rcc::{FraudAlert, RccPayload, RCC_PUBLIC_KEY_B58, RCC_URL, verify_rcc_alert};
pub use revocation::RevocationRecord;
pub use activitypub::{ApFollower, ApPost, ApPostResult, ApStatus};

// Node module and P2P types: native only (TCP/QUIC/mDNS do not compile to wasm32).
#[cfg(not(target_arch = "wasm32"))]
pub use libp2p::{Multiaddr, PeerId};
#[cfg(not(target_arch = "wasm32"))]
pub use node::{peer_id_from_multiaddr, CiviumNode, CiviumRequest, CiviumResponse, NodeCommand, NodeConfig, NodeEvent, NodeHandle};
