# UI Design

Spec UI/UX de l'app desktop. Implémenté dans `apps/desktop` (Tauri 2 + Svelte 5 + Tailwind v4 + shadcn-svelte + Lucide).

## Principes design

1. **Moderne, dense mais aéré.** Inspirations : Raycast, Linear, Arc, Notion, Discord.
2. **Dark mode natif** — c'est le mode par défaut, light mode optionnel.
3. **Clavier-first.** ⌘K (ou Ctrl+K) ouvre la command palette globale.
4. **Pas de bordures inutiles** — utiliser les contrastes de background pour structurer.
5. **Micro-interactions soignées** — hover states, skeletons de loading, transitions via View Transitions API.
6. **Typographie premium** — Inter ou Geist (jamais Arial / system font).

## Layout principal

```
+-------------------------------------------------------------------------+
|  ⌘K  [naruto                         ]  Dans:[Tous ▾]    @user  ⚙       |
+-------------+-----------------------------------------------------------+
| CATÉGORIES  |                                                           |
|   Films     |   Naruto Shippuden VOSTFR 1080p           [anime-fr]      |
|   Séries    |   60 GB · 1080p · VOSTFR · 234 seeders · ajouté il y a 2j |
|   Jeux      |   ─────────────────────────────────────────────────────   |
|   Musique   |   Naruto Movie Pack 4K REMUX              [bluray-fr]     |
|   Livres    |   85 GB · 4K · Multi · 89 seeders · ajouté il y a 5j      |
|   Logiciels |   ─────────────────────────────────────────────────────   |
|   Autre     |   Naruto Manga 1-72 FR                    [manga-com]     |
|             |   2 GB · pdf · FR · 412 seeders · ajouté il y a 1m        |
| ──────────  |                                                           |
| COMMUNAUTÉS |                                                           |
|   ●  ●  ●   |   ╭─ FILTRES ──────────────────────╮                      |
|   ●  ●  +   |   │  Qualité  ☐4K  ☑1080p  ☐720p  │                      |
|             |   │  Langue   ☑VOSTFR ☐VF ☐EN ☐Multi│                    |
| ──────────  |   │  Taille   [▭▭▭▭▭▭▭▭] 0–50 GB   │                     |
| POOLS       |   │  Date     [────●─────] < 30j   │                      |
|   Pool A    |   │  Source   ☑anime-fr ☑manga-com │                      |
|   Pool B    |   ╰────────────────────────────────╯                      |
|   + Nouveau |                                                           |
+-------------+-----------------------------------------------------------+
```

## Sidebar gauche

Structure verticale, trois sections empilées :

### Section CATÉGORIES (en haut)

Navigation principale par type de contenu. Une icône (Lucide) + un label par catégorie.

| Catégorie | Icône Lucide |
|---|---|
| Films | `clapperboard` |
| Séries | `tv` |
| Jeux | `gamepad-2` |
| Musique | `music` |
| Livres | `book-open` |
| Logiciels | `app-window` |
| Autre | `package` |

État actif → background plus clair, accent left border.

### Section COMMUNAUTÉS (milieu)

Grille d'icônes circulaires (40px), 3 par ligne, comme la sidebar Discord.

- Hover → tooltip avec le nom complet et stats (n entries, dernière sync).
- Clic → switch le scope sur cette commu (filtre les résultats).
- Indicateur visuel de statut sync (point vert/jaune/rouge).
- Bouton `+` à la fin pour ajouter une nouvelle source.

### Section POOLS (bas)

Liste compacte des pools définis par l'utilisateur. Drag-and-drop pour réordonner.

- Clic → switch le scope sur ce pool.
- Bouton `+ Nouveau` pour créer un pool.

## Header

```
+--------------------------------------------------------------------+
|  ⌘K  [Search...                  ]  Dans:[Tous ▾]   @user   ⚙     |
+--------------------------------------------------------------------+
```

- **Logo / icône app** à gauche, avec raccourci ⌘K visible.
- **Search bar** centrée, large. Auto-focus au démarrage.
- **Dropdown scope** : Tous / Pool actuel / Commu actuelle / Custom.
- **Avatar identité** à droite (npub tronqué + couleur générée).
- **Settings** : icône engrenage.

### Syntaxe inline dans la search bar (power user)

- `naruto` → recherche dans le scope sélectionné
- `naruto in:anime-fr` → restreint à une commu nommée
- `naruto in:pool-A` → restreint à un pool nommé
- `naruto category:films quality:1080p` → filtres composés
- `from:npub1abc...` → entries d'un contributeur précis
- `tag:vostfr tag:remux` → filtres par tags

## Zone centrale (résultats)

### Liste des résultats (par défaut)

Mode liste compacte, 1 ligne par entry :

- Titre en `text-base font-medium`
- Métadonnées en `text-sm text-muted` : taille · qualité · langue · seeders · date relative
- Badge `[commu]` cliquable à droite
- Hover → background subtil, action contextuelle visible
- Clic → ouvre un panel détail à droite (overlay 400px)
- Double-clic → lance directement le magnet

### Vue grille (toggle)

Mode grille pour les Films/Séries quand poster_url est disponible :

- Cards avec poster, titre, qualité-langue badge en bas
- Layout responsive (auto-fit, minmax(180px, 1fr))

