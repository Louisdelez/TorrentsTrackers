<script lang="ts">
  import { X, Download, Upload, AlertTriangle, Loader2, Eye, EyeOff } from "lucide-svelte";
  import { ipc } from "$lib/ipc";
  import { app, showToast } from "$lib/stores.svelte";

  let {
    mode,
    onClose,
  }: {
    mode: "export" | "import";
    onClose: () => void;
  } = $props();

  let path = $state("");
  let passphrase = $state("");
  let force = $state(false);
  let showPass = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (mode === "export") {
      path = `${pickHomeDir()}/torrents-trackers-identity-${Date.now()}.tt-id`;
    }
  });

  function pickHomeDir(): string {
    return app.stats?.data_dir.replace(/\/torrents-trackers$/, "") ?? "/tmp";
  }

  async function submit() {
    if (!path.trim()) {
      error = "Chemin requis.";
      return;
    }
    if (!passphrase) {
      error = "Passphrase requise.";
      return;
    }
    busy = true;
    error = null;
    try {
      if (mode === "export") {
        const n = await ipc.identityExport(path.trim(), passphrase);
        showToast(`Identité exportée (${n} octets) — garde le fichier en sûreté.`);
      } else {
        const id = await ipc.identityImport(path.trim(), passphrase, force);
        app.identity = id;
        showToast("Identité importée.");
      }
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  let title = $derived(mode === "export" ? "Sauvegarder l'identité" : "Restaurer une identité");
  let Icon = $derived(mode === "export" ? Download : Upload);
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
        <Icon size={16} class="text-accent" />
        {title}
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

    {#if mode === "export"}
      <p class="text-secondary mb-4 text-xs">
        Exporte ta clé chiffrée AES-256-GCM (passphrase via scrypt). Stocke le fichier
        ailleurs (clé USB, password manager). Sans backup et en cas de perte de la machine,
        l'identité est irrécupérable.
      </p>
    {:else}
      <p class="text-secondary mb-4 text-xs">
        Restaure une identité depuis un backup chiffré. Si une identité existe déjà ici,
        coche "écraser" pour la remplacer (la précédente sera perdue si tu ne l'as pas
        également exportée).
      </p>
    {/if}

    <label class="block">
      <span class="text-secondary mb-1.5 block text-xs font-medium">Chemin du fichier</span>
      <input
        type="text"
        bind:value={path}
        placeholder={mode === "import" ? "/path/to/backup.tt-id" : "/où/sauvegarder.tt-id"}
        class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 font-mono text-sm outline-none focus:ring-2"
      />
    </label>

    <label class="mt-3 block">
      <span class="text-secondary mb-1.5 block text-xs font-medium">Passphrase</span>
      <div
        class="bg-base border-border focus-within:ring-accent flex items-center gap-2 rounded-lg border px-3 focus-within:ring-2"
      >
        {#if showPass}
          <input
            type="text"
            bind:value={passphrase}
            class="text-primary h-9 flex-1 bg-transparent text-sm outline-none"
          />
        {:else}
          <input
            type="password"
            bind:value={passphrase}
            class="text-primary h-9 flex-1 bg-transparent text-sm outline-none"
          />
        {/if}
        <button
          type="button"
          class="text-muted hover:text-primary"
          onclick={() => (showPass = !showPass)}
          aria-label="Afficher / masquer"
        >
          {#if showPass}<EyeOff size={14} />{:else}<Eye size={14} />{/if}
        </button>
      </div>
    </label>

    {#if mode === "import" && app.identity}
      <label class="mt-3 flex cursor-pointer items-start gap-2">
        <input type="checkbox" bind:checked={force} class="accent-accent mt-1" />
        <span class="text-secondary text-xs">
          Écraser l'identité existante <span class="text-warning">(irréversible)</span>
        </span>
      </label>
    {/if}

    {#if error}
      <p class="text-danger mt-3 inline-flex items-center gap-1.5 text-xs">
        <AlertTriangle size={12} /> {error}
      </p>
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
          <Icon size={14} />
        {/if}
        {mode === "export" ? "Exporter" : "Importer"}
      </button>
    </div>
  </div>
</div>
