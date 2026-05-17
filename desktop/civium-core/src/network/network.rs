use crate::{crypto::GroupKey, Cid, CiviumError, CiviumKeypair};
use serde::{Deserialize, Serialize};

use super::{
    invitation::Invitation,
    member::{MemberRecord, MemberRole, PendingRecord, TrustCircle},
};

/// The network-scoped address of a member: `<member_short>@<network_short>`.
///
/// Used for display and for cross-network referencing.
#[derive(Debug, Clone)]
pub struct NetworkAddress {
    pub member_short: String,
    pub network_short: String,
}

impl std::fmt::Display for NetworkAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.member_short, self.network_short)
    }
}

/// Whether a network acts as a general-purpose group, a public directory, or a malicious-network registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NetworkKind {
    #[default]
    Standard,
    Directory,
    Rrm,
}

/// Serializable snapshot of a Network — persisted to JSON (Phase 0).
/// SQLCipher replaces this in weeks 9-10.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkData {
    pub network_secret_b58: String,
    pub cid_short: String,
    pub cid_full: String,
    pub name: String,
    pub members: Vec<MemberRecord>,
    pub pending: Vec<PendingRecord>,
    /// ChaCha20-Poly1305 group key (base58). Empty on networks created before week 5.
    #[serde(default)]
    pub group_key_b58: String,
    /// Standard (default) or Directory — backward-compatible (missing = Standard).
    #[serde(default)]
    pub kind: NetworkKind,
    /// Fédération ActivityPub activée pour ce réseau.
    #[serde(default)]
    pub ap_enabled: bool,
    /// URL de l'acteur ActivityPub (défini par le RCC après activation).
    #[serde(default)]
    pub ap_actor_url: Option<String>,
    /// Réseau public (découvrable via annuaire) ou privé (invitation uniquement).
    #[serde(default)]
    pub is_public: bool,
    /// CID complet du réseau parent (réseaux de réseaux). None = réseau racine.
    #[serde(default)]
    pub parent_cid: Option<String>,
}

/// A Civium network — a sovereign group with its own identity, rules and members.
///
/// The network has its own Ed25519 keypair (owned by the founding admin) from which
/// its CID is derived. All invitations are signed with this keypair.
pub struct Network {
    keypair: CiviumKeypair,
    pub data: NetworkData,
}

impl Network {
    /// Create a new network. Generates a fresh keypair and group key for the network.
    /// The founding admin is added as the first member.
    pub fn create(
        name: String,
        admin_cid: &Cid,
        admin_display_name: String,
        admin_pub_key_b58: Option<String>,
        is_public: bool,
        parent_cid: Option<String>,
    ) -> Result<Self, CiviumError> {
        let keypair = CiviumKeypair::generate()?;
        let cid = keypair.cid();
        let group_key = GroupKey::generate();

        let admin = MemberRecord {
            cid_short: admin_cid.short().to_string(),
            cid_full: admin_cid.full().to_string(),
            display_name: admin_display_name,
            circle: TrustCircle::Confiance,
            role: MemberRole::Admin,
            joined_at: unix_now(),
            is_minor: false,
            pub_key_b58: admin_pub_key_b58,
        };

        let data = NetworkData {
            network_secret_b58: keypair.secret_b58(),
            cid_short: cid.short().to_string(),
            cid_full: cid.full().to_string(),
            name,
            members: vec![admin],
            pending: vec![],
            group_key_b58: group_key.to_b58(),
            kind: NetworkKind::Standard,
            ap_enabled: false,
            ap_actor_url: None,
            is_public,
            parent_cid,
        };

        Ok(Self { keypair, data })
    }

    /// Restore a Network from its persisted data.
    pub fn from_data(data: NetworkData) -> Result<Self, CiviumError> {
        let keypair = CiviumKeypair::from_secret_b58(&data.network_secret_b58)?;
        Ok(Self { keypair, data })
    }

    pub fn cid_short(&self) -> &str {
        &self.data.cid_short
    }

    pub fn cid_full(&self) -> &str {
        &self.data.cid_full
    }

    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn keypair(&self) -> &CiviumKeypair {
        &self.keypair
    }

    /// Base58-encoded Ed25519 public key of this network's keypair.
    pub fn pubkey_b58(&self) -> String {
        bs58::encode(self.keypair.public_key_bytes()).into_string()
    }

