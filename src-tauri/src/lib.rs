mod commands;
mod index;
mod models;
mod parse;
mod session;
mod settings;
mod state;
mod store;
mod vault;
mod watcher;

use std::sync::Mutex;

use tauri::Manager;

use crate::commands::PendingUpdate;
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            std::fs::create_dir_all(&dir)?;
            let settings_path = dir.join("settings.json");
            let loaded = settings::load(&settings_path);
            app.manage(AppState::new(loaded, settings_path));
            app.manage(PendingUpdate {
                inner: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings_get,
            commands::settings_set,
            commands::session_get,
            commands::vault_open,
            commands::vault_create,
            commands::tree_list,
            commands::note_read,
            commands::note_write,
            commands::note_create,
            commands::folder_create,
            commands::path_rename,
            commands::path_delete,
            commands::note_set_properties,
            commands::search_query,
            commands::note_summaries,
            commands::backlinks_for,
            commands::tags_list,
            commands::graph_data,
            commands::resolve_link,
            commands::daily_note_open,
            commands::bookmarks_list,
            commands::bookmarks_toggle,
            commands::parse_note_body,
            commands::update_check,
            commands::update_install
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
