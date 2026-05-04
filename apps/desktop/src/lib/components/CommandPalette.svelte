<script lang="ts">
  import {
    Search,
    RefreshCw,
    Plus,
    Settings,
    Filter,
    Layers,
    Upload,
    Download,
    Folder,
    Globe,
    GitBranch,
    Cloud,
    Server,
    Radio,
    Sparkles,
  } from "lucide-svelte";
  import { app } from "$lib/stores.svelte";

  // Lucide icons are still emitted as legacy Svelte 4 components, so we type
  // them loosely until lucide-svelte ships runes-mode bindings.
  type IconComp = unknown;
  type Action = {
    id: string;
    label: string;
    hint?: string;
    Icon: IconComp;
    run: () => void;
    keywords?: string[];
  };

  let {
    onSyncAll,
    onAddSource,
    onCreatePool,
    onPublish,
    onExportIdentity,
  }: {
    onSyncAll: () => void;
    onAddSource: () => void;
    onCreatePool: () => void;
    onPublish: () => void;
    onExportIdentity: () => void;
  } = $props();

  let inputEl = $state<HTMLInputElement | undefined>();
  let query = $state("");
  let activeIdx = $state(0);

  const sourceIcons = {
    LocalFolder: Folder,
    HttpUrl: Globe,
    GitRepo: GitBranch,
    GoogleDrive: Cloud,
    Dropbox: Cloud,
    OneDrive: Cloud,
    Server,
    Nostr: Radio,
    Ipfs: Radio,
  } as const;

  function close(): void {
    app.paletteOpen = false;
    query = "";
    activeIdx = 0;
  }

  function gotoBrowse() {
    app.view = "browse";
  }

  // Static actions
  let staticActions: Action[] = $derived([
    {
      id: "sync-all",
      label: "Synchroniser toutes les sources",
      hint: "récupère les nouveautés",
      Icon: RefreshCw,
      run: () => {
        close();
        onSyncAll();
      },
      keywords: ["sync", "refresh", "pull"],
    },
    {
      id: "add-source",
      label: "Ajouter une source",
      hint: "local / http / git",
      Icon: Plus,
      run: () => {
        close();
        onAddSource();
      },
      keywords: ["add", "new"],
    },
    {
      id: "create-pool",
      label: "Créer un pool",
      hint: "agrège plusieurs commus",
      Icon: Layers,
      run: () => {
        close();
        onCreatePool();
      },
    },
    {
      id: "publish",
      label: "Publier un magnet",
      hint: "signe et envoie",
      Icon: Upload,
      run: () => {
        close();
        onPublish();
      },
    },
    {
      id: "filters",
      label: app.filtersOpen ? "Masquer les filtres" : "Afficher les filtres",
      Icon: Filter,
      run: () => {
        close();
        app.filtersOpen = !app.filtersOpen;
      },
      keywords: ["filter"],
    },
    {
      id: "settings",
      label: "Ouvrir les paramètres",
      Icon: Settings,
      run: () => {
        close();
        app.view = "settings";
      },
    },
    {
      id: "export-identity",
      label: "Exporter l'identité",
      hint: "backup chiffré",
      Icon: Download,
      run: () => {
        close();
        onExportIdentity();
      },
      keywords: ["backup", "export"],
    },
  ]);

  // Source / pool nav actions
  let dynamicActions: Action[] = $derived(
    [
      ...app.sources.map<Action>((s) => ({
        id: `src-${s.id}`,
        label: s.display_name,
        hint: `commu · ${s.kind}`,
        Icon: (sourceIcons[s.kind] as IconComp) ?? Folder,
        run: () => {
          gotoBrowse();
          app.selectedSourceId = s.id;
          app.selectedPoolId = null;
          app.selectedCategory = null;
          close();
        },
      })),
      ...app.pools.map<Action>((p) => ({
        id: `pool-${p.id}`,
        label: p.name,
        hint: "pool",
        Icon: Layers as IconComp,
        run: () => {
          gotoBrowse();
          app.selectedPoolId = p.id;
          app.selectedSourceId = null;
          app.selectedCategory = null;
          close();
        },
      })),
    ],
  );

  // Top entries (filter by query when typed)
  let entryActions: Action[] = $derived(
    app.results.slice(0, 8).map<Action>((r) => ({
      id: `entry-${r.id}`,
      label: r.title,
      hint: r.id.slice(0, 8),
      Icon: Sparkles as IconComp,
      run: () => {
        gotoBrowse();
        app.selectedEntryId = r.id;
        close();
      },
      keywords: r.tags,
    })),
  );

  let actions: Action[] = $derived([...staticActions, ...dynamicActions, ...entryActions]);

  let filtered: Action[] = $derived.by(() => {
    if (!query.trim()) return actions;
    const q = query.toLowerCase();
    return actions.filter((a) => {
      if (a.label.toLowerCase().includes(q)) return true;
      if (a.hint?.toLowerCase().includes(q)) return true;
      if (a.keywords?.some((k) => k.toLowerCase().includes(q))) return true;
      return false;
    });
  });

  $effect(() => {
    void query;
    if (activeIdx >= filtered.length) activeIdx = 0;
  });

  $effect(() => {
    if (app.paletteOpen) {
      // focus the input on open
      setTimeout(() => inputEl?.focus(), 0);
    }
  });

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      close();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      activeIdx = Math.min(filtered.length - 1, activeIdx + 1);
      scrollActiveIntoView();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      activeIdx = Math.max(0, activeIdx - 1);
      scrollActiveIntoView();
    } else if (e.key === "Enter") {
      e.preventDefault();
      filtered[activeIdx]?.run();
    }
  }

  function scrollActiveIntoView() {
    queueMicrotask(() => {
      const el = document.querySelector<HTMLElement>(`[data-cmd-idx="${activeIdx}"]`);
      el?.scrollIntoView({ block: "nearest" });
    });
  }
