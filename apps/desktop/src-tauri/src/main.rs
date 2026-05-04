// Hide the console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ipc;
mod state;

use std::sync::Arc;

use tracing_subscriber::EnvFilter;
use tt_downloads::DownloadManager;
use tt_storage::Database;

use crate::state::AppState;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_target(false)
        .init();

    let db_path = tt_storage::paths::db_path().expect("locate data dir");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("create data dir");
    }
    let db = Database::open(&db_path).expect("open database");

    let downloads_dir = tt_downloads::manager::default_data_dir().expect("downloads dir");
    let downloads = tauri::async_runtime::block_on(DownloadManager::new(downloads_dir))
        .expect("init download manager");

    let state = AppState {
        db: Arc::new(db),
        chats: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        downloads: Arc::new(downloads),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            ipc::source::list_sources,
            ipc::source::add_source,
            ipc::source::sync_source,
            ipc::source::sync_all_sources,
            ipc::source::remove_source,
            ipc::pool::list_pools,
            ipc::pool::create_pool,
            ipc::pool::remove_pool,
            ipc::search::search,
            ipc::identity::identity_show,
            ipc::identity::identity_init,
            ipc::identity::identity_export,
            ipc::identity::identity_import,
            ipc::identity::identity_forget,
            ipc::publish::publish,
            ipc::magnet::open_magnet,
            ipc::stats::stats,
            ipc::chat::chat_list,
            ipc::chat::chat_connect,
            ipc::chat::chat_disconnect,
            ipc::chat::chat_send,
            ipc::chat::chat_history,
            ipc::downloads::download_add,
            ipc::downloads::download_list,
            ipc::downloads::download_pause,
            ipc::downloads::download_unpause,
            ipc::downloads::download_remove,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
