use tauri::State;
use tt_identity::{DefaultStore, IdentityStore, LocalKeypair, npub::encode_npub};

use crate::ipc::dto::IdentityDto;
use crate::state::AppState;

#[tauri::command]
pub fn identity_show(state: State<'_, AppState>) -> Result<Option<IdentityDto>, String> {
    let li = state.db.get_local_identity().map_err(|e| e.to_string())?;
    let Some(li) = li else { return Ok(None) };
    let npub = encode_npub(&li.pubkey).map_err(|e| e.to_string())?;
    Ok(Some(IdentityDto::from_local(li, npub)))
}

#[tauri::command]
pub fn identity_init(
    state: State<'_, AppState>,
    name: Option<String>,
) -> Result<IdentityDto, String> {
    let store = DefaultStore::new().map_err(|e| e.to_string())?;
    if store.load().map_err(|e| e.to_string())?.is_some() {
        return Err("identity already exists".into());
    }
    let kp = LocalKeypair::generate();
    store.store(&kp.seed()).map_err(|e| e.to_string())?;
    state
        .db
        .put_local_identity(&kp.public_bytes(), name.as_deref())
        .map_err(|e| e.to_string())?;
    let npub = kp.npub();
    let li = state
        .db
        .get_local_identity()
        .map_err(|e| e.to_string())?
        .expect("just inserted");
    Ok(IdentityDto::from_local(li, npub))
}
