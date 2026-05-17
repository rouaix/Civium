//! wasm-bindgen bindings — JS API for the Civium web client.
//!
//! The API is stateless: Rust transforms data and returns JSON; the JS layer
//! (IndexedDB) owns all persistence.  Build with:
//!   wasm-pack build --target web -- --features wasm
#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Serialize a Rust value to a JS value via JSON round-trip.
fn to_js<T: serde::Serialize>(val: &T) -> Result<JsValue, JsError> {
    let json = serde_json::to_string(val).map_err(|e| JsError::new(&e.to_string()))?;
    js_sys::JSON::parse(&json).map_err(|e| JsError::new(&format!("{e:?}")))
}

fn js_err(msg: impl std::fmt::Display) -> JsError {
    JsError::new(&msg.to_string())
}

// ── Version ───────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn civium_version() -> String {
    format!("{} (wasm)", env!("CARGO_PKG_VERSION"))
}

// ── Identity ──────────────────────────────────────────────────────────────────

/// Generate a new random Ed25519 identity.
///
/// Returns `{ cid_full, cid_short, secret_b58, pub_key_b58 }`.
/// Store `secret_b58` securely — it is the only way to recover the identity.
#[wasm_bindgen]
pub fn generate_identity() -> Result<JsValue, JsError> {
    let kp = crate::CiviumKeypair::generate().map_err(|e| js_err(e))?;
    let cid = kp.cid();
    to_js(&serde_json::json!({
        "cid_full":   cid.full(),
        "cid_short":  cid.short(),
        "secret_b58": kp.secret_b58(),
        "pub_key_b58": kp.pub_key_b58(),
    }))
}

/// Restore an identity from its base58-encoded secret key.
///
/// Returns `{ cid_full, cid_short, pub_key_b58 }`.
#[wasm_bindgen]
pub fn load_identity(secret_b58: &str) -> Result<JsValue, JsError> {
    let kp = crate::CiviumKeypair::from_secret_b58(secret_b58).map_err(|e| js_err(e))?;
    let cid = kp.cid();
    to_js(&serde_json::json!({
        "cid_full":   cid.full(),
        "cid_short":  cid.short(),
        "pub_key_b58": kp.pub_key_b58(),
    }))
}

// ── Group key (symmetric crypto) ──────────────────────────────────────────────

/// Generate a new random group key. Returns its base58 representation.
#[wasm_bindgen]
pub fn group_key_generate() -> String {
    crate::GroupKey::generate().to_b58()
}

/// Encrypt `plaintext` with the group key.
///
/// Returns `{ nonce_b58, ciphertext_b58 }`.
#[wasm_bindgen]
pub fn group_key_encrypt(group_key_b58: &str, plaintext: &str) -> Result<JsValue, JsError> {
    let gk = crate::GroupKey::from_b58(group_key_b58).map_err(|e| js_err(e))?;
    let (nonce_b58, ciphertext_b58) = gk.encrypt(plaintext.as_bytes()).map_err(|e| js_err(e))?;
    to_js(&serde_json::json!({ "nonce_b58": nonce_b58, "ciphertext_b58": ciphertext_b58 }))
}

/// Decrypt a message encrypted by `group_key_encrypt`. Returns the plaintext string.
#[wasm_bindgen]
pub fn group_key_decrypt(group_key_b58: &str, nonce_b58: &str, ciphertext_b58: &str) -> Result<String, JsError> {
    let gk = crate::GroupKey::from_b58(group_key_b58).map_err(|e| js_err(e))?;
    let bytes = gk.decrypt(nonce_b58, ciphertext_b58).map_err(|e| js_err(e))?;
    String::from_utf8(bytes).map_err(|e| js_err(e))
}

// ── Network ───────────────────────────────────────────────────────────────────

