use anyhow::{Context, anyhow};
use chrono::Utc;
use tauri::State;
use tt_core::{Source, SourceAdapter, SourceId, SourceKind, SyncPolicy, SyncStatus, TrustLevel};
use tt_sources::git::GitRepo;
use tt_sources::http::HttpUrl;
use tt_sources::local::LocalFolder;
use uuid::Uuid;

use crate::ipc::dto::SourceDto;
use crate::state::AppState;

#[tauri::command]
pub fn list_sources(state: State<'_, AppState>) -> Result<Vec<SourceDto>, String> {
    state
        .db
        .list_sources()
        .map(|v| v.into_iter().map(SourceDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_source(
    state: State<'_, AppState>,
    kind: String,
    endpoint: String,
    name: Option<String>,
) -> Result<SourceDto, String> {
    let kind = parse_kind(&kind).map_err(|e| e.to_string())?;
    let display_name = name.unwrap_or_else(|| derive_name(&kind, &endpoint));
    let s = Source {
        id: SourceId::new(),
        kind,
        endpoint,
        display_name,
        description: None,
        auth: None,
        sync_policy: SyncPolicy::default(),
        last_sync: None,
        last_status: SyncStatus::Idle,
        trust_level: TrustLevel::Unverified,
    };
    state.db.insert_source(&s).map_err(|e| e.to_string())?;
    Ok(SourceDto::from(s))
}

#[tauri::command]
pub async fn sync_source(state: State<'_, AppState>, id: String) -> Result<usize, String> {
    let id = parse_id(&id)?;
    let s = state
        .db
        .get_source(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no such source".to_string())?;
    sync_one(&s, &state).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_all_sources(state: State<'_, AppState>) -> Result<usize, String> {
    let sources = state.db.list_sources().map_err(|e| e.to_string())?;
    let mut total = 0usize;
    for s in sources {
        match sync_one(&s, &state).await {
            Ok(n) => total += n,
            Err(e) => {
                tracing::warn!(target: "tt_desktop::ipc", "sync {} failed: {e:#}", s.display_name)
            }
        }
    }
    Ok(total)
}

#[tauri::command]
pub fn remove_source(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let id = parse_id(&id)?;
    state.db.delete_source(id).map_err(|e| e.to_string())
}

async fn sync_one(s: &Source, state: &AppState) -> anyhow::Result<usize> {
    let adapter = make_adapter(s)?;
    let entries = adapter
        .fetch_entries(s.last_sync)
        .await
        .with_context(|| format!("fetch failed for {}", s.display_name))?;
    let mut accepted = 0usize;
    for entry in entries {
        match state.db.upsert_entry(&entry, s.id) {
            Ok(_) => accepted += 1,
            Err(tt_storage::StorageError::InvalidSignature) => {
                tracing::warn!(target: "tt_desktop::ipc", "rejected entry with invalid signature");
            }
            Err(tt_storage::StorageError::Banned) => {
                tracing::warn!(target: "tt_desktop::ipc", "rejected entry from banned contributor");
            }
            Err(e) => return Err(e.into()),
        }
    }
    state.db.update_source_sync_status(
        s.id,
        Utc::now(),
        &serde_json::to_string(&SyncStatus::Success {
            at: Utc::now(),
            fetched: accepted,
        })?,
    )?;
    Ok(accepted)
}

fn make_adapter(s: &Source) -> anyhow::Result<Box<dyn SourceAdapter>> {
    Ok(match s.kind {
        SourceKind::LocalFolder => Box::new(LocalFolder::new(&s.endpoint)),
        SourceKind::HttpUrl => Box::new(
            HttpUrl::new(s.endpoint.clone()).map_err(|e| anyhow!("invalid http source: {e}"))?,
        ),
        SourceKind::GitRepo => {
            let cache = tt_sources::git::default_cache_path(&s.id)
                .map_err(|e| anyhow!("git cache path: {e}"))?;
            Box::new(GitRepo::new(s.endpoint.clone(), cache))
        }
        other => anyhow::bail!("source kind {other:?} not supported in Phase 3"),
    })
}

fn parse_kind(s: &str) -> anyhow::Result<SourceKind> {
    Ok(match s.to_ascii_lowercase().as_str() {
        "local" | "localfolder" => SourceKind::LocalFolder,
        "http" | "httpurl" | "url" => SourceKind::HttpUrl,
        "git" | "gitrepo" => SourceKind::GitRepo,
        other => anyhow::bail!("unknown source kind '{other}'"),
    })
}

fn parse_id(s: &str) -> Result<SourceId, String> {
    Uuid::parse_str(s).map(SourceId).map_err(|e| e.to_string())
}

fn derive_name(kind: &SourceKind, endpoint: &str) -> String {
    match kind {
        SourceKind::LocalFolder => std::path::Path::new(endpoint)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| endpoint.to_string()),
        SourceKind::HttpUrl | SourceKind::GitRepo => endpoint
            .rsplit('/')
            .find(|s| !s.is_empty())
            .map(|s| s.trim_end_matches(".git").to_string())
            .unwrap_or_else(|| endpoint.to_string()),
        _ => endpoint.to_string(),
    }
}
