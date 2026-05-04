use std::path::PathBuf;

use tauri::State;
use tt_identity::{DefaultStore, IdentityStore, LocalKeypair, npub::encode_npub, portable};

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

#[tauri::command]
pub fn identity_export(path: String, passphrase: String) -> Result<usize, String> {
    if passphrase.is_empty() {
        return Err("passphrase cannot be empty".into());
    }
    let store = DefaultStore::new().map_err(|e| e.to_string())?;
    let seed = store
        .load()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no identity to export".to_string())?;
    let blob = portable::export(&seed, &passphrase).map_err(|e| e.to_string())?;
    let path = PathBuf::from(&path);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, &blob).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(blob.len())
}

#[tauri::command]
pub fn identity_import(
    state: State<'_, AppState>,
    path: String,
    passphrase: String,
    force: bool,
) -> Result<IdentityDto, String> {
    let store = DefaultStore::new().map_err(|e| e.to_string())?;
    if !force && store.load().map_err(|e| e.to_string())?.is_some() {
        return Err("identity already exists; pass force=true to overwrite".into());
    }
    let blob = std::fs::read(&path).map_err(|e| e.to_string())?;
    let seed = portable::import(&blob, &passphrase).map_err(|e| e.to_string())?;
    let kp = LocalKeypair::from_seed(&seed);
    store.store(&seed).map_err(|e| e.to_string())?;
    state
        .db
        .put_local_identity(&kp.public_bytes(), None)
        .map_err(|e| e.to_string())?;
    let npub = kp.npub();
    let li = state
        .db
        .get_local_identity()
        .map_err(|e| e.to_string())?
        .expect("just inserted");
    Ok(IdentityDto::from_local(li, npub))
}

#[tauri::command]
pub fn identity_forget(state: State<'_, AppState>) -> Result<(), String> {
    let store = DefaultStore::new().map_err(|e| e.to_string())?;
    store.delete().map_err(|e| e.to_string())?;
    state.db.clear_local_identity().map_err(|e| e.to_string())
}
