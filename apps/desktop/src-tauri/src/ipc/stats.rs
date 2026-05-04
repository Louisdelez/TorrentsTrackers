use tauri::State;

use crate::ipc::dto::StatsDto;
use crate::state::AppState;

#[tauri::command]
pub fn stats(state: State<'_, AppState>) -> Result<StatsDto, String> {
    let data_dir = tt_storage::paths::data_dir().map_err(|e| e.to_string())?;
    let db_path = tt_storage::paths::db_path().map_err(|e| e.to_string())?;
    let sources = state.db.list_sources().map_err(|e| e.to_string())?.len();
    let pools = state.db.list_pools().map_err(|e| e.to_string())?.len();
    let entries = state.db.count_entries().map_err(|e| e.to_string())?;
    Ok(StatsDto {
        data_dir: data_dir.display().to_string(),
        db_path: db_path.display().to_string(),
        sources,
        pools,
        entries,
    })
}
