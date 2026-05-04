use std::sync::Arc;

use tt_storage::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}
