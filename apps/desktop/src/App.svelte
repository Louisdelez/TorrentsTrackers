<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Header from "$lib/components/Header.svelte";
  import Browse from "$lib/components/Browse.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import Onboarding from "$lib/components/Onboarding.svelte";
  import AddSourceDialog from "$lib/components/AddSourceDialog.svelte";
  import { ipc } from "$lib/ipc";
  import { app, showToast } from "$lib/stores.svelte";
  import type { SearchQueryDto } from "$lib/types";

  let booted = $state(false);
  let dialogOpen = $state(false);

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
      let scope: SearchQueryDto["scope"] = "all";
      if (app.selectedSourceId) scope = { source: app.selectedSourceId };
      else if (app.selectedPoolId) scope = { pool: app.selectedPoolId };
      const q: SearchQueryDto = {
        text: app.searchText.trim() || null,
        scope: scope as never,
        categories: app.selectedCategory ? [app.selectedCategory] : null,
        qualities: null,
        languages: null,
        size_min: null,
        size_max: null,
        seeders_min: null,
        limit: 200,
      };
      // Tauri expects "scope" as the discriminated DTO; build manually:
      const wireQuery = {
        ...q,
        scope:
          scope === "all"
            ? { kind: "all" }
            : "source" in (scope as object)
              ? { kind: "source", id: app.selectedSourceId! }
              : { kind: "pool", id: app.selectedPoolId! },
      };
      app.results = await ipc.search(wireQuery as never);
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

  onMount(async () => {
    await refreshAll();
    await search();
    booted = true;
  });

  $effect(() => {
    // Re-run search whenever the scope/category changes.
    void app.selectedCategory;
    void app.selectedSourceId;
    void app.selectedPoolId;
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
    <Sidebar onAddSource={() => (dialogOpen = true)} />
    <div class="flex min-w-0 flex-1 flex-col">
      <Header onSearch={search} onSyncAll={syncAll} />
      {#if app.view === "settings"}
        <Settings />
      {:else}
        <Browse />
      {/if}
    </div>
  </div>

  {#if dialogOpen}
    <AddSourceDialog onClose={() => (dialogOpen = false)} />
  {/if}

  {#if app.toast}
    <div
      class="bg-overlay text-primary border-border fixed right-6 bottom-6 z-40 max-w-sm rounded-lg border px-4 py-2.5 text-sm shadow-lg"
    >
      {app.toast}
    </div>
  {/if}
{/if}
