use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::ids::SourceId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: SourceId,
    pub kind: SourceKind,
    pub endpoint: String,
    pub display_name: String,
    pub description: Option<String>,
    pub auth: Option<AuthConfig>,
    pub sync_policy: SyncPolicy,
    pub last_sync: Option<DateTime<Utc>>,
    pub last_status: SyncStatus,
    pub trust_level: TrustLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceKind {
    LocalFolder,
    HttpUrl,
    GitRepo,
    GoogleDrive,
    Dropbox,
    OneDrive,
    Server,
    Nostr,
    Ipfs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    None,
    BearerToken(String),
    Basic { username: String, password: String },
    PublicKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPolicy {
    pub auto_sync: bool,
    #[serde(with = "duration_secs")]
    pub interval: Duration,
    pub bandwidth_limit_kbps: Option<u32>,
}

impl Default for SyncPolicy {
    fn default() -> Self {
        Self {
            auto_sync: true,
            interval: Duration::from_secs(60 * 30),
            bandwidth_limit_kbps: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Idle,
    Syncing { started_at: DateTime<Utc> },
    Success { at: DateTime<Utc>, fetched: usize },
    Failed { at: DateTime<Utc>, error: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    Unverified,
    Trusted,
    Modos,
}

mod duration_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(d.as_secs())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let secs = u64::deserialize(d)?;
        Ok(Duration::from_secs(secs))
    }
}
