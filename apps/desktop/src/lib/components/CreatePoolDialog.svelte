<script lang="ts">
  import { X, Layers, Loader2 } from "lucide-svelte";
  import { ipc } from "$lib/ipc";
  import { app, showToast } from "$lib/stores.svelte";

  let { onClose }: { onClose: () => void } = $props();

  let name = $state("");
  let selected = $state<string[]>([]);
  let busy = $state(false);
  let error = $state<string | null>(null);

  function toggle(id: string) {
    selected = selected.includes(id) ? selected.filter((s) => s !== id) : [...selected, id];
  }

  async function submit() {
    if (!name.trim()) {
      error = "Nom requis.";
      return;
    }
    if (selected.length === 0) {
      error = "Choisis au moins une source.";
      return;
    }
    busy = true;
    error = null;
    try {
      const dto = await ipc.createPool(name.trim(), selected);
      app.pools = [...app.pools, dto];
      showToast(`Pool '${dto.name}' créé`);
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
    class="bg-elevated border-border w-full max-w-md rounded-xl border p-6 shadow-2xl"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="mb-5 flex items-center justify-between">
      <h2 class="text-primary inline-flex items-center gap-2 text-base font-semibold">
        <Layers size={16} class="text-accent" />
        Nouveau pool
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

    <p class="text-secondary mb-4 text-xs">
      Un pool agrège plusieurs communautés. La recherche dans son scope dédupe les entries
      identiques entre commus et trace leur provenance.
    </p>

    <label class="block">
      <span class="text-secondary mb-1.5 block text-xs font-medium">Nom</span>
      <input
        type="text"
        bind:value={name}
        placeholder="ex: Mes Films VF"
        class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 text-sm outline-none focus:ring-2"
      />
    </label>

    <div class="mt-3">
      <span class="text-secondary mb-1.5 block text-xs font-medium">Communautés membres</span>
      {#if app.sources.length === 0}
        <p class="text-muted bg-base rounded p-3 text-xs">
          Aucune source. Ajoute-en une avant de créer un pool.
        </p>
      {:else}
        <div class="bg-base border-border max-h-56 overflow-y-auto rounded-lg border">
          {#each app.sources as s}
            <label
              class="hover:bg-overlay flex cursor-pointer items-center gap-2 px-3 py-2 text-sm"
            >
              <input
                type="checkbox"
                checked={selected.includes(s.id)}
                onchange={() => toggle(s.id)}
                class="accent-accent"
              />
              <span class="text-primary flex-1 truncate">{s.display_name}</span>
              <span class="text-muted text-[10px] uppercase">{s.kind}</span>
            </label>
          {/each}
        </div>
      {/if}
    </div>

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
        disabled={busy || app.sources.length === 0}
        onclick={submit}
      >
        {#if busy}
          <Loader2 size={14} class="animate-spin" />
        {/if}
        Créer
      </button>
    </div>
  </div>
</div>
