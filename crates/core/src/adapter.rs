use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::community::{Ban, CommunityMetadata};
use crate::entry::Entry;
use crate::error::Result;
use crate::source::SourceKind;

#[async_trait]
pub trait SourceAdapter: Send + Sync {
    fn kind(&self) -> SourceKind;

    fn capabilities(&self) -> SourceCapabilities;

    async fn fetch_entries(&self, since: Option<DateTime<Utc>>) -> Result<Vec<Entry>>;

    async fn fetch_metadata(&self) -> Result<CommunityMetadata>;

    async fn publish_entry(&self, entry: &Entry) -> Result<()>;

    async fn fetch_bans(&self) -> Result<Vec<Ban>>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceCapabilities {
    pub read: bool,
    pub write: bool,
    pub watch: bool,
    pub incremental_sync: bool,
    pub authenticated: bool,
}
