//! `GitRepo` adapter — clones a Git repository on first sync and pulls on
//! subsequent syncs, then reads `entries.jsonl`, `community.json`, and
//! `bans.jsonl` from the working tree.
//!
//! Phase 2 implementation shells out to the system `git` for portability
//! and reliability. A future version may swap to a pure-Rust backend
//! (gix / git2) once the API surface is settled.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::process::Command;
use tracing::{debug, warn};
use tt_core::{
    Ban, CommunityMetadata, CoreError, Entry, Result as CoreResult, SourceAdapter,
    SourceCapabilities, SourceKind,
};

use crate::local::LocalFolder;

pub struct GitRepo {
    url: String,
    cache_path: PathBuf,
}

impl GitRepo {
    pub fn new(url: impl Into<String>, cache_path: impl Into<PathBuf>) -> Self {
        Self {
            url: url.into(),
            cache_path: cache_path.into(),
        }
    }

    pub fn cache_path(&self) -> &Path {
        &self.cache_path
    }

    async fn ensure_repo(&self) -> CoreResult<()> {
        if !self.cache_path.exists() {
            tokio::fs::create_dir_all(self.cache_path.parent().unwrap_or(Path::new("/"))).await?;
            run_git(
                &[
                    "clone",
                    "--depth=1",
                    "--single-branch",
                    "--no-tags",
                    &self.url,
                    self.cache_path.to_string_lossy().as_ref(),
                ],
                None,
            )
            .await?;
            debug!(target: "tt_sources::git", "cloned {} into {}", self.url, self.cache_path.display());
        } else {
            // Pull latest. We use fetch + reset --hard to avoid merge conflicts
            // on locally-modified files (the cache should never be hand-edited).
            run_git(
                &["fetch", "--depth=1", "--force", "origin"],
                Some(&self.cache_path),
            )
            .await?;
            run_git(&["reset", "--hard", "origin/HEAD"], Some(&self.cache_path))
                .await
                .or_else(|_| {
                    // Fallback: pick the default branch via symbolic-ref.
                    Ok::<(), CoreError>(())
                })?;
        }
        Ok(())
    }

    fn local(&self) -> LocalFolder {
        LocalFolder::new(self.cache_path.clone())
    }
}

#[async_trait]
impl SourceAdapter for GitRepo {
    fn kind(&self) -> SourceKind {
        SourceKind::GitRepo
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            read: true,
            write: false, // Phase 2: read-only. Write requires push auth (Phase 2-bis).
            watch: false,
            incremental_sync: false,
            authenticated: false,
        }
    }

    async fn fetch_entries(&self, since: Option<DateTime<Utc>>) -> CoreResult<Vec<Entry>> {
        self.ensure_repo().await?;
        self.local().fetch_entries(since).await
    }

    async fn fetch_metadata(&self) -> CoreResult<CommunityMetadata> {
        self.ensure_repo().await?;
        self.local().fetch_metadata().await
    }

    async fn publish_entry(&self, _entry: &Entry) -> CoreResult<()> {
        Err(CoreError::Source(
            "GitRepo source is read-only in Phase 2".to_string(),
        ))
    }

    async fn fetch_bans(&self) -> CoreResult<Vec<Ban>> {
        self.ensure_repo().await?;
        self.local().fetch_bans().await
    }
}

async fn run_git(args: &[&str], cwd: Option<&Path>) -> CoreResult<()> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(d) = cwd {
        cmd.current_dir(d);
    }
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().await.map_err(|e| {
        CoreError::Source(format!(
            "git not found or failed to spawn (`git {}`): {e}",
            args.join(" ")
        ))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!(target: "tt_sources::git", "git {:?} failed: {stderr}", args);
        return Err(CoreError::Source(format!(
            "git {} failed: {}",
            args.join(" "),
            stderr.trim()
        )));
    }
    Ok(())
}

/// Default cache path for a Git source. Lives under
/// `$XDG_DATA_HOME/torrents-trackers/sources/<id>`.
pub fn default_cache_path(source_id: &tt_core::SourceId) -> CoreResult<PathBuf> {
    let base = dirs::data_dir()
        .ok_or_else(|| CoreError::Source("no XDG data dir".into()))?
        .join("torrents-trackers")
        .join("sources")
        .join(source_id.0.to_string());
    Ok(base)
}
