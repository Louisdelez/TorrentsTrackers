<script lang="ts">
  import {
    Copy,
    RefreshCw,
    Trash2,
    ArrowLeft,
    Download,
    Upload,
    Layers,
    Plus,
    AlertOctagon,
  } from "lucide-svelte";
  import { ipc } from "$lib/ipc";
  import { app, showToast } from "$lib/stores.svelte";

  let {
    onExportIdentity,
    onImportIdentity,
    onCreatePool,
    onPublish,
  }: {
    onExportIdentity: () => void;
    onImportIdentity: () => void;
    onCreatePool: () => void;
    onPublish: () => void;
  } = $props();

  async function refreshAll() {
    app.syncing = true;
    try {
      await ipc.syncAllSources();
      app.sources = await ipc.listSources();
      app.stats = await ipc.stats();
      showToast("Synchronisation terminée");
    } finally {
      app.syncing = false;
    }
  }

  async function forgetIdentity() {
    if (
      !confirm(
        "Effacer l'identité ? Tu ne pourras plus signer de contributions et tu ne pourras pas la récupérer si tu n'as pas de backup. Cette action est irréversible.",
      )
    )
      return;
    await ipc.identityForget();
    app.identity = null;
    showToast("Identité oubliée.");
  }

  async function removePool(id: string, name: string) {
    if (!confirm(`Supprimer le pool "${name}" ?`)) return;
    await ipc.removePool(id);
    app.pools = app.pools.filter((p) => p.id !== id);
    showToast(`Pool supprimé : ${name}`);
  }

  async function removeSource(id: string, name: string) {
    if (!confirm(`Supprimer la source "${name}" ?`)) return;
    await ipc.removeSource(id);
    app.sources = app.sources.filter((s) => s.id !== id);
    showToast(`Source supprimée : ${name}`);
  }

  function copy(s: string) {
    navigator.clipboard.writeText(s);
    showToast("Copié dans le presse-papiers");
  }

  function shortId(s: string): string {
    return s.slice(0, 8);
  }
</script>

