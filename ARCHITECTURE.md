# Architecture

Document technique de référence. Lis [README.md](./README.md) d'abord pour la vision.

## Principes directeurs

1. **Local-first.** L'app est utilisable hors-ligne, avec des fichiers locaux uniquement. Le réseau est optionnel.
2. **Pas de site web, pas de domaine.** Aucun composant web central. La distribution se fait via des canaux existants (Git, cloud drive, P2P, fichier nu).
3. **L'utilisateur final n'héberge rien.** Aucun port ouvert sur sa box. Il est toujours client.
4. **Une source = une communauté.** Le canal de distribution porte les règles. La modération est intrinsèque à l'accès en écriture du backend.
5. **Identité cryptographique.** Une paire de clés ed25519 locale identifie l'utilisateur partout, indépendamment des comptes plateformes.
6. **Provenance traçable.** Chaque entry sait de quelle source elle vient. Permet ban/blacklist par source.
7. **Pluggable.** Sources, formats de chat, types de contenu sont des adapters. L'app reste agnostique.

## Modèle conceptuel

```
+-----------------------------------------------------------+
|                          UI                               |
|     sidebar (catégories + commus + pools) | search | view |
+-----------------------------------+-----------------------+
                                    |
+-----------------------------------v-----------------------+
|                          Pools                            |
|   combinaisons N -> 1 de communautés, dédup + filtres     |
+-----------------------------------+-----------------------+
                                    |
+-----------------------------------v-----------------------+
|                       Communautés                         |
|   1 source = 1 commu (règles, modos, accès = ceux         |
|   du backend choisi)                                      |
+-----------------------------------+-----------------------+
                                    |
+-----------------------------------v-----------------------+
|                         Sources                           |
|   adapters par type de plateforme                         |
|   (LocalFolder, HTTP, Git, Drive, Server, Nostr, IPFS)    |
+-----------------------------------+-----------------------+
                                    |
+-----------------------------------v-----------------------+
|                        Storage                            |
|   SQLite local : entries, sources, pools, identity, msgs  |
+-----------------------------------------------------------+
```

## Modèle de données

### Entry — l'unité de contenu

```rust
pub struct Entry {
    pub id: ContentId,                  // hash stable du contenu (dédup cross-commu)
    pub title: String,
    pub link: ContentLink,              // magnet, .torrent URL, ou InfoHash brut
    pub category: Category,             // enum canonique
    pub tags: Vec<String>,              // freeform
    pub quality: Option<Quality>,
    pub languages: Vec<Language>,
    pub size_bytes: Option<u64>,
    pub added_at: DateTime<Utc>,
    pub contributor_pubkey: PublicKey,  // qui a contribué
    pub source_id: SourceId,            // d'où ça vient (commu)
    pub signature: Signature,           // signature ed25519 de (id|title|link|...|added_at)
}

pub enum Category {
    Films, Series, Games, Music, Books, Software, Other,
}

pub enum Quality { P480, P720, P1080, P4K, Other(String) }

pub enum Language { FR, VOSTFR, EN, Multi, Other(String) }

pub enum ContentLink {
    Magnet(String),
    TorrentUrl(String),
    InfoHash([u8; 20]),
}
```

`ContentId` est un hash stable (BLAKE3) calculé sur les champs canoniques de l'entry. Permet la déduplication cross-source : si deux commus publient le même film avec des metadata légèrement différentes mais le même magnet, l'app détecte le doublon.

### Source — la connexion à un backend

```rust
pub struct Source {
    pub id: SourceId,                   // UUID local
    pub kind: SourceKind,
    pub endpoint: String,               // URL, path, IP:port selon kind
    pub display_name: String,           // "Anime FR", "Pote Jean", ...
    pub description: Option<String>,
    pub auth: Option<AuthConfig>,       // si la source est privée
    pub sync_policy: SyncPolicy,        // fréquence de pull, etc.
    pub last_sync: Option<DateTime<Utc>>,
    pub last_status: SyncStatus,
}

pub enum SourceKind {
    LocalFolder,
    HttpUrl,
    GitRepo,
    GoogleDrive,
    Dropbox,
    OneDrive,
    Server,                             // chat server, IP:port
    Nostr,                              // relay URL
    Ipfs,                               // IPNS/CID
}
```

### Community — wrapper sémantique sur une source

Dans la première implémentation, **une Source = une Community**. On peut introduire plus tard une indirection (plusieurs sources composant une commu, métadonnées partagées) si le besoin émerge.

### Pool — agrégation de communautés

```rust
pub struct Pool {
    pub id: PoolId,
    pub name: String,                   // "Mes Films", "Anime"
    pub members: Vec<SourceId>,         // les commus inclues
    pub filters: PoolFilters,           // catégories, tags, qualité, langue à appliquer
    pub dedup_strategy: DedupStrategy,  // par défaut: par ContentId
    pub conflict_strategy: ConflictStrategy, // que faire si la même entry vient de N commus
}
```

## Identité cryptographique

- **Algorithme** : Ed25519 (rapide, sûr, signatures courtes 64 octets).
- **Génération** : au premier lancement de l'app, une paire de clés est générée localement. La clé privée est stockée dans le keyring de l'OS (libsecret / Keychain / Credential Manager) ou en fichier chiffré.
- **Identité publique** = la clé publique encodée en `npub1...` (compatible Nostr).
- **Signature** : chaque Entry et chaque message de chat est signé. La vérification est gratuite côté lecteur.
- **Modération** : une commu maintient (dans son backend) une liste de clés bannies. L'app filtre côté client.
- **Portabilité** : la même clé fonctionne sur toutes les sources et tous les chats. Si l'utilisateur change de PC, il importe sa clé.

