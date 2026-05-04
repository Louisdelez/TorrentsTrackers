<script lang="ts">
  import { Search, Settings, RefreshCw, User, MessageSquare, Download } from "lucide-svelte";
  import { app } from "$lib/stores.svelte";

  let { onSearch, onSyncAll }: { onSearch: () => void; onSyncAll: () => void } = $props();

  let searchInput = $state(app.searchText);
  let timer: ReturnType<typeof setTimeout> | undefined;

  function handleInput(): void {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      app.searchText = searchInput;
      onSearch();
    }, 200);
  }

  function shortNpub(n: string): string {
    if (!n) return "—";
    return n.slice(0, 8) + "…" + n.slice(-4);
  }
</script>

<header
  class="bg-elevated border-border flex h-14 shrink-0 items-center gap-3 border-b px-4"
>
  <div class="flex items-center gap-2">
    <div
      class="from-accent flex h-7 w-7 items-center justify-center rounded-md bg-gradient-to-br to-indigo-700 text-xs font-bold text-white"
    >
      tt
    </div>
    <span class="text-primary text-sm font-medium">TorrentsTrackers</span>
  </div>

  <div class="ml-2 flex max-w-2xl flex-1 items-center">
    <div
      class="bg-base focus-within:ring-accent border-border flex h-9 w-full items-center gap-2 rounded-lg border px-3 transition focus-within:ring-2"
    >
      <Search size={15} class="text-muted" />
      <input
        type="text"
        placeholder="Recherche dans la base locale..."
        class="text-primary placeholder:text-muted h-full flex-1 bg-transparent text-sm outline-none"
        bind:value={searchInput}
        oninput={handleInput}
      />
      <kbd class="text-muted hidden text-xs sm:inline">⌘K</kbd>
    </div>
  </div>

  <div class="flex items-center gap-1.5">
    <button
      type="button"
      class="icon-btn"
      class:active={app.view === "downloads"}
      title="Téléchargements"
      onclick={() =>
        (app.view = app.view === "downloads" ? "browse" : "downloads")}
    >
      <Download size={16} />
    </button>
    <button
      type="button"
      class="icon-btn"
      class:active={app.view === "chat"}
      title="Chat"
      onclick={() => (app.view = app.view === "chat" ? "browse" : "chat")}
    >
      <MessageSquare size={16} />
      {#if app.chatServers.length > 0}
        <span class="text-success text-[10px]">●</span>
      {/if}
    </button>
    <button type="button" class="icon-btn" title="Synchroniser tout" onclick={onSyncAll}>
      <RefreshCw size={16} class={app.syncing ? "animate-spin" : ""} />
    </button>
    <button
      type="button"
      class="icon-btn"
      title={app.identity ? app.identity.npub : "Identité"}
      onclick={() => (app.view = "settings")}
    >
      <User size={16} />
      <span class="text-secondary hidden text-xs sm:inline">
        {app.identity ? shortNpub(app.identity.npub) : "—"}
      </span>
    </button>
    <button
      type="button"
      class="icon-btn"
      title="Paramètres"
      onclick={() => (app.view = app.view === "settings" ? "browse" : "settings")}
    >
      <Settings size={16} />
    </button>
  </div>
</header>

