//! JSON-friendly mirrors of the workspace types. UUIDs are stringified,
//! datetimes are RFC 3339 strings, and binary keys are hex.

use serde::{Deserialize, Serialize};
use tt_core::{Category, ContentLink, Entry, Language, Pool, Quality, Source, SourceKind};
use tt_storage::{LocalIdentity, SearchHit};

#[derive(Debug, Serialize)]
pub struct SourceDto {
    pub id: String,
    pub kind: SourceKind,
    pub endpoint: String,
    pub display_name: String,
    pub description: Option<String>,
    pub last_sync: Option<String>,
    pub last_status: String,
    pub trust_level: tt_core::TrustLevel,
}

impl From<Source> for SourceDto {
    fn from(s: Source) -> Self {
        Self {
            id: s.id.0.to_string(),
            kind: s.kind,
            endpoint: s.endpoint,
            display_name: s.display_name,
            description: s.description,
            last_sync: s.last_sync.map(|t| t.to_rfc3339()),
            last_status: serde_json::to_string(&s.last_status).unwrap_or_else(|_| "{}".into()),
            trust_level: s.trust_level,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PoolDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_ids: Vec<String>,
    pub created_at: String,
}

impl From<Pool> for PoolDto {
    fn from(p: Pool) -> Self {
        Self {
            id: p.id.0.to_string(),
            name: p.name,
            description: p.description,
            member_ids: p.members.iter().map(|s| s.0.to_string()).collect(),
            created_at: p.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct IdentityDto {
    pub npub: String,
    pub pubkey_hex: String,
    pub display_name: Option<String>,
    pub created_at: String,
}

impl IdentityDto {
    pub fn from_local(li: LocalIdentity, npub: String) -> Self {
        Self {
            npub,
            pubkey_hex: hex::encode(li.pubkey.0),
            display_name: li.display_name,
            created_at: li.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StatsDto {
    pub data_dir: String,
    pub db_path: String,
    pub sources: usize,
    pub pools: usize,
    pub entries: i64,
}

#[derive(Debug, Serialize)]
pub struct SearchHitDto {
    pub id: String,
    pub title: String,
    pub magnet: Option<String>,
    pub category: Category,
    pub tags: Vec<String>,
    pub quality: Option<Quality>,
    pub languages: Vec<Language>,
    pub size_bytes: Option<u64>,
    pub seeders: Option<u32>,
    pub leechers: Option<u32>,
    pub added_at: String,
    pub contributor_pubkey_hex: String,
    pub provenance: Vec<String>,
    pub description: Option<String>,
}

impl From<SearchHit> for SearchHitDto {
    fn from(h: SearchHit) -> Self {
        let e: Entry = h.entry;
        let magnet = match &e.link {
            ContentLink::Magnet(s) => Some(s.clone()),
            ContentLink::TorrentUrl(s) => Some(s.clone()),
            ContentLink::InfoHash(b) => Some(tt_core::magnet::build_magnet(b, Some(&e.title))),
        };
        Self {
            id: e.id.as_hex(),
            title: e.title,
            magnet,
            category: e.category,
            tags: e.tags,
            quality: e.quality,
            languages: e.languages,
            size_bytes: e.size_bytes,
            seeders: e.seeders,
            leechers: e.leechers,
            added_at: e.added_at.to_rfc3339(),
            contributor_pubkey_hex: hex::encode(e.contributor_pubkey.0),
            provenance: h.provenance.iter().map(|s| s.0.to_string()).collect(),
            description: e.description,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ScopeDto {
    All,
    Source { id: String },
    Pool { id: String },
}

#[derive(Debug, Deserialize)]
pub struct SearchQueryDto {
    pub text: Option<String>,
    pub scope: ScopeDto,
    pub categories: Option<Vec<Category>>,
    pub qualities: Option<Vec<Quality>>,
    pub languages: Option<Vec<Language>>,
    pub size_min: Option<u64>,
    pub size_max: Option<u64>,
    pub seeders_min: Option<u32>,
    pub limit: Option<usize>,
}