Détails dans [docs/identity.md](./docs/identity.md).

## Sources : protocole d'adapter

Chaque adapter implémente :

```rust
pub trait SourceAdapter: Send + Sync {
    async fn fetch_entries(&self, since: Option<DateTime<Utc>>) -> Result<Vec<Entry>>;
    async fn publish_entry(&self, entry: &Entry) -> Result<()>;     // si write supporté
    async fn fetch_metadata(&self) -> Result<CommunityMetadata>;     // nom, description, modos
    fn capabilities(&self) -> SourceCapabilities;                    // read, write, watch, etc.
}
```

Format de fichier standard pour les sources file-based (Local, HTTP, Git, Drive…) :
**JSON Lines** (`.jsonl`), une entry par ligne, chaque ligne = un objet `Entry` sérialisé.
Avantages : append-only naturel, streaming-friendly, diff-friendly pour Git.

Détails dans [docs/sources.md](./docs/sources.md).

## Chat / messagerie

- **Server-only.** Le chat tourne sur des serveurs autonomes hébergés par les modérateurs d'une commu (ou un membre).
- **L'utilisateur est toujours client.** Aucun port ouvert chez lui.
- **Protocole** : WebSocket sur TLS (axum côté serveur, tokio-tungstenite côté client). Messages JSON signés ed25519.
- **Identité** = même clé que pour signer les entries.
- **Forum** (threads asynchrones) : peut être implémenté côté file-based aussi (Issues GitHub, dossier append-only sur Drive). Le distinguo est : chat live = server, forum async = n'importe quelle source qui supporte l'append.

Détails dans [docs/chat-protocol.md](./docs/chat-protocol.md) (à venir).

## Storage

- **SQLite** local (un fichier par installation).
- Tables principales :
  - `entries` (toutes les entries vues, dédupées par `id`)
  - `entry_sources` (n-n : quelle entry vient de quelle source)
  - `sources` (les sources configurées)
  - `pools` (les pools définis par l'utilisateur)
  - `pool_sources` (n-n)
  - `identity` (la clé publique locale et metadata)
  - `bans` (les pubkeys bannies, par source)
  - `messages` (cache local des messages de chat)
- **FTS5** (extension SQLite) pour la recherche full-text rapide sur titres et tags.

## Stack technique détaillée

### Workspace Rust

| Crate | Rôle |
|---|---|
| `tt-core` | types de domaine (`Entry`, `Source`, `Pool`, `Community`), logique de dédup et de filtrage |
| `tt-sources` | implémentations des `SourceAdapter` (un module par kind) |
| `tt-identity` | génération + stockage clés, signing, verification |
| `tt-storage` | SQLite, migrations, requêtes |
| `tt-chat` | client WebSocket, gestion des connexions multi-server |
| `tt-cli` | binaire CLI (Phase 1 MVP) |

### Apps

| App | Rôle |
|---|---|
| `apps/desktop` | Tauri 2 + Svelte 5 — l'app desktop principale |
| `apps/chat-server` | binaire séparé pour héberger un chat (axum) — destiné aux modos d'une commu |

### Dépendances clés

- `tokio` — async runtime
- `serde` / `serde_json` — sérialisation
- `rusqlite` (avec features `bundled`, `serde_json`, `chrono`) ou `sqlx` (async)
- `reqwest` — HTTP client
- `gix` — Git (pure Rust)
- `nostr-sdk` — relays Nostr
- `ed25519-dalek` — signing
- `blake3` — content hash
- `axum` — chat server
- `tauri` v2 — desktop shell
- `clap` — CLI parsing
- `tracing` + `tracing-subscriber` — observabilité
- `thiserror` + `anyhow` — gestion d'erreur

## Sécurité

- **Clés privées** jamais en clair sur disque (keyring OS ou fichier AES-GCM avec passphrase).
- **Toutes les contributions signées** ; l'app refuse les entries non signées par défaut.
- **Pas d'exécution de contenu** — l'app ne télécharge aucun fichier .torrent automatiquement, elle se contente de transmettre des magnets au client externe.
- **Sandbox Tauri** strict : capabilities minimales, allowlist explicite des commands.
- **License MIT** : tout reste auditable et redistribuable.

## Décisions architecturales explicites

- **Pas de PHP/Node backend** : tout en Rust, single-language stack pour le backend.
- **SQLite plutôt que Postgres** : pas de service à démarrer, embarqué, suffisant pour le scope.
- **Tauri plutôt qu'Electron** : binaire 5–10 MB vs 100+ MB, perf natives, sécurité renforcée.
- **Svelte plutôt que React** : moins de boilerplate, bundle plus petit, syntaxe plus lisible.
- **Ed25519 plutôt que RSA** : signatures plus courtes, plus rapide, standard moderne.
- **JSON Lines plutôt que JSON ou XML** : append-only natif, diff-friendly.
- **MIT plutôt qu'AGPL** : license permissive choisie pour maximiser l'adoption et faciliter la redistribution sous toutes ses formes.

## Hors-scope (volontairement)

- Pas de client BitTorrent intégré au lancement (on s'appuie sur qBittorrent / Transmission). Une intégration `librqbit` est possible plus tard.
- Pas de DRM, pas de paiement, pas d'analytics.
- Pas de mobile au début (Linux / Win / macOS d'abord). Tauri Mobile pourra venir plus tard.
- Pas de support du tracker BitTorrent technique en hébergement local (hors-scope du projet — `opentracker` existe déjà pour ça).
