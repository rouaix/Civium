use serde::Serialize;
use tauri::{AppHandle, Manager};

use civium_core::{network::Network, CiviumKeypair};

use crate::store;

// ── Return types (serialized to JSON for the frontend) ────────────────────────

#[derive(Serialize)]
pub struct IdentityInfo {
    pub cid_short: String,
    pub cid_full: String,
    pub secret_b58: String,
}

#[derive(Serialize)]
pub struct NetworkInfo {
    pub cid_short: String,
    pub cid_full: String,
    pub name: String,
    pub member_count: usize,
}

#[derive(Serialize)]
pub struct MemberInfo {
    pub cid_short: String,
    pub display_name: String,
    pub circle: u8,
    pub role: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn data_dir(app: &AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("./civium-data"))
}

fn open(app: &AppHandle) -> Result<rusqlite::Connection, String> {
    store::open_db(&data_dir(app)).map_err(|e| e.to_string())
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn identity_exists(app: AppHandle) -> bool {
    open(&app).map(|c| store::identity_exists(&c)).unwrap_or(false)
}

#[tauri::command]
pub fn identity_init(app: AppHandle) -> Result<IdentityInfo, String> {
    let conn = open(&app)?;
    if store::identity_exists(&conn) {
        return Err("identity already exists — use identity_show".into());
    }
    let keypair = CiviumKeypair::generate().map_err(|e| e.to_string())?;
    let cid = keypair.cid();
    store::save_identity(&conn, &keypair).map_err(|e| e.to_string())?;
    Ok(IdentityInfo {
        cid_short: cid.short().to_string(),
        cid_full: cid.full().to_string(),
        secret_b58: keypair.secret_b58(),
    })
}

#[tauri::command]
pub fn network_create(
    app: AppHandle,
    name: String,
    display_name: String,
) -> Result<NetworkInfo, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let admin_cid = keypair.cid();

    let network = Network::create(name, &admin_cid, display_name)
        .map_err(|e| e.to_string())?;

    let info = NetworkInfo {
        cid_short: network.cid_short().to_string(),
        cid_full: network.cid_full().to_string(),
        name: network.name().to_string(),
        member_count: network.data.members.len(),
    };

    store::save_network(&conn, &network).map_err(|e| e.to_string())?;
    Ok(info)
}

#[tauri::command]
pub fn network_list(app: AppHandle) -> Result<Vec<NetworkInfo>, String> {
    let conn = open(&app)?;
    let networks = store::list_networks(&conn).map_err(|e| e.to_string())?;
    Ok(networks
        .iter()
        .map(|n| NetworkInfo {
            cid_short: n.cid_short().to_string(),
            cid_full: n.cid_full().to_string(),
            name: n.name().to_string(),
            member_count: n.data.members.len(),
        })
        .collect())
}

#[tauri::command]
pub fn network_invite(
    app: AppHandle,
    network_cid: String,
    expires_in: u64,
) -> Result<String, String> {
    let conn = open(&app)?;
    let keypair = store::load_identity(&conn).map_err(|e| e.to_string())?;
    let inviter_cid = keypair.cid();
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    network
        .create_invitation(&inviter_cid, expires_in)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn member_list(
    app: AppHandle,
    network_cid: String,
) -> Result<Vec<MemberInfo>, String> {
    let conn = open(&app)?;
    let network = store::load_network(&conn, &network_cid).map_err(|e| e.to_string())?;
    Ok(store::network_members(&network)
        .iter()
        .map(|m| MemberInfo {
            cid_short: m.cid_short.clone(),
            display_name: m.display_name.clone(),
            circle: m.circle as u8,
            role: m.role.to_string(),
        })
        .collect())
}
