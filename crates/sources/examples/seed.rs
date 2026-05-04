//! Seed a local-folder source with a handful of fixture entries.
//!
//! Usage: `cargo run -p tt-sources --example seed -- <path>`
//!
//! Writes `entries.jsonl` and `community.json` under the given path.
//! Useful for local end-to-end testing of the CLI before real signed entries
//! exist (Phase 2).

use std::path::PathBuf;

use chrono::Utc;
use tt_core::{
    Category, ContentId, ContentLink, Entry, Language, PublicKeyBytes, Quality, SignatureBytes,
    SourceAdapter, SourceId,
};
use tt_sources::local::LocalFolder;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: seed <path>");

    let src = LocalFolder::new(&path);
    src.ensure_layout().await.expect("ensure layout");

    let source_id = SourceId::new();
    let entries = vec![
        make_entry(
            source_id,
            "Inception 2010 1080p BluRay VOSTFR",
            "magnet:?xt=urn:btih:1234567890abcdef1234567890abcdef12345678",
            Category::Films,
            vec!["1080p".into(), "vostfr".into(), "bluray".into()],
            Some(Quality::P1080),
            vec![Language::VOSTFR],
            5_368_709_120, // 5 GB
            234,
        ),
        make_entry(
            source_id,
            "The Dark Knight 2008 4K REMUX Multi",
            "magnet:?xt=urn:btih:abcdef1234567890abcdef1234567890abcdef12",
            Category::Films,
            vec!["4k".into(), "remux".into(), "multi".into()],
            Some(Quality::P2160),
            vec![Language::Multi],
            85_899_345_920, // 80 GB
            89,
        ),
        make_entry(
            source_id,
            "Naruto Shippuden Complete VOSTFR 1080p",
            "magnet:?xt=urn:btih:fedcba9876543210fedcba9876543210fedcba98",
            Category::Series,
            vec!["1080p".into(), "vostfr".into(), "complete".into()],
            Some(Quality::P1080),
            vec![Language::VOSTFR],
            64_424_509_440, // 60 GB
            412,
        ),
        make_entry(
            source_id,
            "The Witcher 3 Wild Hunt GOTY",
            "magnet:?xt=urn:btih:0011223344556677889900112233445566778899",
            Category::Games,
            vec!["pc".into(), "goty".into()],
            None,
            vec![Language::Multi],
            53_687_091_200, // 50 GB
            156,
        ),
        make_entry(
            source_id,
            "Daft Punk Discovery FLAC",
            "magnet:?xt=urn:btih:99887766554433221100998877665544332211aa",
            Category::Music,
            vec!["flac".into()],
            None,
            vec![Language::Other("Instrumental".into())],
            524_288_000, // 500 MB
            512,
        ),
    ];

    for e in &entries {
        src.publish_entry(e).await.expect("publish");
    }

    let meta_path = path.join("community.json");
    if !meta_path.exists() {
        let meta = serde_json::json!({
            "display_name": "Seed Fixture",
            "description": "Local fixture for E2E tests",
            "icon_url": null,
            "modo_pubkeys": [],
            "rules": null,
            "language": "fr",
            "created_at": null,
            "member_count": null
        });
        tokio::fs::write(&meta_path, serde_json::to_vec_pretty(&meta).unwrap())
            .await
            .expect("write meta");
    }

    println!(
        "seeded {} entries in {} (community.json + entries.jsonl)",
        entries.len(),
        path.display()
    );
}

#[allow(clippy::too_many_arguments)]
fn make_entry(
    source_id: SourceId,
    title: &str,
    magnet: &str,
    category: Category,
    tags: Vec<String>,
    quality: Option<Quality>,
    languages: Vec<Language>,
    size: u64,
    seeders: u32,
) -> Entry {
    let link = ContentLink::Magnet(magnet.to_string());
    Entry {
        id: ContentId::compute(&link, title).unwrap(),
        title: title.to_string(),
        link,
        category,
        tags,
        quality,
        languages,
        size_bytes: Some(size),
        seeders: Some(seeders),
        leechers: Some(seeders / 10),
        added_at: Utc::now(),
        contributor_pubkey: PublicKeyBytes([7; 32]),
        source_id,
        signature: SignatureBytes([0; 64]),
        description: None,
        poster_url: None,
    }
}
