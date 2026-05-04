use tokio::sync::broadcast;
use tt_chat::ChatMessage;

use crate::config::Config;
use crate::db::Database;

pub struct AppState {
    pub config: Config,
    pub db: Database,
    pub broadcast: broadcast::Sender<ChatMessage>,
}