<div class="bg-base scrollable flex-1 overflow-y-auto">
  <div class="mx-auto max-w-3xl p-8">
    <button
      type="button"
      class="text-muted hover:text-primary mb-4 inline-flex items-center gap-1.5 text-xs"
      onclick={() => (app.view = "browse")}
    >
      <ArrowLeft size={14} /> retour à la recherche
    </button>

    <h1 class="text-primary mb-6 text-xl font-semibold">Paramètres</h1>

    <!-- Identity -->
    <section class="bg-elevated border-border mb-6 rounded-xl border p-5">
      <h2 class="text-primary mb-3 text-sm font-medium">Identité</h2>
      {#if app.identity}
        <div class="space-y-2 text-sm">
          <div class="flex items-center gap-2">
            <span class="text-muted w-16 text-xs">npub</span>
            <code class="bg-base text-secondary flex-1 rounded px-2 py-1 font-mono text-xs">
              {app.identity.npub}
            </code>
            <button
              type="button"
              class="text-muted hover:text-primary"
              onclick={() => copy(app.identity!.npub)}
              aria-label="Copier"
            >
              <Copy size={14} />
            </button>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-muted w-16 text-xs">hex</span>
            <code class="bg-base text-secondary flex-1 truncate rounded px-2 py-1 font-mono text-xs">
              {app.identity.pubkey_hex}
            </code>
          </div>
          {#if app.identity.display_name}
            <div class="flex items-center gap-2">
              <span class="text-muted w-16 text-xs">nom</span>
              <span class="text-primary text-sm">{app.identity.display_name}</span>
            </div>
          {/if}
          <div class="flex items-center gap-2">
            <span class="text-muted w-16 text-xs">créé</span>
            <span class="text-secondary text-xs">
              {new Date(app.identity.created_at).toLocaleString()}
            </span>
          </div>
        </div>
        <div class="border-border mt-5 flex flex-wrap gap-2 border-t pt-4">
          <button type="button" class="btn-secondary" onclick={onExportIdentity}>
            <Download size={13} /> Sauvegarder
          </button>
          <button type="button" class="btn-secondary" onclick={onImportIdentity}>
            <Upload size={13} /> Importer un backup
          </button>
          <button type="button" class="btn-secondary danger" onclick={forgetIdentity}>
            <AlertOctagon size={13} /> Oublier
          </button>
        </div>
      {:else}
        <p class="text-muted text-sm">Pas d'identité.</p>
      {/if}
    </section>

    <!-- Quick actions -->
    <section class="bg-elevated border-border mb-6 rounded-xl border p-5">
      <h2 class="text-primary mb-3 text-sm font-medium">Actions</h2>
      <div class="flex flex-wrap gap-2">
        <button type="button" class="btn-secondary" onclick={onCreatePool}>
          <Layers size={13} /> Créer un pool
        </button>
        <button type="button" class="btn-secondary" onclick={onPublish}>
          <Plus size={13} /> Publier un magnet
        </button>
      </div>
    </section>

    <!-- Pools -->
    {#if app.pools.length > 0}
      <section class="bg-elevated border-border mb-6 rounded-xl border p-5">
        <h2 class="text-primary mb-3 text-sm font-medium">
          Pools ({app.pools.length})
        </h2>
        <ul class="divide-border divide-y">
          {#each app.pools as p}
            <li class="flex items-center gap-3 py-2.5">
              <code class="text-muted text-xs">{p.id.slice(0, 8)}</code>
              <span class="text-primary flex-1 text-sm">{p.name}</span>
              <span class="text-muted text-[10px]">{p.member_ids.length} src</span>
              <button
                type="button"
                class="text-muted hover:text-danger"
                onclick={() => removePool(p.id, p.name)}
                aria-label="Supprimer"
              >
                <Trash2 size={14} />
              </button>
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    <!-- Sources -->
    <section class="bg-elevated border-border mb-6 rounded-xl border p-5">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-primary text-sm font-medium">
          Sources ({app.sources.length})
        </h2>
        <button
          type="button"
          class="text-secondary hover:text-primary inline-flex items-center gap-1.5 text-xs disabled:opacity-50"
          disabled={app.syncing}
          onclick={refreshAll}
        >
          <RefreshCw size={13} class={app.syncing ? "animate-spin" : ""} />
          {app.syncing ? "Sync..." : "Tout synchroniser"}
        </button>
      </div>

      {#if app.sources.length === 0}
        <p class="text-muted text-sm">Aucune source. Ajoute-en une depuis la sidebar.</p>
      {:else}
        <ul class="divide-border divide-y">
          {#each app.sources as s}
            <li class="flex items-center gap-3 py-2.5">
              <code class="text-muted text-xs">{shortId(s.id)}</code>
              <div class="min-w-0 flex-1">
                <div class="text-primary text-sm">{s.display_name}</div>
                <div class="text-muted truncate font-mono text-xs">{s.endpoint}</div>
              </div>
              <span
                class="text-muted bg-base rounded px-1.5 py-0.5 text-[10px] uppercase"
              >
                {s.kind}
              </span>
              <button
                type="button"
                class="text-muted hover:text-danger"
                onclick={() => removeSource(s.id, s.display_name)}
                aria-label="Supprimer"
              >
                <Trash2 size={14} />
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <!-- Stats -->
    {#if app.stats}
      <section class="bg-elevated border-border rounded-xl border p-5">
        <h2 class="text-primary mb-3 text-sm font-medium">Stats locales</h2>
        <dl class="text-sm">
          <div class="flex gap-3 py-1">
            <dt class="text-muted w-32 text-xs">data dir</dt>
            <dd class="text-secondary truncate font-mono text-xs">{app.stats.data_dir}</dd>
          </div>
          <div class="flex gap-3 py-1">
            <dt class="text-muted w-32 text-xs">database</dt>
            <dd class="text-secondary truncate font-mono text-xs">{app.stats.db_path}</dd>
          </div>
          <div class="flex gap-3 py-1">
            <dt class="text-muted w-32 text-xs">entries</dt>
            <dd class="text-primary text-xs">{app.stats.entries}</dd>
          </div>
          <div class="flex gap-3 py-1">
            <dt class="text-muted w-32 text-xs">pools</dt>
            <dd class="text-primary text-xs">{app.stats.pools}</dd>
          </div>
        </dl>
      </section>
    {/if}
  </div>
</div>
