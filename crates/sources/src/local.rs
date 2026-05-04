//! `LocalFolder` adapter — reads JSON Lines files from a local directory.
//!
//! Expected layout under the configured root:
//!   - `entries.jsonl`   one [`Entry`] per line
//!   - `community.json`  a [`CommunityMetadata`] document
//!   - `bans.jsonl`      one [`Ban`] per line (optional)

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, warn};
use tt_core::{
    Ban, CommunityMetadata, CoreError, Entry, Result as CoreResult, SourceAdapter,
    SourceCapabilities, SourceKind,
};

const ENTRIES_FILE: &str = "entries.jsonl";
const COMMUNITY_FILE: &str = "community.json";
const BANS_FILE: &str = "bans.jsonl";

pub struct LocalFolder {
    root: PathBuf,
}

impl LocalFolder {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[async_trait]
impl SourceAdapter for LocalFolder {
    fn kind(&self) -> SourceKind {
        SourceKind::LocalFolder
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            read: true,
            write: true,
            watch: false,
            incremental_sync: true,
            authenticated: false,
        }
    }

    async fn fetch_entries(&self, since: Option<DateTime<Utc>>) -> CoreResult<Vec<Entry>> {
        let path = self.root.join(ENTRIES_FILE);
        if !path.exists() {
            debug!(target: "tt_sources::local", "no entries file at {}", path.display());
            return Ok(Vec::new());
        }
        read_jsonl::<Entry>(&path, |e| since.is_none_or(|t| e.added_at >= t)).await
    }

    async fn fetch_metadata(&self) -> CoreResult<CommunityMetadata> {
        let path = self.root.join(COMMUNITY_FILE);
        if !path.exists() {
            return Ok(CommunityMetadata {
                display_name: self
                    .root
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "local".into()),
                description: None,
                icon_url: None,
                modo_pubkeys: Vec::new(),
                rules: None,
                language: None,
                created_at: None,
                member_count: None,
            });
        }
        let bytes = tokio::fs::read(&path).await?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    async fn publish_entry(&self, entry: &Entry) -> CoreResult<()> {
        tokio::fs::create_dir_all(&self.root).await?;
        let path = self.root.join(ENTRIES_FILE);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;
        let mut line = serde_json::to_vec(entry)?;
        line.push(b'\n');
        file.write_all(&line).await?;
        file.flush().await?;
        Ok(())
    }

    async fn fetch_bans(&self) -> CoreResult<Vec<Ban>> {
        let path = self.root.join(BANS_FILE);
        if !path.exists() {
            return Ok(Vec::new());
        }
        read_jsonl::<Ban>(&path, |_| true).await
    }
}

async fn read_jsonl<T>(path: &Path, mut keep: impl FnMut(&T) -> bool) -> CoreResult<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    let file = tokio::fs::File::open(path).await?;
    let mut reader = BufReader::new(file).lines();
    let mut out = Vec::new();
    let mut line_no = 0usize;
    while let Some(line) = reader.next_line().await? {
        line_no += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        match serde_json::from_str::<T>(trimmed) {
            Ok(item) if keep(&item) => out.push(item),
            Ok(_) => {}
            Err(e) => {
                warn!(
                    target: "tt_sources::local",
                    "skipping malformed line {} in {}: {}",
                    line_no,
                    path.display(),
                    e
                );
            }
        }
    }
    Ok(out)
}

// Map serde / io errors transparently — `?` already does this thanks to
// the `From` impls on `CoreError`.
impl LocalFolder {
    /// Convenience: ensure the layout files exist (creates empty entries.jsonl).
    pub async fn ensure_layout(&self) -> CoreResult<()> {
        tokio::fs::create_dir_all(&self.root).await?;
        let path = self.root.join(ENTRIES_FILE);
        if !path.exists() {
            tokio::fs::File::create(&path).await?;
        }
        Ok(())
    }
}

// silence unused-import warning when CoreError isn't named directly.
#[allow(dead_code)]
fn _assert_error_path(_: CoreError) {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;
    use tt_core::{
        Category, ContentId, ContentLink, Entry, Language, PublicKeyBytes, Quality, SignatureBytes,
        SourceId,
    };

    fn make_entry(title: &str) -> Entry {
        let link = ContentLink::Magnet(
            "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567".into(),
        );
        Entry {
            id: ContentId::compute(&link, title).unwrap(),
            title: title.into(),
            link,
            category: Category::Films,
            tags: vec!["1080p".into()],
            quality: Some(Quality::P1080),
            languages: vec![Language::VOSTFR],
            size_bytes: Some(2_000_000_000),
            seeders: Some(50),
            leechers: Some(2),
            added_at: Utc::now(),
            contributor_pubkey: PublicKeyBytes([1; 32]),
            source_id: SourceId::new(),
            signature: SignatureBytes([2; 64]),
            description: None,
            poster_url: None,
        }
    }

    #[tokio::test]
    async fn publish_and_fetch_roundtrip() {
        let dir = TempDir::new().unwrap();
        let src = LocalFolder::new(dir.path());
        src.ensure_layout().await.unwrap();
        src.publish_entry(&make_entry("First Movie")).await.unwrap();
        src.publish_entry(&make_entry("Second Movie"))
            .await
            .unwrap();
        let entries = src.fetch_entries(None).await.unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn fetch_returns_empty_when_no_file() {
        let dir = TempDir::new().unwrap();
        let src = LocalFolder::new(dir.path());
        let entries = src.fetch_entries(None).await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn skips_malformed_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("entries.jsonl");
        let valid = serde_json::to_string(&make_entry("Valid")).unwrap();
        tokio::fs::write(
            &path,
            format!("not json\n{valid}\n# comment\n\n{{ partial: oops"),
        )
        .await
        .unwrap();
        let src = LocalFolder::new(dir.path());
        let entries = src.fetch_entries(None).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Valid");
    }

    #[tokio::test]
    async fn since_filter_works() {
        let dir = TempDir::new().unwrap();
        let src = LocalFolder::new(dir.path());
        src.ensure_layout().await.unwrap();
        let mut old = make_entry("Old");
        old.added_at = Utc::now() - chrono::Duration::days(10);
        src.publish_entry(&old).await.unwrap();
        let recent = make_entry("Recent");
        src.publish_entry(&recent).await.unwrap();
        let cutoff = Utc::now() - chrono::Duration::days(1);
        let entries = src.fetch_entries(Some(cutoff)).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Recent");
    }

    #[tokio::test]
    async fn metadata_falls_back_to_dirname() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("anime-fr");
        tokio::fs::create_dir(&sub).await.unwrap();
        let src = LocalFolder::new(&sub);
        let meta = src.fetch_metadata().await.unwrap();
        assert_eq!(meta.display_name, "anime-fr");
    }
}
