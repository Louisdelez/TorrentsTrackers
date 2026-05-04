<script lang="ts">
  import { X, RotateCcw } from "lucide-svelte";
  import { app, resetFilters } from "$lib/stores.svelte";
  import type { Language, Quality } from "$lib/types";

  let { onChange }: { onChange: () => void } = $props();

  const QUALITIES: { id: Quality; label: string }[] = [
    { id: "P480", label: "480p" },
    { id: "P720", label: "720p" },
    { id: "P1080", label: "1080p" },
    { id: "P2160", label: "4K" },
  ];

  const LANGUAGES: { id: Language; label: string }[] = [
    { id: "VOSTFR", label: "VOSTFR" },
    { id: "FR", label: "VF" },
    { id: "EN", label: "EN" },
    { id: "Multi", label: "Multi" },
  ];

  const SIZE_PRESETS: { label: string; min: number | null; max: number | null }[] = [
    { label: "tout", min: null, max: null },
    { label: "< 1 GB", min: null, max: 1_000_000_000 },
    { label: "1 – 10 GB", min: 1_000_000_000, max: 10_000_000_000 },
    { label: "10 – 50 GB", min: 10_000_000_000, max: 50_000_000_000 },
    { label: "> 50 GB", min: 50_000_000_000, max: null },
  ];

  function toggle<T>(arr: T[], v: T): T[] {
    return arr.includes(v) ? arr.filter((x) => x !== v) : [...arr, v];
  }

  function setQuality(q: Quality) {
    app.filters.qualities = toggle(app.filters.qualities, q);
    onChange();
  }
  function setLanguage(l: Language) {
    app.filters.languages = toggle(app.filters.languages, l);
    onChange();
  }
  function setSize(min: number | null, max: number | null) {
    app.filters.sizeMin = min;
    app.filters.sizeMax = max;
    onChange();
  }
  function setSeedersMin(v: number | null) {
    app.filters.seedersMin = v;
    onChange();
  }
  function toggleSource(id: string) {
    app.filters.sourceIds = toggle(app.filters.sourceIds, id);
    onChange();
  }

  function reset() {
    resetFilters();
    onChange();
  }

  let activeSizePreset = $derived(
    SIZE_PRESETS.findIndex(
      (p) => p.min === app.filters.sizeMin && p.max === app.filters.sizeMax,
    ),
  );
</script>

{#if app.filtersOpen}
  <aside
    class="bg-elevated border-border scrollable flex w-72 shrink-0 flex-col overflow-y-auto border-l"
  >
    <header class="border-border flex items-center justify-between border-b px-4 py-3">
      <h3 class="text-primary text-sm font-semibold">Filtres</h3>
      <div class="flex items-center gap-1">
        <button
          type="button"
          class="text-muted hover:text-primary p-1.5"
          title="Réinitialiser"
          onclick={reset}
        >
          <RotateCcw size={14} />
        </button>
        <button
          type="button"
          class="text-muted hover:text-primary p-1.5"
          onclick={() => (app.filtersOpen = false)}
          aria-label="Fermer"
        >
          <X size={14} />
        </button>
      </div>
    </header>

    <section class="p-4">
      <h4 class="filter-h">Qualité</h4>
      <div class="flex flex-wrap gap-1.5">
        {#each QUALITIES as q}
          <button
            type="button"
            class="chip"
            class:active={app.filters.qualities.includes(q.id)}
            onclick={() => setQuality(q.id)}
          >
            {q.label}
          </button>
        {/each}
      </div>
    </section>

    <section class="p-4 pt-0">
      <h4 class="filter-h">Langue</h4>
      <div class="flex flex-wrap gap-1.5">
        {#each LANGUAGES as l}
          <button
            type="button"
            class="chip"
            class:active={app.filters.languages.includes(l.id)}
            onclick={() => setLanguage(l.id)}
          >
            {l.label}
          </button>
        {/each}
      </div>
    </section>

    <section class="p-4 pt-0">
      <h4 class="filter-h">Taille</h4>
      <div class="flex flex-wrap gap-1.5">
        {#each SIZE_PRESETS as p, i}
          <button
            type="button"
            class="chip"
            class:active={i === activeSizePreset}
            onclick={() => setSize(p.min, p.max)}
          >
            {p.label}
          </button>
        {/each}
      </div>
    </section>

    <section class="p-4 pt-0">
      <h4 class="filter-h">Seeders min</h4>
      <div class="flex flex-wrap gap-1.5">
        {#each [null, 1, 10, 50, 200] as v}
          <button
            type="button"
            class="chip"
            class:active={(app.filters.seedersMin ?? null) === v}
            onclick={() => setSeedersMin(v)}
          >
            {v === null ? "tout" : `≥ ${v}`}
          </button>
        {/each}
      </div>
    </section>

    {#if app.sources.length > 0}
      <section class="p-4 pt-0">
        <h4 class="filter-h">Communautés</h4>
        <div class="flex flex-col gap-1">
          {#each app.sources as s}
            <label class="hover:bg-overlay flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5">
              <input
                type="checkbox"
                checked={app.filters.sourceIds.includes(s.id)}
                onchange={() => toggleSource(s.id)}
                class="accent-accent"
              />
              <span class="text-secondary truncate text-sm">{s.display_name}</span>
            </label>
          {/each}
        </div>
      </section>
    {/if}
  </aside>
{/if}

<style>
  .filter-h {
    color: var(--color-muted);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    margin-bottom: 0.625rem;
  }
  .chip {
    background: var(--color-base);
    border: 1px solid var(--color-border);
    color: var(--color-secondary);
    border-radius: 0.5rem;
    padding: 0.3rem 0.65rem;
    font-size: 0.75rem;
    transition: all 0.1s;
    cursor: pointer;
  }
  .chip:hover {
    background: var(--color-overlay);
    color: var(--color-primary);
  }
  .chip.active {
    background: color-mix(in srgb, var(--color-accent) 18%, transparent);
    border-color: var(--color-accent);
    color: var(--color-primary);
  }
</style>
