<script lang="ts">
  import { Sparkles, Key, ShieldAlert, Loader2 } from "lucide-svelte";
  import { ipc } from "$lib/ipc";
  import { app, showToast } from "$lib/stores.svelte";

  let displayName = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);

  async function init() {
    busy = true;
    error = null;
    try {
      const id = await ipc.identityInit(displayName.trim() || null);
      app.identity = id;
      showToast("Identité créée — pense à exporter ta sauvegarde !");
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="bg-base flex h-full w-full items-center justify-center">
  <div class="bg-elevated border-border w-full max-w-md rounded-2xl border p-8 shadow-2xl">
    <div class="mb-6 flex items-center gap-3">
      <div
        class="from-accent flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br to-indigo-700"
      >
        <Sparkles size={24} class="text-white" />
      </div>
      <div>
        <h1 class="text-primary text-xl font-semibold">Bienvenue</h1>
        <p class="text-secondary text-sm">TorrentsTrackers · première utilisation</p>
      </div>
    </div>

    <div class="text-secondary space-y-3 text-sm">
      <p>
        Tu vas créer ta paire de clés <span class="text-primary font-medium">ed25519</span>.
        C'est ton identité partout — tu signes tes contributions, et les modos peuvent t'identifier
        sur n'importe quelle communauté.
      </p>
      <div class="bg-overlay text-secondary rounded-lg p-3 text-xs">
        <div class="flex items-start gap-2">
          <ShieldAlert size={14} class="text-warning mt-0.5 shrink-0" />
          <span>
            La clé privée vit en local. Sauvegarde-la (Settings → Identité) sinon perdre ton PC =
            perdre l'identité.
          </span>
        </div>
      </div>
    </div>

    <div class="mt-6 space-y-3">
      <label class="block">
        <span class="text-secondary mb-1.5 block text-xs font-medium">Pseudonyme (optionnel)</span>
        <input
          type="text"
          bind:value={displayName}
          placeholder="ex: louis"
          class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 text-sm outline-none focus:ring-2"
        />
      </label>

      {#if error}
        <p class="text-danger text-xs">{error}</p>
      {/if}

      <button
        type="button"
        class="bg-accent hover:bg-accent-hover flex h-10 w-full items-center justify-center gap-2 rounded-lg text-sm font-medium text-white transition disabled:opacity-50"
        disabled={busy}
        onclick={init}
      >
        {#if busy}
          <Loader2 size={16} class="animate-spin" />
        {:else}
          <Key size={16} />
        {/if}
        Générer mon identité
      </button>
    </div>
  </div>
</div>
