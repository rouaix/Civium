/// Civium Identifier — derived from an Ed25519 public key via BLAKE3.
///
/// Format: "civ1" + base58(blake3(pub_key_bytes))
/// Short:  "civ1" + first 8 chars of the base58 string
///
/// Network address: <member_short>@<network_short>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cid {
    raw: [u8; 32],
    full: String,
    short: String,
}

const PREFIX: &str = "civ1";
const SHORT_SUFFIX_LEN: usize = 8;

impl Cid {
    /// Parse a Cid from its full string representation (`civ1…`).
    pub fn from_full(full: &str) -> Result<Self, crate::CiviumError> {
        let b58 = full.strip_prefix(PREFIX).ok_or_else(|| {
            crate::CiviumError::Identity(format!("invalid CID '{full}': missing 'civ1' prefix"))
        })?;
        let bytes = bs58::decode(b58).into_vec().map_err(|e| {
            crate::CiviumError::Identity(format!("invalid CID base58: {e}"))
        })?;
        let raw: [u8; 32] = bytes.try_into().map_err(|_| {
            crate::CiviumError::Identity("invalid CID: expected 32 bytes".into())
        })?;
        let short_suffix = &b58[..SHORT_SUFFIX_LEN.min(b58.len())];
        Ok(Self {
            raw,
            full: full.to_string(),
            short: format!("{PREFIX}{short_suffix}"),
        })
    }

    pub fn from_public_key_bytes(pub_key: &[u8; 32]) -> Self {
        let raw: [u8; 32] = *blake3::hash(pub_key).as_bytes();
        let b58 = bs58::encode(raw).into_string();
        let short_suffix = &b58[..SHORT_SUFFIX_LEN.min(b58.len())];
        Self {
            raw,
            full: format!("{PREFIX}{b58}"),
            short: format!("{PREFIX}{short_suffix}"),
        }
    }

    pub fn full(&self) -> &str {
        &self.full
    }

    pub fn short(&self) -> &str {
        &self.short
    }

    pub fn raw_bytes(&self) -> &[u8; 32] {
        &self.raw
    }
}

impl std::fmt::Display for Cid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.short)
    }
}
