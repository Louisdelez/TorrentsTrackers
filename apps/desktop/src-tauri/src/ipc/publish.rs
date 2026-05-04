use chrono::Utc;
use tauri::State;
use tt_core::{
    Category, ContentId, ContentLink, Entry, Language, PublicKeyBytes, Quality, SignatureBytes,
    SourceAdapter, SourceId, SourceKind,
};
use tt_identity::{DefaultStore, IdentityStore, LocalKeypair, sign_entry};
use tt_sources::http::HttpUrl;
use tt_sources::local::LocalFolder;
use uuid::Uuid;

use crate::ipc::dto::SearchHitDto;
use crate::state::AppState;

#[tauri::command]
pub async fn publish(
    state: State<'_, AppState>,
    magnet: String,
    target_source_id: String,
    title: String,
    category: Category,
    tags: Vec<String>,
    quality: Option<Quality>,
    languages: Vec<Language>,
    size_bytes: Option<u64>,
) -> Result<SearchHitDto, String> {
    let kp = load_keypair()?;

    let target_id = SourceId(Uuid::parse_str(&target_source_id).map_err(|e| e.to_string())?);
    let source = state
        .db
        .get_source(target_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no such source".to_string())?;

    if !matches!(source.kind, SourceKind::LocalFolder) {
        return Err(format!(
            "publishing is only supported on LocalFolder sources for now (got {:?})",
            source.kind
        ));
    }

    let link = ContentLink::Magnet(magnet);
    let id = ContentId::compute(&link, &title).map_err(|e| e.to_string())?;
    let mut entry = Entry {
        id,
        title,
        link,
        category,
        tags,
        quality,
        languages,
        size_bytes,
        seeders: None,
        leechers: None,
        added_at: Utc::now(),
        contributor_pubkey: PublicKeyBytes([0; 32]),
        source_id: target_id,
        signature: SignatureBytes([0; 64]),
        description: None,
        poster_url: None,
    };
    sign_entry(&mut entry, &kp);

    let adapter: Box<dyn SourceAdapter> = match source.kind {
        SourceKind::LocalFolder => Box::new(LocalFolder::new(&source.endpoint)),
        SourceKind::HttpUrl => {
            Box::new(HttpUrl::new(source.endpoint.clone()).map_err(|e| e.to_string())?)
        }
        _ => unreachable!("kind check above"),
    };
    adapter
        .publish_entry(&entry)
        .await
        .map_err(|e| e.to_string())?;

    state
        .db
        .upsert_entry(&entry, target_id)
        .map_err(|e| e.to_string())?;

    let provenance = vec![target_source_id.clone()];
    let dto = build_hit_dto(entry, provenance);
    Ok(dto)
}

fn load_keypair() -> Result<LocalKeypair, String> {
    let store = DefaultStore::new().map_err(|e| e.to_string())?;
    let seed = store
        .load()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "identity not initialised".to_string())?;
    Ok(LocalKeypair::from_seed(&seed))
}

fn build_hit_dto(entry: Entry, provenance: Vec<String>) -> SearchHitDto {
    let magnet = match &entry.link {
        ContentLink::Magnet(s) => Some(s.clone()),
        ContentLink::TorrentUrl(s) => Some(s.clone()),
        ContentLink::InfoHash(b) => Some(tt_core::magnet::build_magnet(b, Some(&entry.title))),
    };
    SearchHitDto {
        id: entry.id.as_hex(),
        title: entry.title,
        magnet,
        category: entry.category,
        tags: entry.tags,
        quality: entry.quality,
        languages: entry.languages,
        size_bytes: entry.size_bytes,
        seeders: entry.seeders,
        leechers: entry.leechers,
        added_at: entry.added_at.to_rfc3339(),
        contributor_pubkey_hex: hex::encode(entry.contributor_pubkey.0),
        provenance,
        description: entry.description,
    }
}
