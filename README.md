# TorrentsTrackers

> Découverte de torrents fédérée, locale et sans site web.
> Une app desktop qui remplace Pirate Bay, YGG, 1337x — en branchant des sources que tu choisis.

**Status :** alpha (`v0.1.0`) — feature-complete on the planned scope; awaiting community testing.
**License :** MIT
**Stack :** Rust · Tauri 2 · Svelte 5 · SQLite

---

## TL;DR

Tu en as marre des sites de torrents qui changent de domaine, qui se font phisher, qui tombent du jour au lendemain ?
TorrentsTrackers est une **app desktop locale** qui agrège des **sources de listes de torrents** depuis n'importe où — un dossier local, un repo GitHub, un Drive partagé, un serveur P2P. Chaque source est une **communauté** avec ses règles, ses modos, ses contributeurs. Tu peux les combiner en **pools** pour te construire ta propre méga-base de découverte.

Tu cherches, tu cliques, ça lance ton client torrent existant (qBittorrent, Transmission). Plus jamais de navigateur sur des sites louches.

## Concepts clés

- **Source** = un endroit qui sert des listes (LocalFolder, HTTP URL, GitHub repo, Drive, Dropbox, server custom, Nostr…)
- **Communauté** = une instance précise de source (un repo donné, un dossier Drive donné). C'est le **serveur Discord** du système : règles, modos, accès, identité propres.
- **Pool** = combinaison user-définie de N communautés, avec déduplication, filtres, et provenance traçable.
- **Identité** = paire de clés cryptographique locale (ed25519). Tu signes tes contributions, tu es identifié partout par ta clé publique.
- **Chat / forum** (optionnel) = serveur autonome rejoint par IP:port. Toi tu es client, tu n'héberges jamais rien.

## Pourquoi

Les indexes de torrents traditionnels souffrent de problèmes structurels :

- Dépendance à un nom de domaine et à un hébergement (= takedown, saisie, blocage FAI).
- Phishing massif via faux clones.
- Modération centralisée fragile.
- Pas d'identité utilisateur portable d'un site à l'autre.

TorrentsTrackers déplace tout ça vers un modèle **fédéré et local-first** :

- **Pas de site web** à héberger ni à protéger.
- **Pas de domaine** à gérer.
- **La modération est physique** : qui a accès en écriture à la source = qui peut publier.
- **Une communauté peut fork librement** en clonant son backend (repo, Drive…).
- **Aucun single point of failure**.

## Fonctionnalités prévues

- Recherche multi-scope (tous, un pool, une commu, syntaxe inline `in:` `category:`)
- Catégories canoniques (Films, Séries, Jeux, Musique, Livres, Logiciels) + tags freeform
- Filtres : qualité (1080p / 4K), langue (VF / VOSTFR), taille, date, commu
- Lancement direct dans qBittorrent (ou client compatible)
- Identité crypto unifiée (ed25519)
- Chat / forum (server-based, optionnel)
- Mode 100 % offline (juste fichiers locaux)
- Cross-platform : Linux, Windows, macOS

## Architecture (résumé)

```
[ Sources ]    LocalFolder · HTTP · Git · Drive · Server · Nostr · IPFS
                          |
                          v
[ Communautés ]   un endpoint = une commu (règles, modos, accès)
                          |
                          v
[ Pools ]      combinaisons user-définies, dédup + filtres
                          |
                          v
[ UI ]         sidebar (catégories + commus + pools) + search + browse
                          |
                          v
[ Action ]     ouvre le magnet dans qBittorrent
```

Détails dans [ARCHITECTURE.md](./ARCHITECTURE.md).

## Stack technique

| Couche | Choix |
|---|---|
| Backend | Rust (édition 2024) |
| Async runtime | tokio |
| Storage local | SQLite via rusqlite/sqlx |
| Identité | ed25519-dalek |
| HTTP | reqwest |
| Git source | gix |
| Nostr | nostr-sdk |
| Desktop shell | Tauri 2 |
| Frontend | Svelte 5 + Tailwind v4 + shadcn-svelte + Lucide |
| Chat server (séparé) | axum |

## Structure du dépôt

```
TorrentsTrackers/
├── crates/
│   ├── core/         logique métier (Entry, Source, Community, Pool)
│   ├── sources/      adapters par type (local, http, git, nostr, server)
│   ├── identity/     keypair, signing, verification
│   ├── storage/      SQLite + migrations
│   ├── chat/         client de messagerie
│   └── cli/          binaire CLI (MVP phase 1)
├── apps/
│   ├── desktop/      app Tauri + Svelte (phase 3)
│   └── chat-server/  binaire serveur de chat (phase 4)
└── docs/             specs détaillées
```

## Install

Pre-built bundles are produced by CI on every `v*` tag and uploaded to
the [GitHub Releases](https://github.com/loicdelez/TorrentsTrackers/releases)
page.

| Platform | Artefact | Size |
|---|---|---|
| Debian / Ubuntu | `TorrentsTrackers_<v>_amd64.deb` | ~5 MB |
| Fedora / RHEL | `TorrentsTrackers-<v>-1.x86_64.rpm` | ~5 MB |
| Linux (any) | `TorrentsTrackers_<v>_amd64.AppImage` | ~76 MB |
| macOS | `TorrentsTrackers_<v>_aarch64.dmg` | TBA |
| Windows | `TorrentsTrackers_<v>_x64-setup.exe` | TBA |

The CLI (`tt`) and the chat server (`tt-chat-server`) ship as separate
single-file binaries (also on the release page).

See [docs/quickstart.md](./docs/quickstart.md) for the full first-run
walkthrough.

## Roadmap

Voir [ROADMAP.md](./ROADMAP.md) et [CHANGELOG.md](./CHANGELOG.md).

| Phase | Statut |
|---|---|
| 0 — Spec & skeleton | done |
| 1 — Core + CLI MVP | done |
| 2 — Identité + sources avancées | done |
| 3 — Tauri desktop UI | done |
| 3-bis — Polish UI (palette, filtres, publish) | done |
| 4 — Chat & messagerie | done |
| 5 — Polish & release | done — 0.1.0 |

## Contribuer

Voir [CONTRIBUTING.md](./CONTRIBUTING.md).

## Note sur le nom

`TorrentsTrackers` est un placeholder. Un nom de produit plus court / mémorisable sera choisi avant la première release publique (suggestions : *Tide*, *Cove*, *Shoal*, *Drift*…).

## License

MIT — voir [LICENSE](./LICENSE).
License permissive : libre d'usage, de modification, de redistribution, y compris commerciale.