</script>

{#if app.paletteOpen}
  <div
    class="fixed inset-0 z-50 flex items-start justify-center bg-black/50 p-4 pt-[12vh] backdrop-blur"
    onclick={close}
    onkeydown={onKey}
    role="presentation"
  >
    <div
      class="bg-elevated border-border w-full max-w-xl overflow-hidden rounded-xl border shadow-2xl"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="border-border flex items-center gap-2 border-b px-4 py-3">
        <Search size={16} class="text-muted" />
        <input
          bind:this={inputEl}
          bind:value={query}
          type="text"
          placeholder="Tape une commande, le nom d'une source, ou un titre…"
          class="text-primary placeholder:text-muted flex-1 bg-transparent text-sm outline-none"
        />
        <kbd class="text-muted text-xs">ESC</kbd>
      </div>

      <div class="scrollable max-h-[55vh] overflow-y-auto p-1.5">
        {#if filtered.length === 0}
          <div class="text-muted px-3 py-8 text-center text-sm">Aucune correspondance.</div>
        {:else}
          {#each filtered as action, i (action.id)}
            {@const Icon = action.Icon as ConstructorOfATypedSvelteComponent}
            <button
              type="button"
              data-cmd-idx={i}
              class="palette-row"
              class:active={i === activeIdx}
              onmouseenter={() => (activeIdx = i)}
              onclick={action.run}
            >
              <Icon size={15} />
              <span class="text-primary flex-1 truncate text-sm">{action.label}</span>
              {#if action.hint}
                <span class="text-muted text-xs">{action.hint}</span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      <div
        class="bg-base/40 text-muted border-border flex items-center justify-between border-t px-4 py-2 text-[11px]"
      >
        <span>{filtered.length} action(s)</span>
        <span>↑↓ naviguer · ↵ exécuter · esc fermer</span>
      </div>
    </div>
  </div>
{/if}
