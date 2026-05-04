use anyhow::{Context, Result, bail};
use chrono::Utc;
use clap::Args;
use tt_core::{
    ContentId, ContentLink, Entry, Language, PublicKeyBytes, SignatureBytes, SourceCapabilities,
};
use tt_identity::sign_entry;
use tt_storage::Database;

use crate::commands::identity::load_keypair;
use crate::commands::search::{parse_category_str, parse_quality_str};
use crate::commands::source::make_adapter;
use crate::fmt::short_id;

#[derive(Args)]
pub struct PublishArgs {
    /// Magnet URI (e.g. `magnet:?xt=urn:btih:...`).
    pub magnet: String,
    /// Source id prefix to publish to (must support writes — currently only
    /// LocalFolder).
    #[arg(long)]
    pub to: String,
    /// Title (defaults to derived from magnet `dn=`).
    #[arg(long)]
    pub title: Option<String>,
    /// Category.
    #[arg(long, default_value = "other")]
    pub category: String,
    /// Comma-separated tags.
    #[arg(long, value_delimiter = ',')]
    pub tags: Vec<String>,
    /// Quality (480p / 720p / 1080p / 4k).
    #[arg(long)]
    pub quality: Option<String>,
    /// Languages (vf / vostfr / en / multi). Comma separated.
    #[arg(long, value_delimiter = ',')]
    pub language: Vec<String>,
    /// Total size in bytes.
    #[arg(long)]
    pub size: Option<u64>,
}

pub async fn run(args: PublishArgs, db: &Database) -> Result<()> {
    let kp = load_keypair()?;

    let sources = db.list_sources()?;
    let source = sources
        .iter()
        .find(|s| s.id.0.to_string().starts_with(&args.to))
        .cloned()
        .with_context(|| format!("no source matches prefix '{}'", args.to))?;

    let adapter = make_adapter(&source)?;
    let SourceCapabilities { write, .. } = adapter.capabilities();
    if !write {
        bail!(
            "source {} ({:?}) is read-only — cannot publish to it",
            source.display_name,
            source.kind
        );
    }

    let title = args
        .title
        .clone()
        .or_else(|| extract_dn(&args.magnet))
        .with_context(|| "no title provided and no `dn=` field in magnet")?;
    let category = parse_category_str(&args.category)?;
    let quality = args.quality.as_deref().map(parse_quality_str).transpose()?;
    let languages = args
        .language
        .iter()
        .map(|s| crate::commands::search::parse_language_str(s))
        .collect::<Result<Vec<Language>>>()?;

    let link = ContentLink::Magnet(args.magnet.clone());
    let id =
        ContentId::compute(&link, &title).map_err(|e| anyhow::anyhow!("invalid magnet: {e}"))?;

    let mut entry = Entry {
        id,
        title: title.clone(),
        link,
        category,
        tags: args.tags.clone(),
        quality,
        languages,
        size_bytes: args.size,
        seeders: None,
        leechers: None,
        added_at: Utc::now(),
        contributor_pubkey: PublicKeyBytes([0; 32]),
        source_id: source.id,
        signature: SignatureBytes([0; 64]),
        description: None,
        poster_url: None,
    };
    sign_entry(&mut entry, &kp);

    adapter.publish_entry(&entry).await?;
    db.upsert_entry(&entry, source.id)?;

    println!(
        "published {} '{}' to {} '{}'",
        short_id(&entry.id.as_hex()),
        title,
        short_id(&source.id.0.to_string()),
        source.display_name
    );
    Ok(())
}

/// Pull `dn=<urlencoded title>` out of a magnet URI.
fn extract_dn(magnet: &str) -> Option<String> {
    let q = magnet.strip_prefix("magnet:?")?;
    for pair in q.split('&') {
        if let Some(v) = pair.strip_prefix("dn=") {
            let decoded: String = url::form_urlencoded::parse(v.as_bytes())
                .map(|(k, _)| k.into_owned())
                .collect();
            if !decoded.is_empty() {
                return Some(decoded);
            }
        }
    }
    None
}
