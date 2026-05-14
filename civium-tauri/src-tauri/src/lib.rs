mod commands;
mod store;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            identity_exists,
            identity_init,
            network_create,
            network_list,
            network_invite,
            network_join,
            member_list,
            member_pending_list,
            member_admit,
            member_reject,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Civium");
}
