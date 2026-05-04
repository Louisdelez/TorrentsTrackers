//! `HttpUrl` adapter — fetches a JSON Lines file from an HTTP(S) URL.
//!
//! Two URL conventions are supported for `endpoint`:
//!
//! 1. **File URL** — ends with `.jsonl`. The entries file is fetched as-is,
//!    metadata and bans URLs are derived by replacing the filename.
//! 2. **Directory URL** — anything else. The adapter appends `entries.jsonl`,
//!    `community.json`, `bans.jsonl`.
//!
//! Conditional GET via `If-Modified-Since` is used when `since` is provided to
//! `fetch_entries`, falling back to a full body parse if the server returns
//! 200 instead of 304.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderValue, IF_MODIFIED_SINCE};
use reqwest::{Client, StatusCode};
use tracing::{debug, warn};
use tt_core::{
    Ban, CommunityMetadata, CoreError, Entry, Result as CoreResult, SourceAdapter,
    SourceCapabilities, SourceKind,
};
use url::Url;

const ENTRIES_FILE: &str = "entries.jsonl";
const COMMUNITY_FILE: &str = "community.json";
const BANS_FILE: &str = "bans.jsonl";

pub struct HttpUrl {
    endpoint: String,
    bearer: Option<String>,
    client: Client,
}

impl HttpUrl {
    pub fn new(endpoint: impl Into<String>) -> CoreResult<Self> {
        Self::with_auth(endpoint, None)
    }