/// Create a new Civium network.
///
/// Returns the full `NetworkData` JSON — persist it to IndexedDB.
#[wasm_bindgen]
pub fn network_create(name: &str, admin_secret_b58: &str, admin_display_name: &str) -> Result<JsValue, JsError> {
    let admin_kp = crate::CiviumKeypair::from_secret_b58(admin_secret_b58).map_err(|e| js_err(e))?;
    let admin_cid = admin_kp.cid();
    let pub_key_b58 = admin_kp.pub_key_b58();
    let net = crate::network::Network::create(
        name.to_string(),
        &admin_cid,
        admin_display_name.to_string(),
        Some(pub_key_b58),
        false,
        None,
    ).map_err(|e| js_err(e))?;
    to_js(&net.data)
}

// ── Messaging ─────────────────────────────────────────────────────────────────

/// Build and encrypt a message. The message is NOT stored — the caller persists it.
///
/// `kind`: `"thread"` | `"direct:<cid_short>"` | `"e2e:<cid_full>"`
///
/// Returns a `Message` JSON ready to be stored in IndexedDB and broadcast via P2P.
#[wasm_bindgen]
pub fn message_build(
    author_cid_short: &str,
    kind: &str,
    content: &str,
    group_key_b58: &str,
) -> Result<JsValue, JsError> {
    let gk = crate::GroupKey::from_b58(group_key_b58).map_err(|e| js_err(e))?;
    let msg_kind = parse_message_kind(kind)?;
    let mut mailbox = crate::Mailbox::new();
    let msg = mailbox.post(author_cid_short.to_string(), msg_kind, content, &gk)
        .map_err(|e| js_err(e))?;
    to_js(msg)
}

fn parse_message_kind(kind: &str) -> Result<crate::MessageKind, JsError> {
    if kind == "thread" {
        Ok(crate::MessageKind::Thread)
    } else if let Some(cid) = kind.strip_prefix("direct:") {
        Ok(crate::MessageKind::Direct { to_cid_short: cid.to_string() })
    } else if let Some(cid) = kind.strip_prefix("e2e:") {
        Ok(crate::MessageKind::E2E { to_cid_full: cid.to_string() })
    } else {
        Err(js_err(format!("unknown message kind: {kind}")))
    }
}

/// Decrypt a message body. Returns the plaintext string.
#[wasm_bindgen]
pub fn message_decrypt(nonce_b58: &str, ciphertext_b58: &str, group_key_b58: &str) -> Result<String, JsError> {
    group_key_decrypt(group_key_b58, nonce_b58, ciphertext_b58)
}

// ── Governance ────────────────────────────────────────────────────────────────

/// Create a new governance proposal.
///
/// `options_json`: JSON array of option strings, e.g. `["Oui","Non","Abstention"]`.
/// `closes_at`: Unix timestamp (seconds) when voting closes (0 = 7 days from now).
///
/// Returns a `Proposal` JSON.
#[wasm_bindgen]
pub fn proposal_create(
    title: &str,
    description: &str,
    options_json: &str,
    author_cid_full: &str,
    network_cid_short: &str,
    quorum_percent: u8,
    closes_at: f64,
) -> Result<JsValue, JsError> {
    let options: Vec<String> = serde_json::from_str(options_json)
        .map_err(|e| js_err(format!("options_json: {e}")))?;
    let now = crate::time::unix_now();
    let closes = if closes_at > 0.0 { closes_at as u64 } else { now + 7 * 86_400 };
    let proposal = crate::Proposal::new(
        network_cid_short.to_string(),
        title.to_string(),
        description.to_string(),
        options,
        author_cid_full.to_string(),
        now,
        closes,
        quorum_percent,
    );
    to_js(&proposal)
}

