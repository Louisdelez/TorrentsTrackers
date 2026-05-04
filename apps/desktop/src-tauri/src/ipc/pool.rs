use chrono::Utc;
use tauri::State;
use tt_core::{ConflictStrategy, DedupStrategy, Pool, PoolFilters, PoolId, SourceId};
use uuid::Uuid;

use crate::ipc::dto::PoolDto;
use crate::state::AppState;

#[tauri::command]
pub fn list_pools(state: State<'_, AppState>) -> Result<Vec<PoolDto>, String> {
    state
        .db
        .list_pools()
        .map(|v| v.into_iter().map(PoolDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_pool(
    state: State<'_, AppState>,
    name: String,
    source_ids: Vec<String>,
) -> Result<PoolDto, String> {
    if source_ids.is_empty() {
        return Err("a pool needs at least one source".into());
    }
    let mut members = Vec::with_capacity(source_ids.len());
    for s in &source_ids {
        let id = Uuid::parse_str(s).map_err(|e| format!("bad source id '{s}': {e}"))?;
        members.push(SourceId(id));
    }
    let pool = Pool {
        id: PoolId::new(),
        name,
        description: None,
        members,
        filters: PoolFilters::default(),
        dedup_strategy: DedupStrategy::default(),
        conflict_strategy: ConflictStrategy::default(),
        created_at: Utc::now(),
    };
    state.db.insert_pool(&pool).map_err(|e| e.to_string())?;
    Ok(PoolDto::from(pool))
}

#[tauri::command]
pub fn remove_pool(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let id = Uuid::parse_str(&id)
        .map(PoolId)
        .map_err(|e| e.to_string())?;
    state.db.delete_pool(id).map_err(|e| e.to_string())
}