    /// Generate a signed invitation link.
    pub fn create_invitation(
        &self,
        inviter_cid: &Cid,
        expires_hours: u64,
    ) -> Result<String, CiviumError> {
        let invite = Invitation::create(&self.keypair, &self.data.name, inviter_cid, expires_hours)?;
        invite.to_link()
    }

    /// Record a pending join request from an invitation link.
    /// Returns an error if the invitation is invalid or already used.
    pub fn submit_join_request(
        &mut self,
        member_cid: &Cid,
        display_name: String,
        invite: &Invitation,
        pub_key_b58: Option<String>,
    ) -> Result<(), CiviumError> {
        invite.verify()?;

        // Ensure the invite is for this network
        if invite.network_cid_full() != self.data.cid_full {
            return Err(CiviumError::Network(
                "invitation is for a different network".into(),
            ));
        }

        // No duplicate pending requests
        if self.data.pending.iter().any(|p| p.cid_full == member_cid.full()) {
            return Err(CiviumError::Network(format!(
                "a pending request already exists for {}",
                member_cid.short()
            )));
        }

        // Not already a member
        if self.data.members.iter().any(|m| m.cid_full == member_cid.full()) {
            return Err(CiviumError::Network(format!(
                "{} is already a member",
                member_cid.short()
            )));
        }

        // Display name unique among members (not checked against pending — admin resolves)
        if self
            .data
            .members
            .iter()
            .any(|m| m.display_name == display_name)
        {
            return Err(CiviumError::Network(format!(
                "display name '{display_name}' is already taken in this network"
            )));
        }

        self.data.pending.push(PendingRecord {
            cid_short: member_cid.short().to_string(),
            cid_full: member_cid.full().to_string(),
            display_name,
            requested_at: unix_now(),
            invite_nonce_b58: invite.nonce_b58().to_string(),
            pub_key_b58,
        });

        Ok(())
    }

    /// Admit a pending member. The admin picks the circle and role.
    pub fn admit(
        &mut self,
        member_cid_short: &str,
        circle: TrustCircle,
        role: MemberRole,
    ) -> Result<MemberRecord, CiviumError> {
        let idx = self
            .data
            .pending
            .iter()
            .position(|p| p.cid_short == member_cid_short)
            .ok_or_else(|| {
                CiviumError::Network(format!("no pending request for {member_cid_short}"))
            })?;

        let pending = self.data.pending.remove(idx);

        // Check display name uniqueness among admitted members
        if self
            .data
            .members
            .iter()
            .any(|m| m.display_name == pending.display_name)
        {
            return Err(CiviumError::Network(format!(
                "display name '{}' is already taken — ask the member to choose another",
                pending.display_name
            )));
        }

        let record = MemberRecord {
            cid_short: pending.cid_short,
            cid_full: pending.cid_full,
            display_name: pending.display_name,
            circle,
            role,
            joined_at: unix_now(),
            is_minor: false,
            pub_key_b58: pending.pub_key_b58,
        };

        self.data.members.push(record.clone());
        Ok(record)
    }

    /// Change the role of an admitted member (admin-only operation).
    pub fn set_member_role(&mut self, member_cid_short: &str, role: MemberRole) -> Result<(), CiviumError> {
        let m = self
            .data
            .members
            .iter_mut()
            .find(|m| m.cid_short == member_cid_short)
            .ok_or_else(|| CiviumError::Network(format!("member {member_cid_short} not found")))?;
        m.role = role;
        Ok(())
    }

    /// Remove an admitted member from the network.
    pub fn remove_member(&mut self, member_cid_short: &str) -> Result<(), CiviumError> {
        let idx = self
            .data
            .members
            .iter()
            .position(|m| m.cid_short == member_cid_short)
            .ok_or_else(|| CiviumError::Network(format!("member {member_cid_short} not found")))?;
        self.data.members.remove(idx);
        Ok(())
    }

    /// Reject a pending join request.
    pub fn reject(&mut self, member_cid_short: &str) -> Result<(), CiviumError> {
        let idx = self
            .data
            .pending
            .iter()
            .position(|p| p.cid_short == member_cid_short)
            .ok_or_else(|| {
                CiviumError::Network(format!("no pending request for {member_cid_short}"))
            })?;
        self.data.pending.remove(idx);
        Ok(())
    }

    /// Network-scoped address for a member CID.
    pub fn address_for(&self, member_cid: &Cid) -> NetworkAddress {
        NetworkAddress {
            member_short: member_cid.short().to_string(),
            network_short: self.data.cid_short.clone(),
        }
    }
}

fn unix_now() -> u64 { crate::time::unix_now() }
