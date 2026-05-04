<script lang="ts">
  import { X, Loader2, Folder, Globe, GitBranch } from "lucide-svelte";
  import { ipc } from "$lib/ipc";
  import { app, showToast } from "$lib/stores.svelte";

  let { onClose }: { onClose: () => void } = $props();

  let kind = $state<"local" | "http" | "git">("local");
  let endpoint = $state("");
  let name = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);

  async function submit() {
    if (!endpoint.trim()) {
      error = "Endpoint requis.";
      return;
    }
    busy = true;
    error = null;
    try {
      const dto = await ipc.addSource(kind, endpoint.trim(), name.trim() || null);
      app.sources = [...app.sources, dto];
      showToast(`Source ajoutée : ${dto.display_name}`);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  const placeholders = {
    local: "/home/you/MesListes",
    http: "https://raw.githubusercontent.com/user/repo/main/entries.jsonl",
    git: "https://github.com/anime-fr/list-vf.git",
  };
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
      <h2 class="text-primary text-base font-semibold">Ajouter une source</h2>
      <button
        type="button"
        class="text-muted hover:text-primary"
        onclick={onClose}
        aria-label="Fermer"
      >
        <X size={18} />
      </button>
    </div>

    <div class="grid grid-cols-3 gap-2">
      {#each [{ id: "local", label: "Local", Icon: Folder }, { id: "http", label: "HTTP", Icon: Globe }, { id: "git", label: "Git", Icon: GitBranch }] as opt}
        {@const SelectedIcon = opt.Icon}
        <button
          type="button"
          class="kind-btn"
          class:active={kind === opt.id}
          onclick={() => (kind = opt.id as typeof kind)}
        >
          <SelectedIcon size={18} />
          <span class="text-xs font-medium">{opt.label}</span>
        </button>
      {/each}
    </div>

    <label class="mt-4 block">
      <span class="text-secondary mb-1.5 block text-xs font-medium">Endpoint</span>
      <input
        type="text"
        bind:value={endpoint}
        placeholder={placeholders[kind]}
        class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 font-mono text-sm outline-none focus:ring-2"
      />
    </label>

    <label class="mt-3 block">
      <span class="text-secondary mb-1.5 block text-xs font-medium">Nom (optionnel)</span>
      <input
        type="text"
        bind:value={name}
        placeholder="dérivé de l'endpoint"
        class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 text-sm outline-none focus:ring-2"
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
        {/if}
        Ajouter
      </button>
    </div>
  </div>
</div>

