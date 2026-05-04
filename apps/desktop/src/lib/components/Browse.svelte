<script lang="ts">
  import { Loader2, Inbox } from "lucide-svelte";
  import EntryRow from "./EntryRow.svelte";
  import { app } from "$lib/stores.svelte";
</script>

<div class="bg-base scrollable flex-1 overflow-y-auto">
  {#if app.searching}
    <div class="text-muted flex h-full items-center justify-center gap-2 text-sm">
      <Loader2 size={16} class="animate-spin" /> recherche…
    </div>
  {:else if app.results.length === 0}
    <div class="text-muted flex h-full flex-col items-center justify-center gap-3 text-sm">
      <Inbox size={32} class="opacity-40" />
      <p>Aucun résultat.</p>
      <p class="text-xs">
        Ajoute une source dans la sidebar et synchronise — ou ajuste ta recherche.
      </p>
    </div>
  {:else}
    <div class="border-border bg-elevated/50 sticky top-0 z-10 border-b px-4 py-2 text-xs">
      <span class="text-secondary">{app.results.length} résultat(s)</span>
    </div>
    <div class="divide-border divide-y">
      {#each app.results as hit (hit.id)}
        <EntryRow {hit} />
      {/each}
    </div>
  {/if}
</div>