/// Compute the vote result for a proposal.
///
/// `proposals_json`: JSON of a single `Proposal`.
/// `votes_json`: JSON array of `Vote`.
/// `delegations_json`: JSON array of `VoteDelegation` (pass `[]` if none).
/// `total_members`: number of members entitled to vote.
///
/// Returns a `VoteResult` JSON.
#[wasm_bindgen]
pub fn vote_compute(
    proposal_json: &str,
    votes_json: &str,
    delegations_json: &str,
    total_members: usize,
) -> Result<JsValue, JsError> {
    let proposal: crate::Proposal = serde_json::from_str(proposal_json)
        .map_err(|e| js_err(format!("proposal_json: {e}")))?;
    let votes: Vec<crate::Vote> = serde_json::from_str(votes_json)
        .map_err(|e| js_err(format!("votes_json: {e}")))?;
    let delegations: Vec<crate::VoteDelegation> = serde_json::from_str(delegations_json)
        .map_err(|e| js_err(format!("delegations_json: {e}")))?;
    let result = crate::compute_result_with_delegations(&proposal, &votes, &delegations, total_members);
    to_js(&result)
}

// ── Agenda ────────────────────────────────────────────────────────────────────

/// Build a new agenda event. Returns an `AgendaEvent` JSON.
///
/// `start_at` / `end_at`: Unix timestamps in seconds (f64 for JS interop).
/// Pass `0` for `end_at` to leave it unset.
#[wasm_bindgen]
pub fn agenda_event_build(
    title: &str,
    description: &str,
    location: &str,
    start_at: f64,
    end_at: f64,
    network_cid_short: &str,
    created_by: &str,
) -> Result<JsValue, JsError> {
    let event = crate::AgendaEvent::new(
        network_cid_short.to_string(),
        title.to_string(),
        description.to_string(),
        start_at as u64,
        if end_at > 0.0 { Some(end_at as u64) } else { None },
        if location.is_empty() { None } else { Some(location.to_string()) },
        created_by.to_string(),
    );
    to_js(&event)
}

// ── Vote ──────────────────────────────────────────────────────────────────────

/// Cast a vote on a proposal. Returns a `Vote` JSON ready to store and broadcast.
#[wasm_bindgen]
pub fn vote_cast(
    proposal_id: &str,
    voter_cid_short: &str,
    choice_index: usize,
) -> Result<JsValue, JsError> {
    let vote = crate::governance::Vote {
        proposal_id:     proposal_id.to_string(),
        voter_cid_short: voter_cid_short.to_string(),
        choice_index,
        cast_at:         crate::time::unix_now(),
    };
    to_js(&vote)
}

// ── Pairing ───────────────────────────────────────────────────────────────────

/// Decode a `civium://pair/<b58payload>` link and return the `secret_b58`.
/// Used on a new browser session to recover identity from a desktop QR code.
#[wasm_bindgen]
pub fn pairing_complete(link: &str) -> Result<String, JsError> {
    crate::complete_pairing(link).map_err(|e| js_err(e))
}

// ── Documents ─────────────────────────────────────────────────────────────────

/// Build and encrypt a new document. Returns a `Document` JSON.
#[wasm_bindgen]
pub fn document_build(
    title: &str,
    body: &str,
    network_cid_short: &str,
    created_by: &str,
    group_key_b58: &str,
) -> Result<JsValue, JsError> {
    let gk = crate::GroupKey::from_b58(group_key_b58).map_err(|e| js_err(e))?;
    let (nonce_b58, body_ciphertext) = gk.encrypt(body.as_bytes()).map_err(|e| js_err(e))?;
    let doc = crate::Document::new(
        network_cid_short.to_string(),
        title.to_string(),
        nonce_b58,
        body_ciphertext,
        created_by.to_string(),
    );
    to_js(&doc)
}

