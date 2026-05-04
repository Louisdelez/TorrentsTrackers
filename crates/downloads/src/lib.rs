//! In-app BitTorrent downloader for TorrentsTrackers.
//!
//! Wraps a single `librqbit::Session` behind a small async API:
//!
//! - [`DownloadManager::add_magnet`] — register a magnet URI; the file(s)
//!   start downloading immediately into `data_dir/downloads/`.
//! - [`DownloadManager::list`] — snapshot of every active torrent: title,
//!   percent done, down/up speed, state.
//! - [`DownloadManager::pause`] / [`DownloadManager::unpause`] —
//!   suspend/resume per torrent.
//! - [`DownloadManager::remove`] — drop the torrent (files left on disk).
//!
//! The session listens on an OS-allocated TCP port for incoming peer
//! connections. **No port forwarding is required** — outgoing connections
//! to the swarm are enough to make progress; if the user wants better
//! throughput they can forward the listening port themselves later.

pub mod error;
pub mod manager;

pub use error::{DownloadError, Result};
pub use manager::{DownloadInfo, DownloadManager, DownloadState};
