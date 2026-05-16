use serde::{Deserialize, Serialize};
use crate::error::CiviumError;

/// URL de base du Registre Central Civium — codée en dur dans tous les clients.
pub const RCC_URL: &str = "https://www.rouaix.com/civium";

/// Ed25519 public key of the RCC (base58). Used to verify signed fraud alerts.
/// Replace with the real RCC keypair's public key before production release.
pub const RCC_PUBLIC_KEY_B58: &str = "";

/// A fraud alert broadcast by the RCC over P2P and signed with its Ed25519 key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudAlert {
    pub alert_type: String,
    pub description: String,
    pub network_cids: Vec<String>,
    pub emitted_at: u64,
    pub emitted_by: String,
}

impl FraudAlert {
    /// Canonical bytes over which the RCC signature is computed.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!(
            "{}|{}|{}|{}",
            self.alert_type,
            self.description,
            self.network_cids.join(","),
            self.emitted_at,
        )
        .into_bytes()
    }
}

/// Verify a RCC fraud alert. Returns the deserialized alert if the signature is valid.
///
/// Returns `Err` if the public key is not yet configured, the JSON is malformed,
/// or the Ed25519 signature does not match.
pub fn verify_rcc_alert(payload_json: &str, signature_b58: &str) -> Result<FraudAlert, CiviumError> {
    if RCC_PUBLIC_KEY_B58.is_empty() {
        return Err(CiviumError::Crypto("RCC public key not configured".into()));
    }

    let pub_key_bytes = bs58::decode(RCC_PUBLIC_KEY_B58)
        .into_vec()
        .map_err(|e| CiviumError::Crypto(e.to_string()))?;
    let pub_key_arr: [u8; 32] = pub_key_bytes
        .try_into()
        .map_err(|_| CiviumError::Crypto("invalid RCC public key length".into()))?;
    let pub_key = libp2p::identity::ed25519::PublicKey::try_from_bytes(&pub_key_arr)
        .map_err(|e| CiviumError::Crypto(e.to_string()))?;

    let alert: FraudAlert = serde_json::from_str(payload_json)
        .map_err(|e| CiviumError::Crypto(format!("alert JSON parse error: {e}")))?;

    let sig_bytes = bs58::decode(signature_b58)
        .into_vec()
        .map_err(|e| CiviumError::Crypto(e.to_string()))?;

    if !pub_key.verify(&alert.canonical_bytes(), &sig_bytes) {
        return Err(CiviumError::Crypto("invalid RCC alert signature".into()));
    }

    Ok(alert)
}

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
