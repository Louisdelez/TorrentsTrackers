//! Schema migrations.
//!
//! Each migration is a forward-only SQL block applied in order. The
//! `meta.schema_version` row tracks the highest applied migration.

use rusqlite::Connection;
use tracing::info;

use crate::error::Result;

/// Latest schema version known by this build.
pub const CURRENT_VERSION: i64 = 1;

/// Apply all pending migrations to bring the database up to [`CURRENT_VERSION`].
pub fn migrate(conn: &mut Connection) -> Result<()> {
    bootstrap_meta(conn)?;
    let mut version = current_version(conn)?;
    while version < CURRENT_VERSION {
        version += 1;
        let tx = conn.transaction()?;
        apply(&tx, version)?;
        tx.execute(
            "INSERT INTO meta(key, value) VALUES('schema_version', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            (version.to_string(),),
        )?;
        tx.commit()?;
        info!(target: "tt_storage", "applied migration v{version}");
    }
    Ok(())
}

fn bootstrap_meta(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
         );",
    )?;
    Ok(())
}

fn current_version(conn: &Connection) -> Result<i64> {
    let v: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        )
        .ok();
    Ok(v.and_then(|s| s.parse().ok()).unwrap_or(0))
}

fn apply(tx: &rusqlite::Transaction<'_>, version: i64) -> Result<()> {
    let sql = match version {
        1 => V1,
        v => panic!("no migration defined for version {v}"),
    };
    tx.execute_batch(sql)?;
    Ok(())
}

const V1: &str = r#"
-- Sources (communities)
CREATE TABLE sources (
    id                  TEXT PRIMARY KEY,
    kind                TEXT NOT NULL,
    endpoint            TEXT NOT NULL,
    display_name        TEXT NOT NULL,
    description         TEXT,
    auth_json           TEXT,
    sync_policy_json    TEXT NOT NULL,
    last_sync           TEXT,
    last_status_json    TEXT NOT NULL,
    trust_level         TEXT NOT NULL DEFAULT 'Unverified',
    created_at          TEXT NOT NULL
);

-- Pools (user-defined combinations of sources)
CREATE TABLE pools (
    id                      TEXT PRIMARY KEY,
    name                    TEXT NOT NULL,
    description             TEXT,
    filters_json            TEXT NOT NULL,
    dedup_strategy          TEXT NOT NULL,
    conflict_strategy_json  TEXT NOT NULL,
    created_at              TEXT NOT NULL
);

-- N:N pool <-> sources
CREATE TABLE pool_sources (
    pool_id     TEXT NOT NULL,
    source_id   TEXT NOT NULL,
    PRIMARY KEY (pool_id, source_id),
    FOREIGN KEY (pool_id)   REFERENCES pools(id)   ON DELETE CASCADE,
    FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE
);

-- Entries (deduped by ContentId, hex)
CREATE TABLE entries (
    id                  TEXT PRIMARY KEY,
    title               TEXT NOT NULL,
    link_kind           TEXT NOT NULL,
    link_value          TEXT NOT NULL,
    category            TEXT NOT NULL,
    tags_json           TEXT NOT NULL,
    quality_json        TEXT,
    languages_json      TEXT NOT NULL,
    size_bytes          INTEGER,
    seeders             INTEGER,
    leechers            INTEGER,
    added_at            TEXT NOT NULL,
    contributor_pubkey  BLOB NOT NULL,
    signature           BLOB NOT NULL,
    description         TEXT,
    poster_url          TEXT,
    first_seen_at       TEXT NOT NULL,
    -- The source declared in the entry payload (where the contributor said it
    -- belongs). Distinct from entry_sources, which tracks every source we've
    -- observed the entry in.
    primary_source_id   TEXT NOT NULL
);

CREATE INDEX idx_entries_category   ON entries(category);
CREATE INDEX idx_entries_added_at   ON entries(added_at);
CREATE INDEX idx_entries_contributor ON entries(contributor_pubkey);

-- N:N entries <-> sources (provenance)
CREATE TABLE entry_sources (
    entry_id    TEXT NOT NULL,
    source_id   TEXT NOT NULL,
    seen_at     TEXT NOT NULL,
    PRIMARY KEY (entry_id, source_id),
    FOREIGN KEY (entry_id)  REFERENCES entries(id) ON DELETE CASCADE,
    FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE
);

CREATE INDEX idx_entry_sources_source ON entry_sources(source_id);

-- FTS5 virtual table for full-text search on titles
CREATE VIRTUAL TABLE entries_fts USING fts5(
    id UNINDEXED,
    title,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TRIGGER entries_fts_ai AFTER INSERT ON entries BEGIN
    INSERT INTO entries_fts(id, title) VALUES (new.id, new.title);
END;
CREATE TRIGGER entries_fts_ad AFTER DELETE ON entries BEGIN
    DELETE FROM entries_fts WHERE id = old.id;
END;
CREATE TRIGGER entries_fts_au AFTER UPDATE ON entries BEGIN
    UPDATE entries_fts SET title = new.title WHERE id = old.id;
END;

-- Bans (a community can ban a contributor by pubkey)
CREATE TABLE bans (
    source_id   TEXT NOT NULL,
    pubkey      BLOB NOT NULL,
    reason      TEXT,
    banned_at   TEXT NOT NULL,
    banned_by   BLOB NOT NULL,
    PRIMARY KEY (source_id, pubkey),
    FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE
);

-- Identity (single row keyed by string 'self')
CREATE TABLE identity (
    key             TEXT PRIMARY KEY,
    pubkey          BLOB NOT NULL,
    display_name    TEXT,
    created_at      TEXT NOT NULL
);
"#;
