use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use clap::{Args, Subcommand};
use tt_core::{Source, SourceAdapter, SourceId, SourceKind, SyncPolicy, SyncStatus, TrustLevel};
use tt_sources::git::GitRepo;
use tt_sources::http::HttpUrl;
use tt_sources::local::LocalFolder;
use tt_storage::Database;

use crate::fmt::{relative_time, short_id};

#[derive(Subcommand)]
pub enum SourceCmd {
    /// Register a new source.
    Add(AddArgs),
    /// List configured sources.
    List,
    /// Sync one source (or all of them).
    Sync(SyncArgs),
    /// Delete a source by id (prefix match allowed).
    Remove(RemoveArgs),
}

#[derive(Args)]
pub struct AddArgs {
    /// Source kind: `local` | `http`.
    pub kind: String,
    /// Endpoint: a path for `local`, a URL for `http`.
    pub endpoint: String,
    /// Display name (default: derived from endpoint).
    #[arg(long)]
    pub name: Option<String>,
}

#[derive(Args)]
pub struct SyncArgs {
    /// Source id (prefix). If omitted, all sources are synced.
    pub id: Option<String>,
}

#[derive(Args)]
pub struct RemoveArgs {
    pub id: String,
}

pub async fn run(cmd: SourceCmd, db: &Database) -> Result<()> {
    match cmd {
        SourceCmd::Add(a) => add(a, db),
        SourceCmd::List => list(db),
        SourceCmd::Sync(a) => sync(a, db).await,
        SourceCmd::Remove(a) => remove(a, db),
    }
}

fn add(args: AddArgs, db: &Database) -> Result<()> {
    let kind = match args.kind.to_ascii_lowercase().as_str() {
        "local" | "localfolder" => SourceKind::LocalFolder,
        "http" | "httpurl" | "url" => SourceKind::HttpUrl,
        "git" | "gitrepo" => SourceKind::GitRepo,
        other => bail!("unknown kind '{other}'. Phase 2 supports: local, http, git."),
    };

    let display_name = args
        .name
        .unwrap_or_else(|| derive_name(&kind, &args.endpoint));

    let source = Source {
        id: SourceId::new(),
        kind,
        endpoint: args.endpoint,
        display_name,
        description: None,
        auth: None,
        sync_policy: SyncPolicy::default(),
        last_sync: None,
        last_status: SyncStatus::Idle,
        trust_level: TrustLevel::Unverified,
    };

    db.insert_source(&source)?;
    println!(
        "added {} {} '{}'",
        short_id(&source.id.0.to_string()),
        source_kind_label(source.kind),
        source.display_name
    );
    println!("       endpoint: {}", source.endpoint);
    Ok(())
}

fn derive_name(kind: &SourceKind, endpoint: &str) -> String {
    match kind {
        SourceKind::LocalFolder => std::path::Path::new(endpoint)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| endpoint.to_string()),
        SourceKind::HttpUrl => url::Url::parse(endpoint)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_else(|| endpoint.to_string()),
        _ => endpoint.to_string(),
    }
}

fn source_kind_label(k: SourceKind) -> &'static str {
    match k {
        SourceKind::LocalFolder => "[local]",
        SourceKind::HttpUrl => "[http]",
        SourceKind::GitRepo => "[git]",
        SourceKind::GoogleDrive => "[drive]",
        SourceKind::Dropbox => "[dropbox]",
        SourceKind::OneDrive => "[onedrive]",
        SourceKind::Server => "[server]",
        SourceKind::Nostr => "[nostr]",
        SourceKind::Ipfs => "[ipfs]",
    }
}

fn list(db: &Database) -> Result<()> {
    let sources = db.list_sources()?;
    if sources.is_empty() {
        println!("no sources yet. Add one with `tt source add <kind> <endpoint>`.");
        return Ok(());
    }
    println!("{} source(s):", sources.len());
    for s in sources {
        let last = match s.last_sync {
            Some(t) => format!("synced {}", relative_time(t)),
            None => "never synced".into(),
        };
        println!(
            "  {}  {:8}  {:30}  {}  {}",
            short_id(&s.id.0.to_string()),
            source_kind_label(s.kind),
            truncate(&s.display_name, 30),
            truncate(&s.endpoint, 50),
            last,
        );
    }
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max - 1).collect();
        format!("{cut}…")
    }
}

async fn sync(args: SyncArgs, db: &Database) -> Result<()> {
    let sources = db.list_sources()?;
    let targets: Vec<Source> = match args.id {
        Some(prefix) => {
            let m = match_prefix(&sources, &prefix)?;
            vec![m]
        }
        None => sources,
    };

    if targets.is_empty() {
        println!("no sources to sync.");
        return Ok(());
    }

    for s in targets {
        print!(
            "syncing {} '{}' ... ",
            short_id(&s.id.0.to_string()),
            s.display_name
        );
        let result = sync_one(&s, db).await;
        match result {
            Ok(n) => println!("ok ({n} entries)"),
            Err(e) => println!("FAILED: {e:#}"),
        }
    }
    Ok(())
}

async fn sync_one(s: &Source, db: &Database) -> Result<usize> {
    let adapter = make_adapter(s)?;
    let entries = adapter
        .fetch_entries(s.last_sync)
        .await
        .with_context(|| format!("fetch failed for {}", s.display_name))?;
    let count = entries.len();
    for entry in entries {
        db.upsert_entry(&entry, s.id)?;
    }
    db.update_source_sync_status(
        s.id,
        Utc::now(),
        &serde_json::to_string(&SyncStatus::Success {
            at: Utc::now(),
            fetched: count,
        })?,
    )?;
    Ok(count)
}

pub fn make_adapter(s: &Source) -> Result<Box<dyn SourceAdapter>> {
    Ok(match s.kind {
        SourceKind::LocalFolder => Box::new(LocalFolder::new(&s.endpoint)),
        SourceKind::HttpUrl => Box::new(
            HttpUrl::new(s.endpoint.clone()).map_err(|e| anyhow!("invalid http source: {e}"))?,
        ),
        SourceKind::GitRepo => {
            let cache = tt_sources::git::default_cache_path(&s.id)
                .map_err(|e| anyhow!("could not derive git cache path: {e}"))?;
            Box::new(GitRepo::new(s.endpoint.clone(), cache))
        }
        other => bail!("source kind {other:?} not implemented yet"),
    })
}

fn match_prefix(sources: &[Source], prefix: &str) -> Result<Source> {
    let matches: Vec<&Source> = sources
        .iter()
        .filter(|s| s.id.0.to_string().starts_with(prefix))
        .collect();
    match matches.len() {
        0 => bail!("no source matches prefix '{prefix}'"),
        1 => Ok(matches[0].clone()),
        n => bail!(
            "{n} sources match prefix '{prefix}', be more specific. Candidates: {}",
            matches
                .iter()
                .map(|s| short_id(&s.id.0.to_string()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn remove(args: RemoveArgs, db: &Database) -> Result<()> {
    let sources = db.list_sources()?;
    let s = match_prefix(&sources, &args.id)?;
    db.delete_source(s.id)?;
    println!(
        "removed {} '{}'",
        short_id(&s.id.0.to_string()),
        s.display_name
    );
    Ok(())
}
