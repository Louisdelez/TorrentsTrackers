//! Typed query helpers — insert/select/update for entries, sources, pools.

use chrono::{DateTime, Utc};
use rusqlite::{OptionalExtension, params};
use tt_core::{
    Category, ContentId, ContentLink, Entry, Language, Pool, PoolFilters, PoolId, PublicKeyBytes,
    Quality, SignatureBytes, Source, SourceId, SourceKind, TrustLevel,
};
use uuid::Uuid;

use crate::db::Database;
use crate::error::{Result, StorageError};

// --------------------------------------------------------------------------
// Sources
// --------------------------------------------------------------------------

impl Database {
    pub fn insert_source(&self, source: &Source) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sources (id, kind, endpoint, display_name, description,
                                  auth_json, sync_policy_json, last_sync,
                                  last_status_json, trust_level, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                source.id.0.to_string(),
                source_kind_to_str(source.kind),
                source.endpoint,
                source.display_name,
                source.description,
                source
                    .auth
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?,
                serde_json::to_string(&source.sync_policy)?,
                source.last_sync.map(|t| t.to_rfc3339()),
                serde_json::to_string(&source.last_status)?,
                trust_level_to_str(source.trust_level),
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_sources(&self) -> Result<Vec<Source>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, kind, endpoint, display_name, description,
                    auth_json, sync_policy_json, last_sync, last_status_json, trust_level
             FROM sources
             ORDER BY display_name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], row_to_source)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(StorageError::from)
    }

    pub fn get_source(&self, id: SourceId) -> Result<Option<Source>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, kind, endpoint, display_name, description,
                    auth_json, sync_policy_json, last_sync, last_status_json, trust_level
             FROM sources WHERE id = ?1",
            params![id.0.to_string()],
            row_to_source,
        )
        .optional()
        .map_err(StorageError::from)
    }

    pub fn delete_source(&self, id: SourceId) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let n = conn.execute(
            "DELETE FROM sources WHERE id = ?1",
            params![id.0.to_string()],
        )?;
        if n == 0 {
            return Err(StorageError::NotFound);
        }
        Ok(())
    }

    pub fn update_source_sync_status(
        &self,
        id: SourceId,
        last_sync: DateTime<Utc>,
        status_json: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sources SET last_sync = ?2, last_status_json = ?3 WHERE id = ?1",
            params![id.0.to_string(), last_sync.to_rfc3339(), status_json],
        )?;
        Ok(())
    }
}

fn row_to_source(row: &rusqlite::Row<'_>) -> rusqlite::Result<Source> {
    let id_str: String = row.get(0)?;
    let last_sync: Option<String> = row.get(7)?;
    let auth_json: Option<String> = row.get(5)?;

    Ok(Source {
        id: SourceId(Uuid::parse_str(&id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?),
        kind: source_kind_from_str(&row.get::<_, String>(1)?).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                "unknown SourceKind".into(),
            )
        })?,
        endpoint: row.get(2)?,
        display_name: row.get(3)?,
        description: row.get(4)?,
        auth: auth_json
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
        sync_policy: serde_json::from_str::<tt_core::SyncPolicy>(&row.get::<_, String>(6)?)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    6,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
        last_sync: last_sync
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|t| t.with_timezone(&Utc)),
        last_status: serde_json::from_str::<tt_core::SyncStatus>(&row.get::<_, String>(8)?)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
        trust_level: trust_level_from_str(&row.get::<_, String>(9)?)
            .unwrap_or(TrustLevel::Unverified),
    })
}

fn source_kind_to_str(k: SourceKind) -> &'static str {
    match k {
        SourceKind::LocalFolder => "LocalFolder",
        SourceKind::HttpUrl => "HttpUrl",
        SourceKind::GitRepo => "GitRepo",
        SourceKind::GoogleDrive => "GoogleDrive",
        SourceKind::Dropbox => "Dropbox",
        SourceKind::OneDrive => "OneDrive",
        SourceKind::Server => "Server",
        SourceKind::Nostr => "Nostr",
        SourceKind::Ipfs => "Ipfs",
    }
}

