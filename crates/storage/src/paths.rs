//! Default filesystem paths for the local data directory.

use std::path::PathBuf;

use crate::error::{Result, StorageError};

/// Return the directory where the app stores its SQLite database, sources
/// cache, and other runtime data.
///
/// Linux:   `$XDG_DATA_HOME/torrents-trackers` (typically `~/.local/share/torrents-trackers`)
/// macOS:   `~/Library/Application Support/torrents-trackers`
/// Windows: `%APPDATA%\torrents-trackers`
pub fn data_dir() -> Result<PathBuf> {
    let base = dirs::data_dir().ok_or(StorageError::NoDataDir)?;
    Ok(base.join("torrents-trackers"))
}

/// Default path of the SQLite database file.
pub fn db_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("data.sqlite"))
}

/// Ensure the data directory exists.
pub fn ensure_data_dir() -> Result<PathBuf> {
    let dir = data_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