/// Decrypt a document body. Returns the plaintext string.
#[wasm_bindgen]
pub fn document_decrypt_body(nonce_b58: &str, body_ciphertext: &str, group_key_b58: &str) -> Result<String, JsError> {
    group_key_decrypt(group_key_b58, nonce_b58, body_ciphertext)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_generate_identity_roundtrip() {
        let js = generate_identity().unwrap();
        let obj: serde_json::Value = serde_json::from_str(
            &js_sys::JSON::stringify(&js).unwrap().as_string().unwrap()
        ).unwrap();
        let cid_full = obj["cid_full"].as_str().unwrap();
        let secret_b58 = obj["secret_b58"].as_str().unwrap();
        assert!(cid_full.starts_with("civ1"), "CID must start with civ1");

        // Reload and verify same CID
        let js2 = load_identity(secret_b58).unwrap();
        let obj2: serde_json::Value = serde_json::from_str(
            &js_sys::JSON::stringify(&js2).unwrap().as_string().unwrap()
        ).unwrap();
        assert_eq!(obj2["cid_full"].as_str().unwrap(), cid_full);
    }

    #[wasm_bindgen_test]
    fn test_group_key_roundtrip() {
        let key_b58 = group_key_generate();
        let plaintext = "Bonjour Civium !";
        let enc = group_key_encrypt(&key_b58, plaintext).unwrap();
        let enc_str = js_sys::JSON::stringify(&enc).unwrap().as_string().unwrap();
        let enc_obj: serde_json::Value = serde_json::from_str(&enc_str).unwrap();
        let nonce = enc_obj["nonce_b58"].as_str().unwrap();
        let ciphertext = enc_obj["ciphertext_b58"].as_str().unwrap();
        let decrypted = group_key_decrypt(&key_b58, nonce, ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[wasm_bindgen_test]
    fn test_network_create() {
        let id = generate_identity().unwrap();
        let id_str = js_sys::JSON::stringify(&id).unwrap().as_string().unwrap();
        let id_obj: serde_json::Value = serde_json::from_str(&id_str).unwrap();
        let secret = id_obj["secret_b58"].as_str().unwrap();
        let net = network_create("Réseau Test", secret, "Alice").unwrap();
        let net_str = js_sys::JSON::stringify(&net).unwrap().as_string().unwrap();
        let net_obj: serde_json::Value = serde_json::from_str(&net_str).unwrap();
        assert_eq!(net_obj["name"].as_str().unwrap(), "Réseau Test");
        assert!(!net_obj["group_key_b58"].as_str().unwrap().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_message_build_and_decrypt() {
        let key_b58 = group_key_generate();
        let content = "Premier message Civium";
        let msg_js = message_build("civ1abc", "thread", content, &key_b58).unwrap();
        let msg_str = js_sys::JSON::stringify(&msg_js).unwrap().as_string().unwrap();
        let msg: serde_json::Value = serde_json::from_str(&msg_str).unwrap();
        let nonce = msg["nonce_b58"].as_str().unwrap();
        let ct = msg["ciphertext_b58"].as_str().unwrap();
        let plain = message_decrypt(nonce, ct, &key_b58).unwrap();
        assert_eq!(plain, content);
    }

    #[wasm_bindgen_test]
    fn test_vote_cast() {
        let vote_js = vote_cast("proposal-1", "civ1alice", 0).unwrap();
        let vote_str = js_sys::JSON::stringify(&vote_js).unwrap().as_string().unwrap();
        let vote: serde_json::Value = serde_json::from_str(&vote_str).unwrap();
        assert_eq!(vote["proposal_id"].as_str().unwrap(), "proposal-1");
        assert_eq!(vote["choice_index"].as_u64().unwrap(), 0);
        assert!(vote["cast_at"].as_u64().unwrap() > 0);
    }

    #[wasm_bindgen_test]
    fn test_document_roundtrip() {
        let key_b58 = group_key_generate();
        let doc_js = document_build("Mon titre", "Contenu secret", "net1", "civ1alice", &key_b58).unwrap();
        let doc_str = js_sys::JSON::stringify(&doc_js).unwrap().as_string().unwrap();
        let doc: serde_json::Value = serde_json::from_str(&doc_str).unwrap();
        let nonce = doc["nonce_b58"].as_str().unwrap();
        let ct    = doc["body_ciphertext"].as_str().unwrap();
        let plain = document_decrypt_body(nonce, ct, &key_b58).unwrap();
        assert_eq!(plain, "Contenu secret");
    }
}
