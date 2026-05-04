<script lang="ts">
  import { X, Upload, Loader2, AlertTriangle } from "lucide-svelte";
  import { ipc } from "$lib/ipc";
  import { app, showToast } from "$lib/stores.svelte";
  import type { Category, Language, Quality } from "$lib/types";
  import { CATEGORIES } from "$lib/types";

  let { onClose }: { onClose: () => void } = $props();

  let magnet = $state("");
  let title = $state("");
  let category = $state<Category>("Films");
  let quality = $state<Quality | null>(null);
  let languages = $state<Language[]>([]);
  let tags = $state("");
  let sizeStr = $state("");
  let target = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);

  let writableSources = $derived(app.sources.filter((s) => s.kind === "LocalFolder"));

  $effect(() => {
    if (writableSources.length > 0 && !target) target = writableSources[0].id;
  });

  $effect(() => {
    if (magnet && !title) {
      const m = magnet.match(/[?&]dn=([^&]+)/);
      if (m) {
        try {
          title = decodeURIComponent(m[1].replace(/\+/g, " "));
        } catch {
          title = m[1];
        }
      }
    }
  });

  const QUALITIES: { id: Quality; label: string }[] = [
    { id: "P480", label: "480p" },
    { id: "P720", label: "720p" },
    { id: "P1080", label: "1080p" },
    { id: "P2160", label: "4K" },
  ];

  const LANGS: { id: Language; label: string }[] = [
    { id: "VOSTFR", label: "VOSTFR" },
    { id: "FR", label: "VF" },
    { id: "EN", label: "EN" },
    { id: "Multi", label: "Multi" },
  ];

  function toggleLang(l: Language) {
    languages = languages.includes(l) ? languages.filter((x) => x !== l) : [...languages, l];
  }

  async function submit() {
    if (!magnet.trim().startsWith("magnet:?")) {
      error = "Magnet invalide.";
      return;
    }
    if (!title.trim()) {
      error = "Titre requis.";
      return;
    }
    if (!target) {
      error = "Source de destination requise.";
      return;
    }
    busy = true;
    error = null;
    try {
      const sizeBytes = sizeStr.trim() ? Number(sizeStr) : null;
      const tagsArr = tags
        .split(",")
        .map((t) => t.trim())
        .filter(Boolean);
      const hit = await ipc.publish({
        magnet: magnet.trim(),
        targetSourceId: target,
        title: title.trim(),
        category,
        tags: tagsArr,
        quality,
        languages,
        sizeBytes,
      });
      showToast(`Publié : ${hit.title}`);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
  onclick={onClose}
  onkeydown={(e) => e.key === "Escape" && onClose()}
  role="presentation"
>
  <div
    class="bg-elevated border-border max-h-[90vh] w-full max-w-lg overflow-y-auto rounded-xl border p-6 shadow-2xl"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="mb-5 flex items-center justify-between">
      <h2 class="text-primary inline-flex items-center gap-2 text-base font-semibold">
        <Upload size={16} class="text-accent" />
        Publier un magnet
      </h2>
      <button
        type="button"
        class="text-muted hover:text-primary"
        onclick={onClose}
        aria-label="Fermer"
      >
        <X size={18} />
      </button>
    </div>

    {#if !app.identity}
      <div
        class="text-warning bg-base inline-flex items-center gap-2 rounded p-3 text-xs"
      >
        <AlertTriangle size={14} />
        Pas d'identité. Génère-en une via les paramètres avant de publier.
      </div>
    {:else if writableSources.length === 0}
      <div class="text-warning bg-base rounded p-3 text-xs">
        Aucune source en écriture. Ajoute une source <code>local</code> pour publier dedans.
      </div>
    {:else}
      <label class="block">
        <span class="text-secondary mb-1.5 block text-xs font-medium">Magnet URI</span>
        <input
          type="text"
          bind:value={magnet}
          placeholder="magnet:?xt=urn:btih:..."
          class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 font-mono text-xs outline-none focus:ring-2"
        />
      </label>

      <label class="mt-3 block">
        <span class="text-secondary mb-1.5 block text-xs font-medium">Titre</span>
        <input
          type="text"
          bind:value={title}
          placeholder="ex: Inception 1080p VOSTFR"
          class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 text-sm outline-none focus:ring-2"
        />
      </label>

      <div class="mt-3 grid grid-cols-2 gap-3">
        <label class="block">
          <span class="text-secondary mb-1.5 block text-xs font-medium">Catégorie</span>
          <select
            bind:value={category}
            class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 text-sm outline-none focus:ring-2"
          >
            {#each CATEGORIES as c}
              <option value={c.id}>{c.label}</option>
            {/each}
          </select>
        </label>
        <label class="block">
          <span class="text-secondary mb-1.5 block text-xs font-medium">Source destination</span>
          <select
            bind:value={target}
            class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 text-sm outline-none focus:ring-2"
          >
            {#each writableSources as s}
              <option value={s.id}>{s.display_name}</option>
            {/each}
          </select>
        </label>
      </div>

      <div class="mt-3">
        <span class="text-secondary mb-1.5 block text-xs font-medium">Qualité</span>
        <div class="flex flex-wrap gap-1.5">
          <button
            type="button"
            class="chip"
            class:active={quality === null}
            onclick={() => (quality = null)}
          >
            —
          </button>
          {#each QUALITIES as q}
            <button
              type="button"
              class="chip"
              class:active={quality === q.id}
              onclick={() => (quality = q.id)}
            >
              {q.label}
            </button>
          {/each}
        </div>
      </div>

      <div class="mt-3">
        <span class="text-secondary mb-1.5 block text-xs font-medium">Langue(s)</span>
        <div class="flex flex-wrap gap-1.5">
          {#each LANGS as l}
            <button
              type="button"
              class="chip"
              class:active={languages.includes(l.id)}
              onclick={() => toggleLang(l.id)}
            >
              {l.label}
            </button>
          {/each}
        </div>
      </div>

      <label class="mt-3 block">
        <span class="text-secondary mb-1.5 block text-xs font-medium">Tags (csv)</span>
        <input
          type="text"
          bind:value={tags}
          placeholder="1080p, vostfr, complete"
          class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 text-sm outline-none focus:ring-2"
        />
      </label>

      <label class="mt-3 block">
        <span class="text-secondary mb-1.5 block text-xs font-medium">Taille (octets, optionnel)</span>
        <input
          type="number"
          bind:value={sizeStr}
          placeholder="ex: 5368709120"
          class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 font-mono text-sm outline-none focus:ring-2"
        />
      </label>

      {#if error}
        <p class="text-danger mt-3 text-xs">{error}</p>
      {/if}

      <div class="mt-5 flex justify-end gap-2">
        <button
          type="button"
          class="text-secondary hover:text-primary px-4 py-2 text-sm"
          onclick={onClose}
        >
          Annuler
        </button>
        <button
          type="button"
          class="bg-accent hover:bg-accent-hover inline-flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium text-white transition disabled:opacity-50"
          disabled={busy}
          onclick={submit}
        >
          {#if busy}
            <Loader2 size={14} class="animate-spin" />
          {:else}
            <Upload size={14} />
          {/if}
          Signer & publier
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
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
