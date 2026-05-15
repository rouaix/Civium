use serde::{Deserialize, Serialize};

/// URL de base du Registre Central Civium — codée en dur dans tous les clients.
pub const RCC_URL: &str = "https://www.rouaix.com/civium";

/// Payload envoyé au RCC pour l'enregistrement d'un réseau.
///
/// La signature Ed25519 couvre `canonical_bytes()` — tous les champs sauf
/// `ip` (détecté côté serveur) et `signature` (le résultat lui-même).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RccPayload {
    pub network_cid: String,
    pub network_name: String,
    pub admin_cid: String,
    /// Clé publique Ed25519 de l'admin (base58) — permet au RCC de vérifier la signature.
    pub admin_pubkey: String,
    pub admin_email: String,
    pub registered_at: u64,
}

impl RccPayload {
    pub fn new(
        network_cid: String,
        network_name: String,
        admin_cid: String,
        admin_pubkey: String,
        admin_email: String,
        registered_at: u64,
    ) -> Self {
        Self { network_cid, network_name, admin_cid, admin_pubkey, admin_email, registered_at }
    }

    /// Message canonique à signer : tous les champs joints par `|`, ordre fixe.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!(
            "{}|{}|{}|{}|{}|{}",
            self.network_cid, self.network_name, self.admin_cid,
            self.admin_pubkey, self.admin_email, self.registered_at
        )
        .into_bytes()
    }
}
