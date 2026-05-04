# Roadmap

Plan de livraison phasé. Chaque phase a un livrable concret et une *Definition of Done* (DoD).

## Phase 0 — Spec & Skeleton (en cours)

**Objectif :** poser les fondations textuelles et techniques. Aligner la vision pour éviter les divergences ultérieures.

**Livrables :**
- README.md
- ARCHITECTURE.md
- ROADMAP.md (ce document)
- CONTRIBUTING.md
- LICENSE (MIT)
- docs/ : data-model.md, identity.md, sources.md, ui-design.md
- `Cargo.toml` workspace + crates skeleton (stubs `lib.rs`)
- `.gitignore`, `rust-toolchain.toml`
- Repo git initialisé

**DoD :** `cargo check` passe sur le workspace vide. Toute la vision tient dans les docs sans dépendre de la mémoire d'une conversation.

---

## Phase 1 — Core + CLI MVP

**Objectif :** valider le modèle de données et les flux essentiels avec un CLI utilisable de bout en bout.

**Livrables :**
- `tt-core` :
  - Types `Entry`, `Source`, `Pool`, `Community`
  - Fonctions de dédup (`ContentId` via BLAKE3)
  - Filtres (catégorie, tags, qualité, langue, taille)
- `tt-storage` :
  - SQLite + migrations (refinery ou sqlx-migrate)
  - Schéma : entries, sources, pools, entry_sources, pool_sources
  - FTS5 pour le full-text search
- `tt-sources` :
  - Adapter `LocalFolder` (lit `.jsonl` d'un dossier)
  - Adapter `HttpUrl` (GET d'un fichier `.jsonl`)
- `tt-cli` :
  - `tt source add <kind> <endpoint>` — enregistre une source
  - `tt source list` / `tt source sync` / `tt source remove`
  - `tt pool create <name> --sources <ids>`
  - `tt search <query> [--in <pool|source>] [--category <cat>] [--quality <q>]`
  - `tt open <entry-id>` — lance le magnet dans qBittorrent (Linux: `xdg-open`)
- Tests unitaires sur dédup, filtrage, parsing.
- Intégration test avec une fixture `.jsonl` de 100 entries.

**DoD :** un utilisateur peut, via CLI, ajouter 2 sources locales, faire une recherche, et lancer un magnet dans son client torrent.

---

## Phase 2 — Identité + sources avancées

**Objectif :** introduire l'identité cryptographique et les sources distantes plus complexes.

**Livrables :**
- `tt-identity` :
  - Génération paire ed25519
  - Stockage clé privée dans le keyring OS (crate `keyring`) avec fallback fichier chiffré
  - Signing / verification d'`Entry`
  - Format `npub1...` (bech32) pour la clé publique
- `tt-sources` (extension) :
  - Adapter `GitRepo` (clone + pull via `gix`)
  - Adapter `Nostr` (relay via `nostr-sdk`)
  - Adapter `GoogleDrive` (REST API + OAuth device flow)
- `tt-core` :
  - Système de bans (par pubkey, par source)
  - Vérification automatique des signatures à la lecture
  - Conflict resolution dans les pools (même entry dans plusieurs commus)
- `tt-cli` :
  - `tt identity init` / `tt identity show` / `tt identity export`
  - `tt source add git <repo-url>` / `tt source add nostr <relay-url>`
  - `tt ban <pubkey> --in <source>`
  - Publish : `tt publish <magnet> --to <source-id>` (signe et pousse)

**DoD :** un utilisateur peut publier une entry signée dans son repo Git, un autre utilisateur la fetch et vérifie la signature.

---

## Phase 3 — Tauri Desktop UI

**Objectif :** rendre l'app accessible aux non-CLI. Implémenter le design moderne défini dans `docs/ui-design.md`.

**Livrables :**
- `apps/desktop` setup Tauri 2 + Svelte 5 + Tailwind v4 + shadcn-svelte + Lucide icons
- IPC Tauri ↔ `tt-core` (commands typées via `tauri-specta` ou `ts-rs`)
- Layout :
  - Sidebar gauche : catégories (Films / Séries / Jeux / Musique / Livres / Logiciels) → puis communautés (icônes) → puis pools
  - Header : barre de recherche avec dropdown scope, identité, settings
  - Main : grille/liste de résultats avec badge `[commu]`
  - Filtres latéraux collapsible (qualité, langue, taille)
- Command palette ⌘K (style Raycast)
- Dark mode natif
- Settings panel : gestion sources, gestion identité, préférences
- Onboarding : génération de l'identité au premier lancement

**DoD :** l'app desktop se build sur Linux, Win, macOS. Un nouvel utilisateur peut, depuis l'UI seule, ajouter une source HTTP, chercher, lancer un magnet.

---

## Phase 4 — Chat & messagerie

**Objectif :** ajouter le canal de communication communautaire.

**Livrables :**
- `tt-chat` :
  - Client WebSocket (tokio-tungstenite + TLS via rustls)
  - Gestion multi-server simultanée
  - Cache local des messages dans SQLite
  - Subscriptions, notifications, threads
- `apps/chat-server` :
  - Binaire standalone axum, déployable par les modos d'une commu
  - Authentification par signature ed25519 (pas de mot de passe)
  - Modération : ban par pubkey, mute, etc.
  - Persistance SQLite côté serveur
  - Config TOML simple
- UI desktop :
  - Vue chat (panneau latéral droit ou onglet)
  - Vue forum (threads asynchrones)
  - Notifications

**DoD :** deux utilisateurs sur deux machines peuvent rejoindre le même `chat-server` et discuter en temps réel, avec messages signés et vérifiés.

---

## Phase 5 — Polish & release

**Objectif :** stabiliser, documenter, distribuer.

**Livrables :**
- Filtres avancés et tri configurable
- Themes / customisation UI
- Plugins API (pour des sources tierces communautaires)
- Cross-platform builds CI : GitHub Actions → artefacts Linux (.deb, .AppImage), Windows (.msi), macOS (.dmg)
- Documentation utilisateur complète (site statique avec mdbook ou Docusaurus)
- Migration `librqbit` (client BitTorrent intégré, optionnel)
- Auto-update Tauri
- Crash reporting opt-in

**DoD :** version 1.0 publique, installable en un clic sur les trois OS, avec docs complètes.

---

## Critères transverses (toutes phases)

- **Tests** : couverture > 70 % sur `tt-core`, > 50 % sur les autres crates
- **Lint** : `cargo clippy --all-targets -- -D warnings` doit passer
- **Format** : `cargo fmt --all --check`
- **CI** : un workflow GitHub Actions valide chaque PR (build, test, lint, format)
- **Sémantique versioning** : à partir de la 1.0
- **Changelog** : format Keep a Changelog maintenu en continu

## Déps inter-phases

```
Phase 0  ─►  Phase 1  ─►  Phase 2  ─►  Phase 3  ─►  Phase 4  ─►  Phase 5
              (core)      (identité)    (UI)        (chat)       (release)
                                  │           │
                                  └───────────┘  Phase 3 dépend faiblement
                                                 de Phase 2 (peut commencer en parallèle)
```

Phase 3 peut démarrer en parallèle de Phase 2 si l'identité côté UI est mockée temporairement.
