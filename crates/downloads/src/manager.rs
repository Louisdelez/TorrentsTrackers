use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use librqbit::{
    AddTorrent, AddTorrentOptions, AddTorrentResponse, Session, SessionOptions, TorrentStatsState,
};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::error::{DownloadError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadState {
    Initializing,
    Live,
    Paused,
    Finished,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadInfo {
    pub id: usize,
    pub title: String,
    pub progress_bytes: u64,
    pub total_bytes: u64,
    pub down_bps: u64,
    pub up_bps: u64,
    pub state: DownloadState,
    pub finished: bool,
}

pub struct DownloadManager {
    session: Arc<Session>,
    data_dir: PathBuf,
    /// Display titles parsed from `dn=...` at add time, keyed by torrent id.
    titles: Mutex<HashMap<usize, String>>,
}

impl DownloadManager {
    /// Start a librqbit session rooted at `data_dir`. Creates the directory
    /// if missing.
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let opts = SessionOptions {
            disable_dht: false,
            disable_dht_persistence: false,
            persistence: None,
            peer_id: None,
            listen_port_range: None,
            enable_upnp_port_forwarding: false,
            ..Default::default()
        };
        let session = Session::new_with_opts(data_dir.clone(), opts)
            .await
            .map_err(|e| DownloadError::Rqbit(e.to_string()))?;
        Ok(Self {
            session,
            data_dir,
            titles: Mutex::new(HashMap::new()),
        })
    }

    pub fn data_dir(&self) -> &std::path::Path {
        &self.data_dir
    }

    /// Add a magnet URI (or `.torrent` URL) and start downloading.
    /// Returns the librqbit-assigned id.
    pub async fn add_magnet(&self, magnet_or_url: &str) -> Result<usize> {
        let added = AddTorrent::from_url(magnet_or_url);
        let opts = AddTorrentOptions {
            paused: false,
            overwrite: false,
            ..Default::default()
        };
        let resp = self
            .session
            .add_torrent(added, Some(opts))
            .await
            .map_err(|e| DownloadError::Rqbit(e.to_string()))?;
        let id = match resp {
            AddTorrentResponse::Added(id, _) => id,
            AddTorrentResponse::AlreadyManaged(id, _) => id,
            AddTorrentResponse::ListOnly(_) => {
                return Err(DownloadError::Rqbit("list-only response".into()));
            }
        };
        let title = extract_dn(magnet_or_url).unwrap_or_else(|| format!("torrent-{id}"));
        self.titles.lock().await.insert(id, title);
        Ok(id)
    }

    pub async fn pause(&self, id: usize) -> Result<()> {
        let h = self.session.get(id.into()).ok_or(DownloadError::NotFound)?;
        self.session
            .pause(&h)
            .await
            .map_err(|e| DownloadError::Rqbit(e.to_string()))
    }

    pub async fn unpause(&self, id: usize) -> Result<()> {
        let h = self.session.get(id.into()).ok_or(DownloadError::NotFound)?;
        self.session
            .unpause(&h)
            .await
            .map_err(|e| DownloadError::Rqbit(e.to_string()))
    }

    pub async fn remove(&self, id: usize) -> Result<()> {
        let h = self.session.get(id.into()).ok_or(DownloadError::NotFound)?;
        self.session
            .delete(id.into(), false)
            .await
            .map_err(|e| DownloadError::Rqbit(e.to_string()))?;
        drop(h);
        self.titles.lock().await.remove(&id);
        Ok(())
    }

    /// Snapshot every torrent in the session.
    pub async fn list(&self) -> Vec<DownloadInfo> {
        let titles = self.titles.lock().await.clone();
        self.session.with_torrents(|iter| {
            iter.map(|(id, h)| {
                let stats = h.stats();
                let title = titles
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| format!("torrent-{id}"));
                let down_bps = stats
                    .live
                    .as_ref()
                    .map(|l| (l.download_speed.mbps * 125_000.0) as u64)
                    .unwrap_or(0);
                let up_bps = stats
                    .live
                    .as_ref()
                    .map(|l| (l.upload_speed.mbps * 125_000.0) as u64)
                    .unwrap_or(0);
                let state = match stats.state {
                    TorrentStatsState::Initializing => DownloadState::Initializing,
                    TorrentStatsState::Live => {
                        if stats.finished {
                            DownloadState::Finished
                        } else {
                            DownloadState::Live
                        }
                    }
                    TorrentStatsState::Paused => DownloadState::Paused,
                    TorrentStatsState::Error => DownloadState::Error,
                };
                DownloadInfo {
                    id,
                    title,
                    progress_bytes: stats.progress_bytes,
                    total_bytes: stats.total_bytes,
                    down_bps,
                    up_bps,
                    state,
                    finished: stats.finished,
                }
            })
            .collect()
        })
    }
}

fn extract_dn(magnet: &str) -> Option<String> {
    let q = magnet.strip_prefix("magnet:?")?;
    for pair in q.split('&') {
        if let Some(v) = pair.strip_prefix("dn=") {
            let s = url_decode(v);
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

fn url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'+' {
            out.push(' ');
            i += 1;
        } else if b == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_digit(bytes[i + 1]), hex_digit(bytes[i + 2])) {
                out.push((h * 16 + l) as char);
                i += 3;
            } else {
                out.push(b as char);
                i += 1;
            }
        } else {
            out.push(b as char);
            i += 1;
        }
    }
    out
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Default location for downloads: `$XDG_DATA_HOME/torrents-trackers/downloads`.
pub fn default_data_dir() -> Result<PathBuf> {
    let base = dirs::data_dir().ok_or(DownloadError::NoDataDir)?;
    Ok(base.join("torrents-trackers").join("downloads"))
}
