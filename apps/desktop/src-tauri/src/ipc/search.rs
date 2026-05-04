use tauri::State;
use tt_core::{PoolId, SourceId};
use tt_storage::{SearchQuery, SearchScope};
use uuid::Uuid;

use crate::ipc::dto::{ScopeDto, SearchHitDto, SearchQueryDto};
use crate::state::AppState;

#[tauri::command]
pub fn search(
    state: State<'_, AppState>,
    query: SearchQueryDto,
) -> Result<Vec<SearchHitDto>, String> {
    let scope = match query.scope {
        ScopeDto::All => SearchScope::All,
        ScopeDto::Source { id } => {
            SearchScope::Source(SourceId(Uuid::parse_str(&id).map_err(|e| e.to_string())?))
        }
        ScopeDto::Pool { id } => {
            SearchScope::Pool(PoolId(Uuid::parse_str(&id).map_err(|e| e.to_string())?))
        }
    };
    let q = SearchQuery {
        text: query.text,
        scope,
        categories: query.categories,
        qualities: query.qualities,
        languages: query.languages,
        size_min: query.size_min,
        size_max: query.size_max,
        seeders_min: query.seeders_min,
        limit: query.limit,
    };
    let hits = state.db.search(&q).map_err(|e| e.to_string())?;
    Ok(hits.into_iter().map(SearchHitDto::from).collect())
}
