<script lang="ts">
  import {
    Clapperboard,
    Tv,
    Gamepad2,
    Music,
    BookOpen,
    AppWindow,
    Package,
    Plus,
    Folder,
    Globe,
    GitBranch,
    Cloud,
    Server,
    Radio,
    Layers,
  } from "lucide-svelte";
  import type { Category, SourceKind } from "$lib/types";
  import { CATEGORIES } from "$lib/types";
  import { app } from "$lib/stores.svelte";

  const categoryIcons: Record<Category, typeof Clapperboard> = {
    Films: Clapperboard,
    Series: Tv,
    Games: Gamepad2,
    Music: Music,
    Books: BookOpen,
    Software: AppWindow,
    Other: Package,
  };

  const sourceIcons: Record<SourceKind, typeof Folder> = {
    LocalFolder: Folder,
    HttpUrl: Globe,
    GitRepo: GitBranch,
    GoogleDrive: Cloud,
    Dropbox: Cloud,
    OneDrive: Cloud,
    Server: Server,
    Nostr: Radio,
    Ipfs: Radio,
  };

  let { onAddSource }: { onAddSource: () => void } = $props();

  function selectCategory(c: Category | null) {
    app.selectedCategory = c;
    app.selectedSourceId = null;
    app.selectedPoolId = null;
  }

  function selectSource(id: string) {
    app.selectedSourceId = app.selectedSourceId === id ? null : id;
    app.selectedPoolId = null;
  }

  function selectPool(id: string) {
    app.selectedPoolId = app.selectedPoolId === id ? null : id;
    app.selectedSourceId = null;
  }
</script>

<aside class="bg-elevated border-border flex h-full w-56 shrink-0 flex-col border-r">
  <!-- Categories -->
  <section class="p-3">
    <h3 class="text-muted mb-2 px-2 text-[10px] font-semibold tracking-wider uppercase">
      Catégories
    </h3>
    <ul class="space-y-0.5">
      <li>
        <button
          type="button"
          class="cat-btn"
          class:active={app.selectedCategory === null &&
            !app.selectedSourceId &&
            !app.selectedPoolId}
          onclick={() => selectCategory(null)}
        >
          <Layers size={16} />
          <span>Tout</span>
        </button>
      </li>
      {#each CATEGORIES as cat}
        {@const Icon = categoryIcons[cat.id]}
        <li>
          <button
            type="button"
            class="cat-btn"
            class:active={app.selectedCategory === cat.id}
            onclick={() => selectCategory(cat.id)}
          >
            <Icon size={16} />
            <span>{cat.label}</span>
          </button>
        </li>
      {/each}
    </ul>
  </section>

  <hr class="border-border mx-3" />

  <!-- Communities -->
  <section class="p-3">
    <h3 class="text-muted mb-2 px-2 text-[10px] font-semibold tracking-wider uppercase">
      Communautés
    </h3>
    {#if app.sources.length === 0}
      <p class="text-muted px-2 text-xs">Aucune source.</p>
    {:else}
      <ul class="space-y-0.5">
        {#each app.sources as s}
          {@const Icon = sourceIcons[s.kind]}
          <li>
            <button
              type="button"
              class="cat-btn"
              class:active={app.selectedSourceId === s.id}
              title={s.endpoint}
              onclick={() => selectSource(s.id)}
            >
              <Icon size={16} class="shrink-0" />
              <span class="truncate">{s.display_name}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
    <button type="button" class="cat-btn text-muted mt-1.5" onclick={onAddSource}>
      <Plus size={16} />
      <span>Ajouter</span>
    </button>
  </section>

  <hr class="border-border mx-3" />

  <!-- Pools -->
  <section class="flex-1 p-3">
    <h3 class="text-muted mb-2 px-2 text-[10px] font-semibold tracking-wider uppercase">
      Pools
    </h3>
    {#if app.pools.length === 0}
      <p class="text-muted px-2 text-xs">Pas de pool.</p>
    {:else}
      <ul class="space-y-0.5">
        {#each app.pools as p}
          <li>
            <button
              type="button"
              class="cat-btn"
              class:active={app.selectedPoolId === p.id}
              onclick={() => selectPool(p.id)}
            >
              <Layers size={16} />
              <span class="truncate">{p.name}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</aside>