fn source_kind_from_str(s: &str) -> Option<SourceKind> {
    Some(match s {
        "LocalFolder" => SourceKind::LocalFolder,
        "HttpUrl" => SourceKind::HttpUrl,
        "GitRepo" => SourceKind::GitRepo,
        "GoogleDrive" => SourceKind::GoogleDrive,
        "Dropbox" => SourceKind::Dropbox,
        "OneDrive" => SourceKind::OneDrive,
        "Server" => SourceKind::Server,
        "Nostr" => SourceKind::Nostr,
        "Ipfs" => SourceKind::Ipfs,
        _ => return None,
    })
}

fn trust_level_to_str(t: TrustLevel) -> &'static str {
    match t {
        TrustLevel::Unverified => "Unverified",
        TrustLevel::Trusted => "Trusted",
        TrustLevel::Modos => "Modos",
    }
}

fn trust_level_from_str(s: &str) -> Option<TrustLevel> {
    Some(match s {
        "Unverified" => TrustLevel::Unverified,
        "Trusted" => TrustLevel::Trusted,
        "Modos" => TrustLevel::Modos,
        _ => return None,
    })
}

// --------------------------------------------------------------------------
// Entries
// --------------------------------------------------------------------------

impl Database {
    /// Upsert an entry (insert if new, update if same `id`) and link it to a
    /// source. Returns `true` if the entry was newly inserted.
    pub fn upsert_entry(&self, entry: &Entry, source_id: SourceId) -> Result<bool> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let now = Utc::now().to_rfc3339();
        let (kind, value) = link_to_columns(&entry.link);

        let existed: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM entries WHERE id = ?1)",
            params![entry.id.as_hex()],
            |r| r.get::<_, i64>(0).map(|v| v != 0),
        )?;

        tx.execute(
            "INSERT INTO entries (
                id, title, link_kind, link_value, category, tags_json, quality_json,
                languages_json, size_bytes, seeders, leechers, added_at,
                contributor_pubkey, signature, description, poster_url,
                first_seen_at, primary_source_id
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                link_kind = excluded.link_kind,
                link_value = excluded.link_value,
                category = excluded.category,
                tags_json = excluded.tags_json,
                quality_json = excluded.quality_json,
                languages_json = excluded.languages_json,
                size_bytes = excluded.size_bytes,
                seeders = excluded.seeders,
                leechers = excluded.leechers,
                description = excluded.description,
                poster_url = excluded.poster_url",
            params![
                entry.id.as_hex(),
                entry.title,
                kind,
                value,
                category_to_str(entry.category),
                serde_json::to_string(&entry.tags)?,
                entry
                    .quality
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?,
                serde_json::to_string(&entry.languages)?,
                entry.size_bytes,
                entry.seeders,
                entry.leechers,
                entry.added_at.to_rfc3339(),
                &entry.contributor_pubkey.0[..],
                &entry.signature.0[..],
                entry.description,
                entry.poster_url,
                now,
                entry.source_id.0.to_string(),
            ],
        )?;

        tx.execute(
            "INSERT INTO entry_sources (entry_id, source_id, seen_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(entry_id, source_id) DO UPDATE SET seen_at = excluded.seen_at",
            params![entry.id.as_hex(), source_id.0.to_string(), now],
        )?;

        tx.commit()?;
        Ok(!existed)
    }

    pub fn count_entries(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        Ok(conn.query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))?)
    }
}

fn link_to_columns(link: &ContentLink) -> (&'static str, String) {
    match link {
        ContentLink::Magnet(s) => ("Magnet", s.clone()),
        ContentLink::TorrentUrl(s) => ("TorrentUrl", s.clone()),
        ContentLink::InfoHash(b) => ("InfoHash", hex::encode(b)),
    }
}

