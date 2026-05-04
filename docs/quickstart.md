# Quickstart

Full installation + first-run guide for end users. Five minutes to a
working setup.

## 1. Install

### Linux (Debian / Ubuntu)

```sh
wget https://github.com/Louisdelez/TorrentsTrackers/releases/latest/download/TorrentsTrackers_0.1.0_amd64.deb
sudo apt install ./TorrentsTrackers_0.1.0_amd64.deb
```

### Linux (Fedora / RHEL)

```sh
wget https://github.com/Louisdelez/TorrentsTrackers/releases/latest/download/TorrentsTrackers-0.1.0-1.x86_64.rpm
sudo dnf install ./TorrentsTrackers-0.1.0-1.x86_64.rpm
```

### Linux (any distro, AppImage)

```sh
wget https://github.com/Louisdelez/TorrentsTrackers/releases/latest/download/TorrentsTrackers_0.1.0_amd64.AppImage
chmod +x TorrentsTrackers_0.1.0_amd64.AppImage
./TorrentsTrackers_0.1.0_amd64.AppImage
```

### macOS / Windows

Download the matching artefact from the latest
[release page](https://github.com/Louisdelez/TorrentsTrackers/releases) —
`.dmg` for macOS, `.msi` or `-setup.exe` for Windows.

## 2. First launch — onboarding

1. The app prompts you to **create a local identity** (ed25519 keypair).
   Pick an optional pseudonym; the keypair lives encrypted under
   `$XDG_CONFIG_HOME/torrents-trackers/identity.key` (mode 0600 on Unix).
2. **Back it up immediately** : Settings → Identité → Sauvegarder. Pick
   a passphrase. Without a backup, losing the machine = losing the
   identity (your past contributions stay verifiable, but you can't
   sign new ones from anywhere else).

## 3. Add a source

A *source* is one community's catalog. Three ways to add one:

| Kind | Endpoint example | Read | Write |
|---|---|---|---|
| `local` | `/home/you/MesListes` | yes | yes |
| `http` | `https://raw.githubusercontent.com/anime-fr/list-vf/main/entries.jsonl` | yes | no |
| `git` | `https://github.com/anime-fr/list-vf.git` | yes | no |

Click the `+` button in the sidebar (or hit ⌘K → "Ajouter une source"),
pick a kind, paste the endpoint.

Then click the sync button in the header (or ⌘K → "Synchroniser
toutes les sources").

## 4. Search and launch

Type in the search bar at the top. Hit ⌘F to open the filter side panel
(quality, language, size, min seeders, source set). Double-click a row
to launch its magnet in your default torrent client (qBittorrent,
Transmission, Deluge — anything that registers `magnet:` on your OS).

## 5. Publish your own contribution

⌘N or `tt publish` from the CLI:

```sh
tt publish "magnet:?xt=urn:btih:..." --to <source-id-prefix> \
   --title "My File 1080p" --category films --quality 1080p \
   --language vostfr --size 5368709120
```

The entry is signed with your local key, written to the source backend,
and immediately verified + indexed in your local catalog.

## 6. Connect a chat server

A community can run a `tt-chat-server` (separate binary). To join:

1. In the desktop app, click the chat icon in the header (or change
   the view via ⌘K → "Ouvrir les paramètres" → … or just toggle).
2. Enter the URL: `ws://host:6970/ws` (or `wss://...` for TLS).
3. The server challenges your key, you accept, you're in.

To **run** a chat server yourself:

```sh
tt-chat-server --init --config tt-chat-server.toml    # generates a starter
# edit bind / server_name in the file
tt-chat-server --config tt-chat-server.toml
```

Defaults: TCP `127.0.0.1:6970`, SQLite at `tt-chat-server.sqlite`,
60 messages per minute per connection.

## 7. CLI vs desktop

Everything the desktop app does is also available from the CLI (`tt`).
The CLI is great for headless boxes, scripting, and CI.

```sh
tt --help                 # full subcommand list
tt info                   # paths and DB stats
tt source list
tt search naruto --quality 1080p
tt identity show
```

The desktop app and CLI share the same on-disk database
(`$XDG_DATA_HOME/torrents-trackers/data.sqlite`), so you can mix and
match.

## 8. Troubleshooting

- **App won't start** — try `RUST_LOG=tt_storage=debug,tt_sources=debug
  ./TorrentsTrackers...AppImage` (or set the env var before launching
  the binary). Most failures are missing/corrupt SQLite or an inaccessible
  data dir.
- **Magnets don't open** — make sure your torrent client is the default
  handler for `magnet:` URIs at the OS level.
- **Git source fails to sync** — `tt-chat-server` and the GitRepo
  adapter currently shell out to the system `git`. Install Git
  (`sudo apt install git`).
- **Lost the identity passphrase on a backup** — there's no recovery.
  Generate a new identity. Past contributions remain valid (signed by
  the old key) but you can't write new ones from elsewhere.