    pub fn with_auth(
        endpoint: impl Into<String>,
        bearer_token: Option<String>,
    ) -> CoreResult<Self> {
        let client = Client::builder()
            .user_agent(format!("torrents-trackers/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| CoreError::Network(e.to_string()))?;
        Ok(Self {
            endpoint: endpoint.into(),
            bearer: bearer_token,
            client,
        })
    }

    fn resolve(&self, file: &str) -> CoreResult<Url> {
        if self.endpoint.ends_with(".jsonl") || self.endpoint.ends_with(".json") {
            // File URL — replace the trailing filename with the requested one.
            let base = Url::parse(&self.endpoint)
                .map_err(|e| CoreError::Network(format!("invalid endpoint url: {e}")))?;
            base.join(file)
                .map_err(|e| CoreError::Network(format!("url join: {e}")))
        } else {
            // Directory URL — ensure trailing slash before joining.
            let with_slash = if self.endpoint.ends_with('/') {
                self.endpoint.clone()
            } else {
                format!("{}/", self.endpoint)
            };
            let base = Url::parse(&with_slash)
                .map_err(|e| CoreError::Network(format!("invalid endpoint url: {e}")))?;
            base.join(file)
                .map_err(|e| CoreError::Network(format!("url join: {e}")))
        }
    }

    fn entries_url(&self) -> CoreResult<Url> {
        if self.endpoint.ends_with(".jsonl") {
            Url::parse(&self.endpoint)
                .map_err(|e| CoreError::Network(format!("invalid endpoint url: {e}")))
        } else {
            self.resolve(ENTRIES_FILE)
        }
    }
}

#[async_trait]
impl SourceAdapter for HttpUrl {
    fn kind(&self) -> SourceKind {
        SourceKind::HttpUrl
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            read: true,
            write: false,
            watch: false,
            incremental_sync: true,
            authenticated: self.bearer.is_some(),
        }
    }

    async fn fetch_entries(&self, since: Option<DateTime<Utc>>) -> CoreResult<Vec<Entry>> {
        let url = self.entries_url()?;
        let mut req = self.client.get(url.clone());
        if let Some(token) = &self.bearer {
            req = req.bearer_auth(token);
        }
        if let Some(t) = since {
            // RFC 7231 / 7232 IMF-fixdate format
            let s = t.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
            if let Ok(v) = HeaderValue::from_str(&s) {
                req = req.header(IF_MODIFIED_SINCE, v);
            }
        }
        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;
        if resp.status() == StatusCode::NOT_MODIFIED {
            debug!(target: "tt_sources::http", "{} unchanged (304)", url);
            return Ok(Vec::new());
        }
        if !resp.status().is_success() {
            return Err(CoreError::Network(format!(
                "{} returned status {}",
                url,
                resp.status()
            )));
        }
        let body = resp
            .text()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;
        Ok(parse_jsonl(&body, since))
    }

    async fn fetch_metadata(&self) -> CoreResult<CommunityMetadata> {
        let url = self.resolve(COMMUNITY_FILE)?;
        let mut req = self.client.get(url.clone());
        if let Some(token) = &self.bearer {
            req = req.bearer_auth(token);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(default_metadata(&self.endpoint));
        }
        if !resp.status().is_success() {
            return Err(CoreError::Network(format!(
                "{} returned status {}",
                url,
                resp.status()
            )));
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    async fn publish_entry(&self, _entry: &Entry) -> CoreResult<()> {
        Err(CoreError::Source("HttpUrl source is read-only".to_string()))
    }

    async fn fetch_bans(&self) -> CoreResult<Vec<Ban>> {
        let url = self.resolve(BANS_FILE)?;
        let mut req = self.client.get(url.clone());
        if let Some(token) = &self.bearer {
            req = req.bearer_auth(token);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        if !resp.status().is_success() {
            return Err(CoreError::Network(format!(
                "{} returned status {}",
                url,
                resp.status()
            )));
        }
        let body = resp
            .text()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;
        Ok(body
            .lines()
            .enumerate()
            .filter_map(|(i, l)| {
                let l = l.trim();
                if l.is_empty() || l.starts_with('#') {
                    return None;
                }
                match serde_json::from_str::<Ban>(l) {
                    Ok(b) => Some(b),
                    Err(e) => {
                        warn!(target: "tt_sources::http", "skipping bad ban line {}: {}", i + 1, e);
                        None
                    }
                }
            })
            .collect())
    }
}

fn parse_jsonl(body: &str, since: Option<DateTime<Utc>>) -> Vec<Entry> {
    body.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            match serde_json::from_str::<Entry>(line) {
                Ok(entry) => Some((i, entry)),
                Err(e) => {
                    warn!(target: "tt_sources::http", "skipping bad line {}: {}", i + 1, e);
                    None
                }
            }
        })
        .filter_map(|(_, entry)| {
            if since.is_some_and(|t| entry.added_at < t) {
                None
            } else {
                Some(entry)
            }
        })
        .collect()
}

fn default_metadata(endpoint: &str) -> CommunityMetadata {
    let display_name = Url::parse(endpoint)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| "remote".to_string());
    CommunityMetadata {
        display_name,
        description: None,
        icon_url: None,
        modo_pubkeys: Vec::new(),
        rules: None,
        language: None,
        created_at: None,
        member_count: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_directory_url() {
        let s = HttpUrl::new("https://example.com/anime-fr").unwrap();
        let u = s.resolve(ENTRIES_FILE).unwrap();
        assert_eq!(u.as_str(), "https://example.com/anime-fr/entries.jsonl");
    }

    #[test]
    fn resolve_directory_url_with_trailing_slash() {
        let s = HttpUrl::new("https://example.com/anime-fr/").unwrap();
        let u = s.resolve(COMMUNITY_FILE).unwrap();
        assert_eq!(u.as_str(), "https://example.com/anime-fr/community.json");
    }

    #[test]
    fn resolve_file_url_replaces_filename() {
        let s =
            HttpUrl::new("https://raw.githubusercontent.com/user/repo/main/entries.jsonl").unwrap();
        let u = s.resolve(COMMUNITY_FILE).unwrap();
        assert_eq!(
            u.as_str(),
            "https://raw.githubusercontent.com/user/repo/main/community.json"
        );
    }

    #[test]
    fn entries_url_uses_file_endpoint_directly() {
        let s = HttpUrl::new("https://raw.example/path/entries.jsonl").unwrap();
        let u = s.entries_url().unwrap();
        assert_eq!(u.as_str(), "https://raw.example/path/entries.jsonl");
    }

    #[test]
    fn parse_jsonl_skips_invalid() {
        let body = "not json\n# comment\n\n";
        let entries = parse_jsonl(body, None);
        assert!(entries.is_empty());
    }
}
