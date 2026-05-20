mod ap;
mod commands;
mod mcp;
mod node;
mod rcc;
mod root_connect;
mod store;

use civium_core::CiviumKeypair;
use commands::*;
use node::AppState;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Tries to open the database. On failure, attempts to restore the most recent
/// backup from `.backups/`. If recovery succeeds, emits `civium://db-restored`.
/// If recovery fails too, emits `civium://db-error` and returns None.
fn open_db_with_recovery(data_dir: &std::path::Path, app: &tauri::AppHandle) -> Option<Connection> {
    match store::open_db(data_dir) {
        Ok(c) => return Some(c),
        Err(original) => {
            tracing::error!("DB open failed: {original} — attempting backup restore");

            // Find the most recent backup
            let backup_dir = data_dir.join(".backups");
            let restored = (|| -> anyhow::Result<()> {
                let mut entries: Vec<_> = std::fs::read_dir(&backup_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_name().to_string_lossy().ends_with(".db")
                    })
                    .collect();
                entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
                let latest = entries.into_iter().next()
                    .ok_or_else(|| anyhow::anyhow!("no backup found"))?;
                std::fs::copy(latest.path(), data_dir.join("civium.db"))?;
                Ok(())
            })();

            if restored.is_ok() {
                match store::open_db(data_dir) {
                    Ok(c) => {
                        let _ = app.emit("civium://db-restored", ());
                        return Some(c);
                    }
                    Err(e2) => {
                        let _ = app.emit("civium://db-error", e2.to_string());
                    }
                }
            } else {
                let _ = app.emit("civium://db-error", original.to_string());
            }
            None
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_deep_link::init())
        .manage(AppState {
            node_tx: Mutex::new(None),
            listen_addrs: Mutex::new(Vec::new()),
            mcp_shutdown: Mutex::new(None),
            mcp_token: Mutex::new(None),
            mcp_port: Mutex::new(None),
            active_alerts: Mutex::new(Vec::new()),
            log_guard: Mutex::new(None),
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            let data_dir: PathBuf = app_handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("./civium-data"));

            // Init file-based structured logging (daily rotation, 7-day retention).
            {
                let log_dir = data_dir.join("logs");
                let _ = std::fs::create_dir_all(&log_dir);
                let file_appender = tracing_appender::rolling::daily(&log_dir, "civium.log");
                let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
                tracing_subscriber::registry()
                    .with(EnvFilter::new("info"))
                    .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
                    .init();
                // Keep the guard alive for the process lifetime.
                *app_handle.state::<AppState>().log_guard.lock().unwrap() = Some(guard);
            }

            // Periodic DB backup: on startup, then every 6 hours.
            {
                let backup_dir = data_dir.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = commands::perform_backup(&backup_dir);
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(6 * 3600)).await;
                        let _ = commands::perform_backup(&backup_dir);
                    }
                });
            }

            // Start the P2P node with watchdog — restarts automatically after a crash.
            tauri::async_runtime::spawn(async move {
                let conn = match open_db_with_recovery(&data_dir, &app_handle) {
                    Some(c) => c,
                    None => return,
                };
                if !store::identity_exists(&conn) {
                    return;
                }
                // Store secret_b58 so we can re-derive the keypair on each restart
                // (CiviumKeypair is not Clone by design — it zeroes secret on drop).
                let secret_b58 = match store::load_secret_b58(&conn) {
                    Ok(s) => s,
                    Err(_) => return,
                };
                drop(conn);

                // Watchdog: restart the node on crash with exponential backoff.
                let mut backoff_secs: u64 = 5;
                loop {
                    let keypair = match CiviumKeypair::from_secret_b58(&secret_b58) {
                        Ok(k) => k,
                        Err(_) => return,
                    };
                    node::start_node(app_handle.clone(), keypair, data_dir.clone()).await;
                    // start_node returned — the node stopped (crash or clean shutdown).
                    let _ = tauri::Emitter::emit(&app_handle, "civium://node-crashed", ());
                    tracing::warn!("[civium] P2P node stopped — restarting in {backoff_secs}s");
                    tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
                    backoff_secs = (backoff_secs * 2).min(300);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            identity_exists,
            identity_init,
            identity_show,
            identity_restore_from_secret,
            identity_backup_export,
            identity_backup_import,
            db_backup_now,
            db_backup_last,
            export_data,
            network_create,
            network_delete,
            network_leave,
            wipe_all_data,
            network_list,
            network_invite,
            network_join,
            member_list,
            member_pending_list,
            member_admit,
            member_reject,
            member_set_role,
            member_change_circle,
            member_remove,
            node_status,
            node_sync,
            network_join_p2p,
            message_list,
            message_list_paged,
            message_delete,
            message_report,
            message_send,
            message_send_direct,
            message_send_e2e,
            message_send_file,
            message_send_file_path,
            message_get_file,
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
            agenda_export_ics,
            activity_list,
            activity_list_all,
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
            alert_dismiss,
            poll_hub_alerts,
            node_settings_get,
            node_settings_set,
            profile_email_get,
            profile_email_set,
            invitation_list,
            invitation_revoke,
            hub_config_set,
            hub_config_get,
            hub_network_register,
            hub_member_join,
            hub_sync,
            hub_public_networks,
            hub_main_network,
            hub_join_public_network,
            logs_get,
            connection_list,
            connection_accept,
            connection_refuse,
            connection_block,
            connection_revoke,
            member_mute,
            member_unmute,
            member_muted_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Civium");
}