fn link_from_columns(kind: &str, value: &str) -> Result<ContentLink> {
    Ok(match kind {
        "Magnet" => ContentLink::Magnet(value.to_string()),
        "TorrentUrl" => ContentLink::TorrentUrl(value.to_string()),
        "InfoHash" => {
            let bytes = hex::decode(value)
                .map_err(|e| StorageError::Invalid(format!("info_hash hex: {e}")))?;
            if bytes.len() != 20 {
                return Err(StorageError::Invalid(format!(
                    "info_hash has {} bytes, expected 20",
                    bytes.len()
                )));
            }
            let mut arr = [0u8; 20];
            arr.copy_from_slice(&bytes);
            ContentLink::InfoHash(arr)
        }
        other => return Err(StorageError::Invalid(format!("unknown link_kind {other}"))),
    })
}

fn category_to_str(c: Category) -> &'static str {
    match c {
        Category::Films => "Films",
        Category::Series => "Series",
        Category::Games => "Games",
        Category::Music => "Music",
        Category::Books => "Books",
        Category::Software => "Software",
        Category::Other => "Other",
    }
}

fn category_from_str(s: &str) -> Category {
    match s {
        "Films" => Category::Films,
        "Series" => Category::Series,
        "Games" => Category::Games,
        "Music" => Category::Music,
        "Books" => Category::Books,
        "Software" => Category::Software,
        _ => Category::Other,
    }
}

// --------------------------------------------------------------------------
// Search
// --------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub scope: SearchScope,
    pub categories: Option<Vec<Category>>,
    pub qualities: Option<Vec<Quality>>,
    pub languages: Option<Vec<Language>>,
    pub size_min: Option<u64>,
    pub size_max: Option<u64>,
    pub seeders_min: Option<u32>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub enum SearchScope {
    #[default]
    All,
    Source(SourceId),
    Pool(PoolId),
}

/// One row of search results: the entry plus the list of source ids that have
/// it (provenance display).
#[derive(Debug, Clone)]
pub struct SearchHit {
    pub entry: Entry,
    pub provenance: Vec<SourceId>,
}

impl Database {
    pub fn search(&self, q: &SearchQuery) -> Result<Vec<SearchHit>> {
        let conn = self.conn.lock().unwrap();

        let scope_clause: String = match &q.scope {
            SearchScope::All => String::new(),
            SearchScope::Source(_) => {
                " AND e.id IN (SELECT entry_id FROM entry_sources WHERE source_id = :scope_source)"
                    .into()
            }
            SearchScope::Pool(_) => " AND e.id IN (
                SELECT entry_id FROM entry_sources WHERE source_id IN (
                    SELECT source_id FROM pool_sources WHERE pool_id = :scope_pool
                )
            )"
            .into(),
        };

        let fts_clause = if q.text.as_deref().is_some_and(|t| !t.trim().is_empty()) {
            " AND e.id IN (SELECT id FROM entries_fts WHERE entries_fts MATCH :fts)"
        } else {
            ""
        };

