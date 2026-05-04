<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    Plus,
    Pause,
    Play,
    Trash2,
    Download as DLIcon,
    Loader2,
    X,
    CheckCircle2,
    AlertCircle,
  } from "lucide-svelte";
  import { ipc } from "$lib/ipc";
  import { app, showToast } from "$lib/stores.svelte";
  import type { DownloadInfo } from "$lib/types";

  let downloads = $state<DownloadInfo[]>([]);
  let dialogOpen = $state(false);
  let magnetInput = $state("");
  let busy = $state(false);
  let interval: ReturnType<typeof setInterval> | undefined;

  async function refresh() {
    try {
      downloads = await ipc.downloadList();
    } catch (e) {
      console.warn("download list failed:", e);
    }
  }

  onMount(() => {
    refresh();
    interval = setInterval(refresh, 1500);
  });

  onDestroy(() => {
    if (interval) clearInterval(interval);
  });

  async function add() {
    const url = magnetInput.trim();
    if (!url) return;
    busy = true;
    try {
      const id = await ipc.downloadAdd(url);
      showToast(`Téléchargement ajouté (#${id})`);
      magnetInput = "";
      dialogOpen = false;
      await refresh();
    } catch (e) {
      showToast(`Échec: ${e}`);
    } finally {
      busy = false;
    }
  }

  async function pause(d: DownloadInfo) {
    try {
      await ipc.downloadPause(d.id);
      await refresh();
    } catch (e) {
      showToast(`Pause échouée: ${e}`);
    }
  }
  async function resume(d: DownloadInfo) {
    try {
      await ipc.downloadUnpause(d.id);
      await refresh();
    } catch (e) {
      showToast(`Reprise échouée: ${e}`);
    }
  }
  async function remove(d: DownloadInfo) {
    if (!confirm(`Retirer "${d.title}" ? Les fichiers déjà sur disque sont conservés.`)) return;
    try {
      await ipc.downloadRemove(d.id);
      await refresh();
    } catch (e) {
      showToast(`Suppression échouée: ${e}`);
    }
  }

  function humanBytes(b: number): string {
    if (b <= 0) return "0 B";
    const u = ["B", "KB", "MB", "GB", "TB"];
    let v = b;
    let i = 0;
    while (v >= 1024 && i < u.length - 1) {
      v /= 1024;
      i++;
    }
    return i === 0 ? `${v} ${u[i]}` : `${v.toFixed(1)} ${u[i]}`;
  }
  function speed(b: number): string {
    return `${humanBytes(b)}/s`;
  }
  function pct(d: DownloadInfo): number {
    if (d.total_bytes === 0) return 0;
    return Math.min(100, Math.round((d.progress_bytes / d.total_bytes) * 100));
  }
</script>

<div class="bg-base scrollable flex-1 overflow-y-auto">
  <div class="mx-auto max-w-4xl p-6">
    <div class="mb-5 flex items-center justify-between">
      <div>
        <h1 class="text-primary text-xl font-semibold inline-flex items-center gap-2">
          <DLIcon size={18} class="text-accent" /> Téléchargements
        </h1>
        <p class="text-muted text-xs mt-1">
          Téléchargements gérés par librqbit en local — port d'écoute alloué par l'OS,
          pas de redirection requise.
        </p>
      </div>
      <button
        type="button"
        class="bg-accent hover:bg-accent-hover inline-flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-white transition"
        onclick={() => (dialogOpen = true)}
      >
        <Plus size={14} /> Ajouter un magnet
      </button>
    </div>

    {#if downloads.length === 0}
      <p class="text-muted bg-elevated border-border rounded-xl border p-8 text-center text-sm">
        Aucun téléchargement actif.
      </p>
    {:else}
      <ul class="space-y-2">
        {#each downloads as d (d.id)}
          <li class="bg-elevated border-border rounded-xl border p-4">
            <div class="flex items-start gap-3">
              <div class="mt-0.5 shrink-0">
                {#if d.state === "finished"}
                  <CheckCircle2 size={16} class="text-success" />
                {:else if d.state === "paused"}
                  <Pause size={16} class="text-muted" />
                {:else if d.state === "error"}
                  <AlertCircle size={16} class="text-danger" />
                {:else if d.state === "initializing"}
                  <Loader2 size={16} class="text-muted animate-spin" />
                {:else}
                  <DLIcon size={16} class="text-accent" />
                {/if}
              </div>
              <div class="min-w-0 flex-1">
                <div class="text-primary truncate text-sm font-medium">{d.title}</div>
                <div class="text-secondary mt-0.5 flex items-center gap-3 text-xs">
                  <span>
                    {humanBytes(d.progress_bytes)} / {d.total_bytes > 0 ? humanBytes(d.total_bytes) : "?"}
                  </span>
                  <span>·</span>
                  <span class="text-success">↓ {speed(d.down_bps)}</span>
                  <span>·</span>
                  <span class="text-warning">↑ {speed(d.up_bps)}</span>
                  {#if d.finished}
                    <span class="text-success">· terminé</span>
                  {/if}
                </div>
                <div class="bg-base mt-2 h-1.5 overflow-hidden rounded-full">
                  <div
                    class="h-full transition-all"
                    class:bg-success={d.finished}
                    class:bg-accent={!d.finished}
                    style="width: {pct(d)}%"
                  ></div>
                </div>
              </div>
              <div class="flex shrink-0 items-center gap-1">
                {#if d.state === "paused"}
                  <button
                    type="button"
                    class="text-muted hover:text-success p-1.5"
                    onclick={() => resume(d)}
                    aria-label="Reprendre"
                    title="Reprendre"
                  >
                    <Play size={14} />
                  </button>
                {:else if !d.finished && d.state !== "error"}
                  <button
                    type="button"
                    class="text-muted hover:text-warning p-1.5"
                    onclick={() => pause(d)}
                    aria-label="Pause"
                    title="Pause"
                  >
                    <Pause size={14} />
                  </button>
                {/if}
                <button
                  type="button"
                  class="text-muted hover:text-danger p-1.5"
                  onclick={() => remove(d)}
                  aria-label="Retirer"
                  title="Retirer"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

{#if dialogOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    onclick={() => (dialogOpen = false)}
    onkeydown={(e) => e.key === "Escape" && (dialogOpen = false)}
    role="presentation"
  >
    <div
      class="bg-elevated border-border w-full max-w-md rounded-xl border p-6 shadow-2xl"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-primary text-base font-semibold">Ajouter un téléchargement</h2>
        <button
          type="button"
          class="text-muted hover:text-primary"
          onclick={() => (dialogOpen = false)}
          aria-label="Fermer"
        >
          <X size={18} />
        </button>
      </div>
      <input
        type="text"
        bind:value={magnetInput}
        placeholder="magnet:?xt=urn:btih:..."
        class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 font-mono text-xs outline-none focus:ring-2"
      />
      <div class="mt-5 flex justify-end gap-2">
        <button
          type="button"
          class="text-secondary hover:text-primary px-4 py-2 text-sm"
          onclick={() => (dialogOpen = false)}
        >
          Annuler
        </button>
        <button
          type="button"
          class="bg-accent hover:bg-accent-hover inline-flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium text-white transition disabled:opacity-50"
          disabled={busy}
          onclick={add}
        >
          {#if busy}
            <Loader2 size={14} class="animate-spin" />
          {/if}
          Ajouter
        </button>
      </div>
    </div>
  </div>
{/if}
