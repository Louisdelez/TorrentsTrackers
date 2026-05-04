# Contributing

Merci de l'intérêt pour TorrentsTrackers. Ce document décrit comment contribuer au projet.

## Avant de contribuer

1. Lis [README.md](./README.md) pour la vision.
2. Lis [ARCHITECTURE.md](./ARCHITECTURE.md) pour l'architecture technique.
3. Regarde [ROADMAP.md](./ROADMAP.md) pour savoir où on en est.
4. Pour une contribution non-triviale, ouvre une **issue de discussion** avant de coder.

## Setup dev

```sh
# Pré-requis
rustup default stable
rustup component add rustfmt clippy

# Build
cargo build --workspace

# Tests
cargo test --workspace

# Lint
cargo clippy --all-targets -- -D warnings

# Format
cargo fmt --all
```

Pour la phase 3 et au-delà (UI Tauri) :

```sh
# Pré-requis additionnels
# - Node.js 20+
# - pnpm
# - Tauri prerequisites: https://tauri.app/start/prerequisites/

cd apps/desktop
pnpm install
pnpm tauri dev
```

## Structure du code

- Le code métier vit dans `crates/` (workspace Rust multi-crate).
- Les apps (desktop, chat-server) vivent dans `apps/`.
- La doc additionnelle vit dans `docs/`.

Voir [ARCHITECTURE.md](./ARCHITECTURE.md) pour le détail des responsabilités par crate.

## Style de code

- **Rust** : suivre les conventions standards (`rustfmt` + `clippy` strict). Pas de `unwrap()` ou `expect()` en code de production sans justification commentée.
- **Erreurs** : `thiserror` pour les enums d'erreur de bibliothèque, `anyhow` dans les binaires.
- **Async** : tout le code I/O est `async` via tokio. Pas de blocking en async sans `spawn_blocking`.
- **Tests** : un module `#[cfg(test)] mod tests` par fichier non-trivial. Tests d'intégration dans `tests/`.
- **Logs** : utiliser `tracing` (pas `println!` ni `log`).
- **Commentaires** : commenter le *pourquoi*, pas le *quoi*. Le code dit *quoi*.

## Workflow PR

1. Fork (ou branche si tu as les droits)
2. Branche par feature : `feat/short-description` ou `fix/short-description`
3. Commits propres, message en anglais, format Conventional Commits :
   - `feat(core): add dedup by ContentId`
   - `fix(sources): handle 404 in HttpUrl adapter`
   - `docs(readme): clarify quickstart`
4. Vérifie avant de push :
   - `cargo fmt --all --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test --workspace`
5. PR vers `main` avec description : *quoi* + *pourquoi* + *comment tester*.

## Commits signés

Idéalement : signer les commits avec une clé GPG ou SSH (`git config commit.gpgsign true`).
Cohérent avec l'esprit "tout est signé" du projet.

## Domaines où l'aide est bienvenue

- **Adapters de sources** : un nouveau backend (Mastodon, Matrix, IPFS pin services…)
- **Catégorisation auto** : classification automatique d'entries par titre (regex / heuristics / NLP léger)
- **UI Svelte** : composants, animations, thèmes
- **Tests d'intégration** : scénarios bout-en-bout
- **Doc utilisateur** : guides, tutos, captures d'écran
- **Traductions** : i18n (l'UI sera prévue pour ça dès le départ)

## Code de conduite

Sois respectueux. On discute des idées, pas des personnes. Toute forme de harcèlement = ban.

## Questions

Ouvre une **discussion** GitHub (pas une issue) pour les questions ouvertes.
Une **issue** pour les bugs concrets ou les propositions de feature précises.