        let cat_clause = match &q.categories {
            Some(cs) if !cs.is_empty() => format!(
                " AND e.category IN ({})",
                cs.iter()
                    .map(|c| format!("'{}'", category_to_str(*c)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            _ => String::new(),
        };

        let size_clause = match (q.size_min, q.size_max) {
            (Some(_), Some(_)) => " AND e.size_bytes >= :smin AND e.size_bytes <= :smax",
            (Some(_), None) => " AND e.size_bytes >= :smin",
            (None, Some(_)) => " AND e.size_bytes <= :smax",
            (None, None) => "",
        };
        let seed_clause = if q.seeders_min.is_some() {
            " AND e.seeders >= :seedmin"
        } else {
            ""
        };

        let limit = q.limit.unwrap_or(500);

        let sql = format!(
            "SELECT e.id, e.title, e.link_kind, e.link_value, e.category, e.tags_json,
                    e.quality_json, e.languages_json, e.size_bytes, e.seeders, e.leechers,
                    e.added_at, e.contributor_pubkey, e.signature, e.description,
                    e.poster_url, e.first_seen_at, e.primary_source_id
             FROM entries e
             WHERE 1=1 {scope_clause} {fts_clause} {cat_clause} {size_clause} {seed_clause}
             ORDER BY e.added_at DESC
             LIMIT {limit}"
        );

        let mut stmt = conn.prepare(&sql)?;

        // Build named params dynamically
        let scope_source = match &q.scope {
            SearchScope::Source(id) => Some(id.0.to_string()),
            _ => None,
        };
        let scope_pool = match &q.scope {
            SearchScope::Pool(id) => Some(id.0.to_string()),
            _ => None,
        };
        let fts = q.text.as_ref().map(|t| make_fts_query(t));
        let smin = q.size_min.map(|v| v as i64);
        let smax = q.size_max.map(|v| v as i64);
        let seedmin = q.seeders_min.map(|v| v as i64);

        let mut named: Vec<(&str, &dyn rusqlite::ToSql)> = Vec::new();
        if let Some(s) = scope_source.as_ref() {
            named.push((":scope_source", s));
        }
        if let Some(s) = scope_pool.as_ref() {
            named.push((":scope_pool", s));
        }
        if let Some(s) = fts.as_ref() {
            named.push((":fts", s));
        }
        if let Some(v) = smin.as_ref() {
            named.push((":smin", v));
        }
        if let Some(v) = smax.as_ref() {
            named.push((":smax", v));
        }
        if let Some(v) = seedmin.as_ref() {
            named.push((":seedmin", v));
        }

        let rows = stmt.query_map(named.as_slice(), row_to_entry)?;
        let entries: Vec<Entry> = rows.collect::<rusqlite::Result<Vec<_>>>()?;

        // Apply post-SQL filters that are awkward in pure SQL
        let entries: Vec<Entry> = entries.into_iter().filter(|e| post_filter(e, q)).collect();

        // Provenance lookup
        let mut hits = Vec::with_capacity(entries.len());
        for e in entries {
            let mut p_stmt =
                conn.prepare("SELECT source_id FROM entry_sources WHERE entry_id = ?1")?;
            let provenance: Vec<SourceId> = p_stmt
                .query_map(params![e.id.as_hex()], |row| {
                    let s: String = row.get(0)?;
                    Uuid::parse_str(&s).map(SourceId).map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            hits.push(SearchHit {
                entry: e,
                provenance,
            });
        }
        Ok(hits)
    }
}

/// Quote terms for FTS5 to avoid syntax errors on punctuation; default to AND
/// between terms.
fn make_fts_query(text: &str) -> String {
    text.split_whitespace()
        .map(|term| {
            let escaped = term.replace('"', "\"\"");
            format!("\"{escaped}\"")
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn post_filter(e: &Entry, q: &SearchQuery) -> bool {
    if let Some(qs) = &q.qualities {
        match &e.quality {
            Some(eq) if qs.contains(eq) => {}
            _ => return false,
        }
    }
    if let Some(ls) = &q.languages
        && !e.languages.iter().any(|l| ls.contains(l))
    {
        return false;
    }
    true
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    let id_hex: String = row.get(0)?;
    let id = ContentId::from_hex(&id_hex).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let link_kind: String = row.get(2)?;
    let link_value: String = row.get(3)?;
    let link = link_from_columns(&link_kind, &link_value).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, e.into())
    })?;
    let added_at_s: String = row.get(11)?;
    let added_at = DateTime::parse_from_rfc3339(&added_at_s)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(11, rusqlite::types::Type::Text, Box::new(e))
        })?
        .with_timezone(&Utc);

    let tags: Vec<String> = serde_json::from_str(&row.get::<_, String>(5)?).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let quality: Option<Quality> = row
        .get::<_, Option<String>>(6)?
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e))
        })?;
    let languages: Vec<Language> =
        serde_json::from_str(&row.get::<_, String>(7)?).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e))
        })?;

    let pubkey_bytes: Vec<u8> = row.get(12)?;
    let mut pubkey = [0u8; 32];
    if pubkey_bytes.len() != 32 {
        return Err(rusqlite::Error::FromSqlConversionFailure(
            12,
            rusqlite::types::Type::Blob,
            "pubkey wrong size".into(),
        ));
    }
    pubkey.copy_from_slice(&pubkey_bytes);

    let sig_bytes: Vec<u8> = row.get(13)?;
    let mut sig = [0u8; 64];
    if sig_bytes.len() != 64 {
        return Err(rusqlite::Error::FromSqlConversionFailure(
            13,
            rusqlite::types::Type::Blob,
            "signature wrong size".into(),
        ));
    }
    sig.copy_from_slice(&sig_bytes);

    let primary_source_id_str: String = row.get(17)?;
    let source_id = SourceId(Uuid::parse_str(&primary_source_id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(17, rusqlite::types::Type::Text, Box::new(e))
    })?);

    Ok(Entry {
        id,
        title: row.get(1)?,
        link,
        category: category_from_str(&row.get::<_, String>(4)?),
        tags,
        quality,
        languages,
        size_bytes: row.get::<_, Option<i64>>(8)?.map(|v| v as u64),
        seeders: row.get::<_, Option<i64>>(9)?.map(|v| v as u32),
        leechers: row.get::<_, Option<i64>>(10)?.map(|v| v as u32),
        added_at,
        contributor_pubkey: PublicKeyBytes(pubkey),
        source_id,
        signature: SignatureBytes(sig),
        description: row.get(14)?,
        poster_url: row.get(15)?,
    })
}

