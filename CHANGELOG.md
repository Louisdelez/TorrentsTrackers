# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
starting at `1.0.0`.

## [Unreleased]

## [0.1.0] — 2026-05-04

First public alpha. Everything ships as a single workspace plus two app
crates (`apps/desktop`, `apps/chat-server`). Linux is the only fully-tested
target so far; macOS and Windows are wired up in CI but unverified.

### Added

#### Core / data model (`crates/core`)
- Canonical domain types: `Entry`, `ContentLink`, `Category`, `Quality`,
  `Language`, `Source`, `SourceKind`, `Pool`, `PoolFilters`,
  `CommunityMetadata`, `Ban`.
- `ContentId` (BLAKE3 over normalized title + info_hash) for cross-source
  dedup.
- `magnet` module: parse hex/base32 BTv1 info_hash, build minimal magnet URIs.
- `parse` module: `normalize_title`, `parse_quality`, `parse_languages`.
- `filter` module: `PoolFilters::matches(&Entry)`.
- `signing` module: deterministic length-prefixed canonical payload (`tt-entry-v1\0`),
  `verify_entry` / `verify_with` (no private key required).
- Async `SourceAdapter` trait + `SourceCapabilities`.

#### Identity (`crates/identity`)
- `LocalKeypair` (ed25519, zeroized on drop), `npub` / `nsec` bech32
  (NIP-19 compatible).
- `sign_entry` (uses tt-core canonical payload).
- `portable` AES-256-GCM + scrypt(log2_n=15) backup format (`tt-id-v1`).
- `IdentityStore` trait, `FileStore` (mode 0600 under
  `$XDG_CONFIG_HOME/torrents-trackers/`), `KeyringStore`,
  `DefaultStore` (file-backed in this release).

#### Storage (`crates/storage`)
- SQLite (WAL + foreign keys + NORMAL sync) with versioned migrations.
- Schema v1: sources, pools, pool_sources, entries (+ `primary_source_id`),
  entry_sources (provenance), bans, identity, FTS5 virtual table on titles
  with INSERT/UPDATE/DELETE triggers.
- Typed CRUD: insert/list/get/delete sources, upsert_entry (verifies
  signature + bans), upsert_entry_unverified (fixtures), insert/list/delete
  pools, count_entries, update_source_sync_status.
- Bans: add, remove, is_banned, list, replace.
- Identity: put / get / clear local row.
- `SearchQuery` / `SearchScope` / `SearchHit` with provenance lookup,
  text+category+quality+language+size+seeders filters.
- `paths` (XDG-compliant `data_dir`, `db_path`).

#### Source adapters (`crates/sources`)
- `LocalFolder`: read/append `entries.jsonl`, read `community.json` and
  `bans.jsonl`, dir-name fallback for metadata.
- `HttpUrl`: GET endpoint as a `.jsonl` file URL or directory URL,
  optional bearer token, conditional GET via `If-Modified-Since`.
- `GitRepo`: shells out to system `git` for shallow clone / fetch / reset
  --hard, then reads through `LocalFolder`. Read-only.
- `examples/seed.rs`: signs fixtures with a generated keypair so they pass
  the post-Phase 2 verification path.

#### Chat (`crates/chat` + `apps/chat-server`)
- Wire protocol `tt-chat-v1`: WebSocket JSON, `kind`-tagged envelopes.
- Auth: server `auth_challenge { nonce_hex }` → client `hello { pubkey,
  signature(domain || nonce) }` → `auth_accepted | auth_rejected`.
- Per-message ed25519 signature (`tt-chat-msg-v1\0`-framed payload).
- `ChatClient`: connect + handshake, subscribe / unsubscribe / send_text /
  fetch_history, background task pumping events to a `mpsc::Receiver`.
- `tt-chat-server` standalone binary: TOML config (`bind`, `server_id`,
  `server_name`, `db_path`, `history_default_limit`, `rate_limit_per_min`),
  WAL SQLite (`messages` indexed by `channel, sent_at`; `bans`), tokio
  broadcast fan-out, server-side signature re-verification, per-connection
  rate limiter.
- `examples/chat_smoke.rs` end-to-end roundtrip demo.

#### CLI (`tt`)
- `source add | list | sync | remove`
- `pool create | list | remove`
- `search [text] [--in source:<p>|pool:<p>] [--category] [--quality]
        [--language] [--min-size] [--max-size] [--min-seeders] [--limit]`
- `open <id-prefix> [--print]`
- `identity init | show | export | import | forget`
- `publish <magnet> --to <source> ...` (signs + pushes via adapter)
- `ban add | remove | list --in <source>`
- `info`, `--db <path>` global override.

#### Desktop app (`apps/desktop`, Tauri 2 + Svelte 5 + Tailwind v4)
- IPC commands wrapping every CLI flow plus chat (`chat_list`,
  `chat_connect`, `chat_disconnect`, `chat_send`, `chat_history`).
- Layout: sidebar (Categories / Communities / Pools), header (search,
  scope, sync, identity, settings, chat toggle).
- Components: `Sidebar`, `Header`, `Browse`, `EntryRow`, `Settings`,
  `Onboarding`, `AddSourceDialog`, `CreatePoolDialog`, `PublishDialog`,
  `IdentityBackupDialog`, `FilterPanel`, `CommandPalette`, `ChatView`.
- Keyboard: ⌘K palette, ⌘F filters, ⌘N publish.
- Multi-server chat with per-server message buffer + Tauri-event-based
  realtime stream.

#### Bundles
- Linux: `.deb` (4.8 MB), `.rpm` (4.8 MB), `.AppImage` (76 MB).
- macOS / Windows targets configured (verified via CI).

#### CI
- `.github/workflows/ci.yml` — fmt + clippy + tests + frontend type-check
  on every push and PR.
- `.github/workflows/release.yml` — Tauri bundles + CLI + chat-server
  binaries on `v*` tags, uploaded to the GitHub Release.

[Unreleased]: https://github.com/loicdelez/TorrentsTrackers/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/loicdelez/TorrentsTrackers/releases/tag/v0.1.0
