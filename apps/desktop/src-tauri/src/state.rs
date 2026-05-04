use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tt_chat::ChatClient;
use tt_downloads::DownloadManager;
use tt_storage::Database;

pub struct ChatConnection {
    pub client: ChatClient,
    pub server_id: String,
    pub server_name: String,
    pub url: String,
    pub _join: tokio::task::JoinHandle<()>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    /// Keyed by `server_id` once authenticated, by URL until then.
    pub chats: Arc<Mutex<HashMap<String, ChatConnection>>>,
    pub downloads: Arc<DownloadManager>,
}
