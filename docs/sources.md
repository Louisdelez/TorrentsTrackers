# Sources

Spec des adapters de sources. Implémenté dans `crates/sources`.

## Trait commun

```rust
#[async_trait]
pub trait SourceAdapter: Send + Sync {
    fn kind(&self) -> SourceKind;
    fn capabilities(&self) -> SourceCapabilities;

    /// Récupère les entries depuis le backend.
    /// `since` = timestamp du dernier sync (incremental si supporté).
    async fn fetch_entries(&self, since: Option<DateTime<Utc>>)
        -> Result<Vec<Entry>>;

    /// Récupère les métadonnées de la communauté.
    async fn fetch_metadata(&self) -> Result<CommunityMetadata>;

    /// Publie une entry signée. Échoue si la source n'a pas la capability Write.
    async fn publish_entry(&self, entry: &Entry) -> Result<()>;

    /// Récupère la liste des bans (pubkeys bannies par les modos).
    async fn fetch_bans(&self) -> Result<Vec<Ban>>;
}

pub struct SourceCapabilities {
    pub read: bool,
    pub write: bool,
    pub watch: bool,                  // notifications push (websocket, sse, etc.)
    pub incremental_sync: bool,       // supporte le `since`
    pub authenticated: bool,
}
```

## Format de fichier standard

Pour toutes les sources file-based (Local, HTTP, Git, Drive, Dropbox), le format canonique est **JSON Lines** (`.jsonl`).

Une ligne = une entry sérialisée en JSON.

```jsonl
{"id":"...","title":"Naruto Shippuden VOSTFR 1080p","link":{"Magnet":"magnet:?..."},"category":"Series","tags":["1080p","vostfr"],"quality":"P1080","languages":["VOSTFR"],"size_bytes":53687091200,"added_at":"2026-04-12T18:30:00Z","contributor_pubkey":"...","source_id":"...","signature":"..."}
{"id":"...","title":"Inception 4K REMUX","link":{"Magnet":"magnet:?..."},"category":"Films","tags":["4k","remux"],"quality":"P2160","languages":["Multi"],"size_bytes":85899345920,"added_at":"2026-04-13T09:12:00Z","contributor_pubkey":"...","source_id":"...","signature":"..."}
```

**Pourquoi JSON Lines :**
- Append-only naturel (ajout en fin de fichier sans réécriture)
- Streaming-friendly (parse ligne par ligne, mémoire constante)
- Diff-friendly côté Git (un changement = une ligne ajoutée)
- Simple à manipuler manuellement (tail, grep, jq)

Fichiers attendus dans une source file-based :

```
<source root>/
├── community.json     # métadonnées (CommunityMetadata)
├── entries.jsonl      # toutes les entries
├── bans.jsonl         # pubkeys bannies (signed)
└── (optionnel)
    ├── modos.json     # liste des pubkeys de modos
    └── icon.png
```

## Adapter par adapter

### LocalFolder

- **Endpoint** : chemin absolu vers le dossier source.
- **Capabilities** : read, write, incremental (basé sur `mtime` des fichiers).
- **Implémentation** : lecture directe des fichiers, parsing streaming via `tokio::io::BufReader`.
- **Sync policy** : watch via `notify` crate pour push automatique des changements.

### HttpUrl

- **Endpoint** : URL HTTPS vers `entries.jsonl` (ou un dossier listable).
- **Capabilities** : read seulement, incremental via header `If-Modified-Since`.
- **Auth** : optionnel `Authorization: Bearer <token>` ou Basic Auth.
- **Implémentation** : `reqwest` avec streaming response.
- **Cas d'usage** : raw GitHub, Pastebin, fichier sur un CDN perso.

### GitRepo

- **Endpoint** : URL clonable (`https://github.com/anime-fr/list-vf.git`).
- **Capabilities** : read (clone+pull), write (commit+push si auth fournie).
- **Auth** : token GitHub/GitLab pour le push.
- **Implémentation** : `gix` (pure Rust). Premier sync = clone shallow, syncs suivants = pull. L'app stocke le clone dans `~/.local/share/torrents-trackers/sources/<id>/`.
- **Avantages** : versioning gratuit, signature gpg/ssh native du commit en bonus, fork = clone.

### GoogleDrive / Dropbox / OneDrive

- **Endpoint** : ID du dossier partagé.
- **Capabilities** : read, write, incremental (delta API).
- **Auth** : OAuth 2.0 device flow (l'utilisateur valide dans son navigateur, pas besoin de back-end de redirect).
- **Implémentation** : REST API directe via `reqwest` (pas de SDK officiel Rust polish — code custom mais limité à ~10 endpoints).
- **Cas d'usage** : commus de potes, partage simple sans setup technique.

### Server (custom)

- **Endpoint** : `tt://<host>:<port>` (protocole custom WebSocket/HTTP).
- **Capabilities** : read, write, watch (push live), authenticated.
- **Auth** : signature ed25519 (clientside).
- **Implémentation** : `tonic` ou `axum` + WebSocket pour les notifs push.
- **Cas d'usage** : grosse commu qui veut un backend dédié, latence basse, modération en temps réel.

### Nostr

- **Endpoint** : `wss://relay.example.com`.
- **Capabilities** : read, write, watch (subscriptions), incremental.
- **Auth** : signature ed25519 (l'identité Nostr **est** notre identité).
- **Implémentation** : `nostr-sdk`. Les entries sont publiées comme événements custom kind (à choisir, ex: `kind: 30001` pour replaceable list events, ou un kind propre du projet).
- **Cas d'usage** : décentralisation maximale, pas de single-point-of-failure, on-chain-style mais sans blockchain.

### IPFS

- **Endpoint** : IPNS name ou CID racine.
- **Capabilities** : read, write (avec un node local ou un service de pinning), watch (via pubsub).
- **Implémentation** : crate `ipfs-api` ou requêtes directes au daemon IPFS local.
- **Cas d'usage** : permanence du contenu (les entries restent même si le contributeur original disparaît, tant que le CID est pinné quelque part).

## Sync orchestration

`tt-sources::SyncOrchestrator` est responsable de :

- Lancer les fetch en parallèle (avec limite de concurrence configurable).
- Respecter les `SyncPolicy` (intervals, bandwidth limits).
- Detecter les erreurs et les retries (backoff exponentiel).
- Vérifier les signatures des entries fetchées avant insertion en base.
- Émettre des événements (via `tokio::sync::broadcast`) pour que l'UI puisse afficher l'état des syncs.

```rust
pub enum SyncEvent {
    Started { source_id: SourceId },
    Progress { source_id: SourceId, fetched: usize },
    Finished { source_id: SourceId, total: usize },
    Failed { source_id: SourceId, error: String },
}
```

## Découverte de nouvelles sources

Mécanismes pour découvrir des nouvelles communautés à brancher :

1. **Manuel** : l'utilisateur entre l'endpoint à la main (le mode par défaut).
2. **QR code / lien `tt://`** : un lien custom scheme qu'on partage. L'app le parse et propose d'ajouter la source.
3. **Annuaire optionnel** : un repo GitHub communautaire `awesome-tt-sources` avec une liste curée. L'app peut le pull pour proposer des sources.
4. **Via une commu existante** : une source peut publier dans son `community.json` une liste de "communautés alliées" (`recommended_sources`). L'app peut suggérer.

Aucun de ces mécanismes n'est obligatoire — le mode 100 % manuel reste premier-class.
