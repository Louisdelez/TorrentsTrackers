use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entry::{Category, Language, Quality};
use crate::ids::{PoolId, SourceId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub id: PoolId,
    pub name: String,
    pub description: Option<String>,
    pub members: Vec<SourceId>,
    pub filters: PoolFilters,
    pub dedup_strategy: DedupStrategy,
    pub conflict_strategy: ConflictStrategy,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PoolFilters {
    pub categories: Option<Vec<Category>>,
    #[serde(default)]
    pub tags_required: Vec<String>,
    #[serde(default)]
    pub tags_excluded: Vec<String>,
    pub qualities: Option<Vec<Quality>>,
    pub languages: Option<Vec<Language>>,
    pub size_min: Option<u64>,
    pub size_max: Option<u64>,
    pub seeders_min: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DedupStrategy {
    #[default]
    ByContentId,
    ByMagnetExact,
    None,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ConflictStrategy {
    KeepAll {
        merge_metadata: bool,
    },
    PreferSource(SourceId),
    PreferNewest,
    #[default]
    PreferMostSeeded,
}