## Filtres latéraux (panneau collapsible à droite)

Cachés par défaut, ouverts via bouton "Filtres" ou ⌘F.

- **Qualité** : checkboxes (480p, 720p, 1080p, 4K)
- **Langue** : checkboxes (VOSTFR, VF, EN, Multi)
- **Taille** : range slider double avec presets (< 1 GB, 1–10 GB, 10–50 GB, > 50 GB)
- **Date d'ajout** : range slider (< 24h, < 7j, < 30j, tout)
- **Source d'origine** : checkboxes par commu (filtre par provenance)
- **Seeders min** : slider (0 → 1000+)
- **Tags** : input multi avec suggestions

État des filtres persisté par scope (chaque commu/pool retient ses filtres).

## Command palette (⌘K)

Style Raycast. Overlay centré, fond flouté.

Actions disponibles :
- Recherche globale (cross-source) — résultats live à la frappe
- Naviguer vers une commu / un pool
- Actions rapides : "Ajouter une source", "Créer un pool", "Sync toutes les sources", "Exporter mon identité"
- Aide : "Comment...?" liste les how-tos
- Settings rapides : "Toggle dark mode"

## Vue détail d'une entry (panneau slide-in droit)

```
+---------------------------------------+
| ← Retour          ⤓ Lancer  ⋯ Plus    |
+---------------------------------------+
| [Poster ou icône grande]              |
|                                       |
| Naruto Shippuden Complete VOSTFR 1080p|
| ────                                  |
| Catégorie : Séries                    |
| Qualité   : 1080p                     |
| Langue    : VOSTFR                    |
| Taille    : 60 GB                     |
| Seeders   : 234   Leechers : 12       |
| Ajouté    : 2026-04-12 (il y a 2j)    |
|                                       |
| Provenance                            |
|   Source : anime-fr (GitHub)          |
|   Par    : npub1abc...      ✓ vérifié |
|                                       |
| Tags                                  |
|   [1080p] [vostfr] [complete]         |
|                                       |
| Description                           |
|   Lorem ipsum...                      |
|                                       |
| Magnet                                |
|   magnet:?xt=urn:btih:...      [Copy] |
|                                       |
| [Lancer dans qBittorrent]             |
+---------------------------------------+
```

Actions du menu `⋯` :
- Copier le magnet
- Voir tous les torrents de ce contributeur
- Voir tous les torrents de cette commu
- Bannir ce contributeur (local ou modo)
- Signaler cette entry à la modo

## Vue Settings

Sections (sidebar à gauche du modal Settings) :

1. **Identité** — npub, génération QR code, export backup, import
2. **Sources** — liste des sources, ajouter, retirer, configurer sync policy
3. **Pools** — gérer les pools définis
4. **Apparence** — theme (dark / light / auto), font, accent color, density
5. **Client torrent** — qBittorrent / Transmission / Deluge, chemin/URL, intégration WebUI
6. **Réseau** — proxy, bandwidth, etc.
7. **Modération personnelle** — blacklist locale de pubkeys
8. **À propos** — version, logs, license

## Onboarding (premier lancement)

Wizard 4 étapes :

1. **Bienvenue** — explique le concept en 3 phrases.
2. **Génère ton identité** — création de la paire ed25519, propose backup immédiat.
3. **Ajoute ta première source** — propose des suggestions (annuaire optionnel) ou import manuel.
4. **Choisis ton client torrent** — détecte qBittorrent / Transmission / Deluge installés.

## Tokens design

```ts
// Couleurs (dark mode par défaut)
--bg-base:        #0a0a0b
--bg-elevated:    #131316
--bg-overlay:     #1c1c20
--border:         #26262a
--text-primary:   #f5f5f7
--text-secondary: #a1a1aa
--text-muted:     #71717a
--accent:         #6366f1     // indigo-500, modifiable

// Typo
--font-sans: 'Inter', system-ui, sans-serif
--font-mono: 'JetBrains Mono', monospace

// Espacement (Tailwind par défaut, étendu)
--radius-sm: 6px
--radius-md: 10px
--radius-lg: 14px
```

## Animations

- View Transitions API pour les changements de view (sidebar → detail panel).
- `motion-safe:transition-all duration-200 ease-out` sur tout ce qui bouge.
- `prefers-reduced-motion` respecté (désactive les animations longues).
- Skeletons de loading (shimmer subtil) pendant les fetch.

## Accessibilité

- Tous les contrôles accessibles au clavier (Tab order propre).
- ARIA labels sur tous les boutons icon-only.
- Contraste WCAG AA minimum sur tous les textes.
- Focus visible appuyé (ring 2px accent).
- Support lecteurs d'écran via les composants shadcn-svelte (déjà accessibles par défaut).

## i18n

- Préparer dès le départ avec un système type `svelte-i18n` ou `paraglide-js`.
- Locales prioritaires : `fr` (par défaut), `en`.
- Clés organisées par feature (`sidebar.categories.films`, `search.scope.all`, etc.).

## Hors-scope phase 3

- Mobile (Tauri Mobile possible plus tard).
- Themes communautaires (custom CSS user-fournies).
- Plugins UI (extensions tierces).
