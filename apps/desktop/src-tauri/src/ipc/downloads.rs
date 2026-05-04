use tauri::State;
use tt_downloads::DownloadInfo;

use crate::state::AppState;

#[tauri::command]
pub async fn download_add(state: State<'_, AppState>, magnet: String) -> Result<usize, String> {
    state
        .downloads
        .add_magnet(&magnet)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_list(state: State<'_, AppState>) -> Result<Vec<DownloadInfo>, String> {
    Ok(state.downloads.list().await)
}

#[tauri::command]
pub async fn download_pause(state: State<'_, AppState>, id: usize) -> Result<(), String> {
    state.downloads.pause(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_unpause(state: State<'_, AppState>, id: usize) -> Result<(), String> {
    state.downloads.unpause(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_remove(state: State<'_, AppState>, id: usize) -> Result<(), String> {
    state.downloads.remove(id).await.map_err(|e| e.to_string())
}
