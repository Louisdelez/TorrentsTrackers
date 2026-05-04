<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Header from "$lib/components/Header.svelte";
  import Browse from "$lib/components/Browse.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import Onboarding from "$lib/components/Onboarding.svelte";
  import AddSourceDialog from "$lib/components/AddSourceDialog.svelte";
  import CommandPalette from "$lib/components/CommandPalette.svelte";
  import FilterPanel from "$lib/components/FilterPanel.svelte";
  import CreatePoolDialog from "$lib/components/CreatePoolDialog.svelte";
  import PublishDialog from "$lib/components/PublishDialog.svelte";
  import IdentityBackupDialog from "$lib/components/IdentityBackupDialog.svelte";
  import { ipc } from "$lib/ipc";
  import { app, showToast } from "$lib/stores.svelte";
  import type { SearchQueryDto } from "$lib/types";

  let booted = $state(false);
  let dialogs = $state({
    addSource: false,
    createPool: false,
    publish: false,
    identity: null as null | "export" | "import",
  });

  async function refreshAll() {
    [app.sources, app.pools, app.stats, app.identity] = await Promise.all([
      ipc.listSources(),
      ipc.listPools(),
      ipc.stats(),
      ipc.identityShow(),
    ]);
  }

  async function search() {
    app.searching = true;
    try {
      const wireQuery = {
        text: app.searchText.trim() || null,
        scope: app.selectedSourceId
          ? { kind: "source", id: app.selectedSourceId }
          : app.selectedPoolId
            ? { kind: "pool", id: app.selectedPoolId }
            : { kind: "all" },
        categories: app.selectedCategory ? [app.selectedCategory] : null,
        qualities: app.filters.qualities.length ? app.filters.qualities : null,
        languages: app.filters.languages.length ? app.filters.languages : null,
        size_min: app.filters.sizeMin,
        size_max: app.filters.sizeMax,
        seeders_min: app.filters.seedersMin,
        limit: 200,
      };
      app.results = await ipc.search(wireQuery as never as SearchQueryDto);
      // Source-id filter is applied client-side because the SQL scope only
      // supports a single source/pool. Phase 4 may push it down.
      if (app.filters.sourceIds.length > 0) {
        const allowed = new Set(app.filters.sourceIds);
        app.results = app.results.filter((r) =>
          r.provenance.some((p) => allowed.has(p)),
        );
      }
    } catch (e) {
      showToast(`Erreur: ${e}`);
      app.results = [];
    } finally {
      app.searching = false;
    }
  }

  async function syncAll() {
    app.syncing = true;
    try {
      const fetched = await ipc.syncAllSources();
      app.sources = await ipc.listSources();
      app.stats = await ipc.stats();
      showToast(`Synchronisation : ${fetched} entries`);
      await search();
    } catch (e) {
      showToast(`Sync échouée: ${e}`);
    } finally {
      app.syncing = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    const meta = e.metaKey || e.ctrlKey;
    if (meta && e.key.toLowerCase() === "k") {
      e.preventDefault();
      app.paletteOpen = !app.paletteOpen;
    } else if (meta && e.key.toLowerCase() === "f") {
      e.preventDefault();
      app.filtersOpen = !app.filtersOpen;
    } else if (meta && e.key.toLowerCase() === "n") {
      e.preventDefault();
      dialogs.publish = true;
    }
  }

  onMount(async () => {
    window.addEventListener("keydown", onKeydown);
    await refreshAll();
    await search();
    booted = true;
  });

  onDestroy(() => {
    window.removeEventListener("keydown", onKeydown);
  });

  $effect(() => {
    void app.selectedCategory;
    void app.selectedSourceId;
    void app.selectedPoolId;
    void app.filters.qualities;
    void app.filters.languages;
    void app.filters.sizeMin;
    void app.filters.sizeMax;
    void app.filters.seedersMin;
    void app.filters.sourceIds;
    if (booted) void search();
  });
</script>

{#if !booted}
  <div class="bg-base flex h-screen w-screen items-center justify-center">
    <div class="text-muted text-sm">Chargement…</div>
  </div>
{:else if !app.identity}
  <Onboarding />
{:else}
  <div class="flex h-screen w-screen">
    <Sidebar onAddSource={() => (dialogs.addSource = true)} />
    <div class="flex min-w-0 flex-1 flex-col">
      <Header onSearch={search} onSyncAll={syncAll} />
      {#if app.view === "settings"}
        <Settings
          onExportIdentity={() => (dialogs.identity = "export")}
          onImportIdentity={() => (dialogs.identity = "import")}
          onCreatePool={() => (dialogs.createPool = true)}
          onPublish={() => (dialogs.publish = true)}
        />
      {:else}
        <div class="flex flex-1 overflow-hidden">
          <Browse />
          <FilterPanel onChange={search} />
        </div>
      {/if}
    </div>
  </div>

  {#if dialogs.addSource}
    <AddSourceDialog onClose={() => (dialogs.addSource = false)} />
  {/if}
  {#if dialogs.createPool}
    <CreatePoolDialog onClose={() => (dialogs.createPool = false)} />
  {/if}
  {#if dialogs.publish}
    <PublishDialog
      onClose={async () => {
        dialogs.publish = false;
        await search();
      }}
    />
  {/if}
  {#if dialogs.identity !== null}
    <IdentityBackupDialog
      mode={dialogs.identity}
      onClose={() => (dialogs.identity = null)}
    />
  {/if}

  <CommandPalette
    onSyncAll={syncAll}
    onAddSource={() => (dialogs.addSource = true)}
    onCreatePool={() => (dialogs.createPool = true)}
    onPublish={() => (dialogs.publish = true)}
    onExportIdentity={() => (dialogs.identity = "export")}
  />

  {#if app.toast}
    <div
      class="bg-overlay text-primary border-border fixed right-6 bottom-6 z-40 max-w-sm rounded-lg border px-4 py-2.5 text-sm shadow-lg"
    >
      {app.toast}
    </div>
  {/if}
{/if}
