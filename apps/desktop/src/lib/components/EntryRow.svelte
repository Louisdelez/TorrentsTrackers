<script lang="ts">
  import { ExternalLink, Tag } from "lucide-svelte";
  import type { SearchHitDto } from "$lib/types";
  import { ipc } from "$lib/ipc";
  import { app, showToast } from "$lib/stores.svelte";

  let { hit }: { hit: SearchHitDto } = $props();

  function humanSize(b: number | null): string {
    if (b === null) return "?";
    const units = ["B", "KB", "MB", "GB", "TB"];
    let v = b;
    let i = 0;
    while (v >= 1024 && i < units.length - 1) {
      v /= 1024;
      i++;
    }
    return i === 0 ? `${v} ${units[i]}` : `${v.toFixed(1)} ${units[i]}`;
  }

  function relativeTime(iso: string): string {
    const d = new Date(iso);
    const ms = Date.now() - d.getTime();
    const s = Math.max(1, Math.floor(ms / 1000));
    if (s < 60) return "à l'instant";
    const m = Math.floor(s / 60);
    if (m < 60) return `il y a ${m}min`;
    const h = Math.floor(m / 60);
    if (h < 24) return `il y a ${h}h`;
    const days = Math.floor(h / 24);
    if (days < 30) return `il y a ${days}j`;
    return d.toLocaleDateString();
  }

  function qualityLabel(q: SearchHitDto["quality"]): string {
    if (!q) return "";
    if (typeof q === "string") {
      return { P480: "480p", P720: "720p", P1080: "1080p", P2160: "4K" }[q] ?? q;
    }
    return q.Other;
  }

  function languageLabel(l: SearchHitDto["languages"][number]): string {
    if (typeof l === "string") return l === "VOSTFR" ? "VOSTFR" : l;
    return l.Other;
  }

  function sourceName(id: string): string {
    return app.sources.find((s) => s.id === id)?.display_name ?? id.slice(0, 8);
  }

  async function open() {
    if (!hit.magnet) {
      showToast("Pas de magnet pour cette entrée");
      return;
    }
    try {
      await ipc.openMagnet(hit.magnet);
      showToast(`Lancé : ${hit.title}`);
    } catch (e) {
      showToast(`Échec ouverture: ${e}`);
    }
  }

  let isSelected = $derived(app.selectedEntryId === hit.id);
</script>

<div
  class="hover:bg-overlay group flex cursor-pointer items-start gap-3 px-4 py-2.5 transition-colors"
  class:bg-overlay={isSelected}
  onclick={() => (app.selectedEntryId = isSelected ? null : hit.id)}
  ondblclick={open}
  role="button"
  tabindex="0"
  onkeydown={(e) => {
    if (e.key === "Enter") open();
  }}
>
  <div class="min-w-0 flex-1">
    <div class="text-primary truncate text-sm font-medium">{hit.title}</div>
    <div class="text-secondary mt-0.5 flex flex-wrap items-center gap-x-3 gap-y-0.5 text-xs">
      <span>{humanSize(hit.size_bytes)}</span>
      {#if hit.quality}
        <span class="text-accent">{qualityLabel(hit.quality)}</span>
      {/if}
      {#if hit.languages.length > 0}
        <span>{hit.languages.map(languageLabel).join("/")}</span>
      {/if}
      {#if hit.seeders !== null}
        <span class="text-success">{hit.seeders} seeders</span>
      {/if}
      <span class="text-muted">{relativeTime(hit.added_at)}</span>
      {#each hit.tags.slice(0, 3) as t}
        <span
          class="bg-base text-muted inline-flex items-center gap-0.5 rounded px-1.5 py-0.5 text-[10px]"
        >
          <Tag size={9} />{t}
        </span>
      {/each}
    </div>
  </div>

  <div class="flex shrink-0 items-center gap-2">
    {#each hit.provenance as srcId}
      <span
        class="bg-base text-secondary inline-flex items-center rounded px-2 py-0.5 text-[10px] font-medium"
      >
        {sourceName(srcId)}
      </span>
    {/each}
    <button
      type="button"
      class="text-muted hover:text-accent rounded p-1 opacity-0 transition group-hover:opacity-100"
      onclick={(e) => {
        e.stopPropagation();
        open();
      }}
      title="Lancer dans qBittorrent"
      aria-label="Lancer le magnet"
    >
      <ExternalLink size={14} />
    </button>
  </div>
</div>