// --------------------------------------------------------------------------
// Pools
// --------------------------------------------------------------------------

impl Database {
    pub fn insert_pool(&self, pool: &Pool) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO pools (id, name, description, filters_json,
                                dedup_strategy, conflict_strategy_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                pool.id.0.to_string(),
                pool.name,
                pool.description,
                serde_json::to_string(&pool.filters)?,
                serde_json::to_string(&pool.dedup_strategy)?,
                serde_json::to_string(&pool.conflict_strategy)?,
                pool.created_at.to_rfc3339(),
            ],
        )?;
        for sid in &pool.members {
            tx.execute(
                "INSERT OR IGNORE INTO pool_sources (pool_id, source_id) VALUES (?1, ?2)",
                params![pool.id.0.to_string(), sid.0.to_string()],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list_pools(&self) -> Result<Vec<Pool>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, filters_json, dedup_strategy,
                    conflict_strategy_json, created_at FROM pools
             ORDER BY name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let id = PoolId(Uuid::parse_str(&id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?);
            let filters: PoolFilters =
                serde_json::from_str(&row.get::<_, String>(3)?).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
            let dedup = serde_json::from_str(&row.get::<_, String>(4)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let conflict = serde_json::from_str(&row.get::<_, String>(5)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let created_at_s: String = row.get(6)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_s)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc);
            Ok((
                id,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                filters,
                dedup,
                conflict,
                created_at,
            ))
        })?;

        let mut pools = Vec::new();
        for row in rows {
            let (id, name, description, filters, dedup, conflict, created_at) = row?;
            let mut m_stmt =
                conn.prepare("SELECT source_id FROM pool_sources WHERE pool_id = ?1")?;
            let members: Vec<SourceId> = m_stmt
                .query_map(params![id.0.to_string()], |r| {
                    let s: String = r.get(0)?;
                    Uuid::parse_str(&s).map(SourceId).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            pools.push(Pool {
                id,
                name,
                description,
                members,
                filters,
                dedup_strategy: dedup,
                conflict_strategy: conflict,
                created_at,
            });
        }
        Ok(pools)
    }

    pub fn delete_pool(&self, id: PoolId) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let n = conn.execute("DELETE FROM pools WHERE id = ?1", params![id.0.to_string()])?;
        if n == 0 {
            return Err(StorageError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use tt_core::{
        Category, ContentLink, Entry, Language, PublicKeyBytes, Quality, SignatureBytes, Source,
        SourceKind, SyncPolicy, SyncStatus,
    };

    use super::*;

    fn make_source() -> Source {
        Source {
            id: SourceId::new(),
            kind: SourceKind::LocalFolder,
            endpoint: "/tmp/source".into(),
            display_name: "Test Source".into(),
            description: None,
            auth: None,
            sync_policy: SyncPolicy::default(),
            last_sync: None,
            last_status: SyncStatus::Idle,
            trust_level: TrustLevel::Trusted,
        }
    }

    fn make_entry(source_id: SourceId, title: &str) -> Entry {
        let link = ContentLink::Magnet(
            "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567".into(),
        );
        let id = ContentId::compute(&link, title).unwrap();
        Entry {
            id,
            title: title.into(),
            link,
            category: Category::Films,
            tags: vec!["1080p".into()],
            quality: Some(Quality::P1080),
            languages: vec![Language::VOSTFR],
            size_bytes: Some(2 * 1024 * 1024 * 1024),
            seeders: Some(50),
            leechers: Some(2),
            added_at: Utc::now(),
            contributor_pubkey: PublicKeyBytes([1; 32]),
            source_id,
            signature: SignatureBytes([2; 64]),
            description: None,
            poster_url: None,
        }
    }

    #[test]
    fn source_insert_list_delete_roundtrip() {
        let db = Database::open_in_memory().unwrap();
        let s = make_source();
        db.insert_source(&s).unwrap();
        let listed = db.list_sources().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, s.id);
        db.delete_source(s.id).unwrap();
        assert!(db.list_sources().unwrap().is_empty());
    }

    #[test]
    fn entry_upsert_dedups_by_id() {
        let db = Database::open_in_memory().unwrap();
        let s = make_source();
        db.insert_source(&s).unwrap();
        let e1 = make_entry(s.id, "Inception 1080p");
        let e2 = make_entry(s.id, "Inception 1080p");
        assert!(db.upsert_entry(&e1, s.id).unwrap()); // new
        assert!(!db.upsert_entry(&e2, s.id).unwrap()); // dup, but updates
        assert_eq!(db.count_entries().unwrap(), 1);
    }

    #[test]
    fn search_finds_by_title_fts() {
        let db = Database::open_in_memory().unwrap();
        let s = make_source();
        db.insert_source(&s).unwrap();
        db.upsert_entry(&make_entry(s.id, "Inception 1080p VOSTFR"), s.id)
            .unwrap();
        db.upsert_entry(&make_entry(s.id, "The Dark Knight 1080p"), s.id)
            .unwrap();

        let q = SearchQuery {
            text: Some("Inception".into()),
            ..Default::default()
        };
        let hits = db.search(&q).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].entry.title.contains("Inception"));
    }

    #[test]
    fn search_filters_by_category() {
        let db = Database::open_in_memory().unwrap();
        let s = make_source();
        db.insert_source(&s).unwrap();
        let mut e = make_entry(s.id, "Some Series E01");
        e.category = Category::Series;
        e.id = ContentId::compute(&e.link, &e.title).unwrap();
        db.upsert_entry(&e, s.id).unwrap();

        let q = SearchQuery {
            categories: Some(vec![Category::Films]),
            ..Default::default()
        };
        assert!(db.search(&q).unwrap().is_empty());

        let q = SearchQuery {
            categories: Some(vec![Category::Series]),
            ..Default::default()
        };
        assert_eq!(db.search(&q).unwrap().len(), 1);
    }
}
