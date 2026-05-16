mod ap;
mod commands;
mod mcp;
mod node;
mod rcc;
mod root_connect;
mod store;

use commands::*;
use node::AppState;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            node_tx: Mutex::new(None),
            listen_addrs: Mutex::new(Vec::new()),
            mcp_shutdown: Mutex::new(None),
            mcp_token: Mutex::new(None),
            mcp_port: Mutex::new(None),
            active_alerts: Mutex::new(Vec::new()),
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            let data_dir: PathBuf = app_handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("./civium-data"));

            // Start the P2P node in the background if an identity already exists.
            tauri::async_runtime::spawn(async move {
                let conn = match store::open_db(&data_dir) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                if !store::identity_exists(&conn) {
                    return;
                }
                let keypair = match store::load_identity(&conn) {
                    Ok(k) => k,
                    Err(_) => return,
                };
                drop(conn);
                node::start_node(app_handle, keypair, data_dir).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            identity_exists,
            identity_init,
            identity_show,
            network_create,
            network_delete,
            network_list,
            network_invite,
            network_join,
            member_list,
            member_pending_list,
            member_admit,
            member_reject,
            node_status,
            node_sync,
            network_join_p2p,
            message_list,
            message_send,
            message_send_direct,
            message_send_e2e,
            proposal_list,
            proposal_create,
            vote_cast,
            vote_results,
            admin_action_list,
            admin_action_contest,
            vote_delegate,
            vote_revoke_delegation,
            vote_list_delegations,
            directory_create,
            directory_list_networks,
            directory_publish,
            directory_list,
            directory_search,
            directory_remove,
            directory_federate,
            directory_unfederate,
            directory_federations,
            rrm_create,
            rrm_report,
            rrm_list,
            rrm_remove,
            network_trust_rrm,
            network_untrust_rrm,
            network_trusted_rrms,
            rrm_check,
            member_set_minor,
            member_set_guardian,
            member_remove_guardian,
            member_guardians,
            member_wards,
            member_set_restrictions,
            member_get_restrictions,
            plugin_list,
            plugin_enable,
            plugin_disable,
            agenda_create,
            agenda_list,
            agenda_update,
            agenda_delete,
            activity_list,
            notification_list,
            notification_unread_count,
            notification_mark_read,
            document_create,
            document_list,
            document_update,
            document_delete,
            mcp_start,
            mcp_stop,
            mcp_status,
            pair_init,
            pair_complete,
            pair_list,
            pair_revoke,
            outbox_count_all,
            rcc_register,
            rcc_force_retry,
            rcc_mark_registered,
            rcc_status,
            rcc_status_list,
            ap_enable,
            ap_disable,
            ap_status,
            ap_list_followers,
            ap_list_posts,
            ap_post,
            get_active_alerts,
            node_settings_get,
            node_settings_set,
            hub_config_set,
            hub_config_get,
            hub_network_register,
            hub_member_join,
            hub_sync,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Civium");
}
