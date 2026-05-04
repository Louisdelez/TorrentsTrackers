# Data Model

Détail des types de domaine. Implémentés dans `crates/core`.

## Vue d'ensemble

```
Entry  <─── n:n ───>  Source  <─── n:n ───>  Pool
  │                     │
  │                     └─► CommunityMetadata (1:1)
  │
  └─► Category, Quality, Language (enums)
  └─► PublicKey (signature)
```

## Entry

L'unité atomique de contenu — un torrent partagé, catalogué par une communauté.

```rust
pub struct Entry {
    pub id: ContentId,
    pub title: String,
    pub link: ContentLink,
    pub category: Category,
    pub tags: Vec<String>,
    pub quality: Option<Quality>,
    pub languages: Vec<Language>,
    pub size_bytes: Option<u64>,
    pub seeders: Option<u32>,
    pub leechers: Option<u32>,
    pub added_at: DateTime<Utc>,
    pub contributor_pubkey: PublicKey,
    pub source_id: SourceId,
    pub signature: Signature,
    pub description: Option<String>,
    pub poster_url: Option<String>,
}
```

### ContentId

Hash stable BLAKE3 calculé sur les champs canoniques :

```rust
pub fn content_id(link: &ContentLink, title: &str) -> ContentId {
    let mut hasher = blake3::Hasher::new();
    match link {
        ContentLink::Magnet(m) => {
            // On hash uniquement l'info_hash extrait du magnet (xt=urn:btih:...)
            hasher.update(extract_info_hash(m).as_bytes());
        }
        ContentLink::TorrentUrl(u) => hasher.update(u.as_bytes()),
        ContentLink::InfoHash(h) => hasher.update(h),
    }
    // Le titre normalisé permet de détecter les doublons même quand
    // les magnets diffèrent légèrement (trackers ajoutés, etc.)
    hasher.update(normalize_title(title).as_bytes());
    ContentId(hasher.finalize().into())
}
```

La normalisation de titre : lowercase, suppression de la ponctuation et des trackers info dans le magnet.

### Category

Set canonique fermé (extensions futures via `Other(String)` si besoin) :

```rust
pub enum Category {
    Films,
    Series,
    Games,
    Music,
    Books,
    Software,
    Other,
}
```

Mappings courants automatiques pour parser les sources :
- `movie`, `cinema`, `films` → `Films`
- `tv`, `show`, `series`, `série` → `Series`
- `game`, `pc-game`, `console` → `Games`
- `audio`, `mp3`, `flac` → `Music`
- `ebook`, `pdf`, `manga`, `comics` → `Books`
- `app`, `application`, `software`, `os` → `Software`

### Quality

```rust
pub enum Quality {
    P480,
    P720,
    P1080,
    P2160,         // 4K
    Other(String), // ex: "DVDrip", "BRRip", "WEBDL"
}
```

Parsing depuis le titre via regex : `\b(480p|720p|1080p|2160p|4k|uhd)\b` (case-insensitive).

### Language

```rust
pub enum Language {
    FR,
    VOSTFR,
    EN,
    Multi,
    Other(String),
}
```

Parsing depuis tags ou titre.

### ContentLink

```rust
pub enum ContentLink {
    Magnet(String),                    // "magnet:?xt=urn:btih:..."
    TorrentUrl(String),                // URL HTTP vers un .torrent
    InfoHash([u8; 20]),                // info_hash brut
}
```

L'app peut convertir `InfoHash` en `Magnet` à la volée pour le passer au client torrent.

## Source

Une connexion à un backend qui sert des entries.

```rust
pub struct Source {
    pub id: SourceId,
    pub kind: SourceKind,
    pub endpoint: String,
    pub display_name: String,
    pub description: Option<String>,
    pub auth: Option<AuthConfig>,
    pub sync_policy: SyncPolicy,
    pub last_sync: Option<DateTime<Utc>>,
    pub last_status: SyncStatus,
    pub trust_level: TrustLevel,
}

pub enum SourceKind {
    LocalFolder,
    HttpUrl,
    GitRepo,
    GoogleDrive,
    Dropbox,
    OneDrive,
    Server,
    Nostr,
    Ipfs,
}

pub enum AuthConfig {
    None,
    BearerToken(String),               // stocké chiffré
    OAuth { provider: String, token: OAuthToken },
    PublicKey(PublicKey),              // pour les sources qui auth par clé
}

pub struct SyncPolicy {
    pub auto_sync: bool,
    pub interval: Duration,
    pub bandwidth_limit_kbps: Option<u32>,
}

pub enum SyncStatus {
    Idle,
    Syncing { started_at: DateTime<Utc> },
    Success { at: DateTime<Utc>, fetched: usize },
    Failed { at: DateTime<Utc>, error: String },
}

pub enum TrustLevel {
    Unverified,        // nouvelle source, on accepte mais on flag
    Trusted,           // l'utilisateur a explicitement validé
    Modos,             // l'utilisateur fait partie des modos
}
```

## Pool

Combinaison user-définie de plusieurs sources avec règles d'agrégation.

```rust
pub struct Pool {
    pub id: PoolId,
    pub name: String,
    pub description: Option<String>,
    pub members: Vec<SourceId>,
    pub filters: PoolFilters,
    pub dedup_strategy: DedupStrategy,
    pub conflict_strategy: ConflictStrategy,
    pub created_at: DateTime<Utc>,
}

pub struct PoolFilters {
    pub categories: Option<Vec<Category>>,    // None = toutes
    pub tags_required: Vec<String>,
    pub tags_excluded: Vec<String>,
    pub qualities: Option<Vec<Quality>>,
    pub languages: Option<Vec<Language>>,
    pub size_min: Option<u64>,
    pub size_max: Option<u64>,
}

pub enum DedupStrategy {
    ByContentId,                        // par défaut
    ByMagnetExact,                      // magnet brut identique
    None,                               // garder tous les doublons
}

pub enum ConflictStrategy {
    KeepAll { merge_metadata: bool },   // garde toutes les copies, merge les metadata
    PreferSource(SourceId),             // une source gagne en cas de conflit
    PreferNewest,                       // la plus récente gagne
    PreferMostSeeded,                   // si seeders connus
}
```

## CommunityMetadata

Information descriptive d'une communauté (= d'une source). Récupérée via `SourceAdapter::fetch_metadata()`.

```rust
pub struct CommunityMetadata {
    pub display_name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub modo_pubkeys: Vec<PublicKey>,
    pub rules: Option<String>,           // markdown
    pub language: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub member_count: Option<u32>,
}
```

Un fichier `community.json` dans le repo / dossier / etc. fournit ces métadonnées.

## Identité

```rust
pub struct LocalIdentity {
    pub keypair: SigningKey,             // ed25519, stocké en keyring
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct PublicKey(pub [u8; 32]);

impl PublicKey {
    pub fn to_npub(&self) -> String { /* bech32 encoding */ }
    pub fn from_npub(s: &str) -> Result<Self> { /* ... */ }
}
```

## Diagramme relationnel SQLite (résumé)

```
identity         (1 row, la clé locale)
sources          (id, kind, endpoint, display_name, ...)
entries          (id, title, link, category, ..., signature)
entry_sources    (entry_id, source_id, first_seen_at)         -- n:n
pools            (id, name, filters, ...)
pool_sources     (pool_id, source_id)                         -- n:n
bans             (source_id, banned_pubkey, reason, banned_at)
chat_servers     (id, url, ...)                               -- phase 4
messages         (id, server_id, channel, author_pubkey, ...)
```

Schéma complet et migrations dans `crates/storage/migrations/`.
