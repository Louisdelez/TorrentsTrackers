use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entry::PublicKeyBytes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityMetadata {
    pub display_name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    #[serde(default)]
    pub modo_pubkeys: Vec<PublicKeyBytes>,
    pub rules: Option<String>,
    pub language: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub member_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ban {
    pub pubkey: PublicKeyBytes,
    pub reason: Option<String>,
    pub banned_at: DateTime<Utc>,
    pub banned_by: PublicKeyBytes,
}
