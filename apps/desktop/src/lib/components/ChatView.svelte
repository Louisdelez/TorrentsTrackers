<script lang="ts">
  import { onMount, tick } from "svelte";
  import { Send, Plug, X, MessageSquare, Loader2, CornerDownRight, Reply } from "lucide-svelte";
  import { ipc } from "$lib/ipc";
  import { app, showToast } from "$lib/stores.svelte";
  import type { ChatMessage } from "$lib/types";

  let connectUrl = $state("ws://127.0.0.1:6970/ws");
  let dialogOpen = $state(false);
  let connecting = $state(false);
  let input = $state("");
  let scrollEl = $state<HTMLDivElement | undefined>();
  let replyTo = $state<ChatMessage | null>(null);

  function previewOf(m: ChatMessage): string {
    return m.content.length > 60 ? m.content.slice(0, 57) + "…" : m.content;
  }

  function findParent(id: string): ChatMessage | undefined {
    return activeMessages.find((m) => m.id === id);
  }

  let activeMessages = $derived(
    app.chatActiveServerId ? (app.chatMessages[app.chatActiveServerId] ?? []) : [],
  );
  let activeServer = $derived(
    app.chatServers.find((s) => s.server_id === app.chatActiveServerId) ?? null,
  );

  $effect(() => {
    void activeMessages.length;
    tick().then(() => {
      if (scrollEl) scrollEl.scrollTop = scrollEl.scrollHeight;
    });
  });

  onMount(async () => {
    app.chatServers = await ipc.chatList();
    if (!app.chatActiveServerId && app.chatServers.length > 0) {
      app.chatActiveServerId = app.chatServers[0].server_id;
    }
  });

  async function connect() {
    connecting = true;
    try {
      const dto = await ipc.chatConnect(connectUrl.trim());
      app.chatServers = [...app.chatServers, dto];
      app.chatMessages[dto.server_id] = [];
      app.chatActiveServerId = dto.server_id;
      // subscribe + fetch recent history
      await ipc.chatHistory(dto.server_id, "main", 100);
      dialogOpen = false;
      showToast(`Connecté à ${dto.server_name}`);
    } catch (e) {
      showToast(`Connexion échouée: ${e}`);
    } finally {
      connecting = false;
    }
  }

  async function disconnect(serverId: string) {
    await ipc.chatDisconnect(serverId);
    app.chatServers = app.chatServers.filter((s) => s.server_id !== serverId);
    delete app.chatMessages[serverId];
    if (app.chatActiveServerId === serverId) {
      app.chatActiveServerId = app.chatServers[0]?.server_id ?? null;
    }
  }

  async function send() {
    const text = input.trim();
    if (!text || !app.chatActiveServerId) return;
    try {
      await ipc.chatSend(app.chatActiveServerId, "main", text, replyTo?.id ?? null);
      input = "";
      replyTo = null;
    } catch (e) {
      showToast(`Échec envoi: ${e}`);
    }
  }

  function startReply(m: ChatMessage) {
    replyTo = m;
  }

  function relTime(iso: string): string {
    const d = new Date(iso);
    const ms = Date.now() - d.getTime();
    const s = Math.floor(ms / 1000);
    if (s < 60) return `${s}s`;
    const m = Math.floor(s / 60);
    if (m < 60) return `${m}min`;
    const h = Math.floor(m / 60);
    if (h < 24) return `${h}h`;
    return d.toLocaleDateString();
  }

  function shortPk(pk: string): string {
    return pk.slice(0, 6);
  }

  function colorForPubkey(pk: string): string {
    let h = 0;
    for (let i = 0; i < pk.length; i++) h = (h * 31 + pk.charCodeAt(i)) >>> 0;
    return `hsl(${h % 360}, 70%, 65%)`;
  }
</script>

<div class="bg-base flex flex-1 overflow-hidden">
  <!-- Server list -->
  <div class="bg-elevated border-border flex w-56 shrink-0 flex-col border-r">
    <div class="border-border flex items-center justify-between border-b px-3 py-2">
      <span class="text-muted text-[10px] font-semibold tracking-wider uppercase">Serveurs</span>
      <button
        type="button"
        class="text-muted hover:text-primary inline-flex items-center gap-1 text-xs"
        onclick={() => (dialogOpen = true)}
      >
        <Plug size={12} /> connecter
      </button>
    </div>
    <div class="flex-1 overflow-y-auto p-2">
      {#if app.chatServers.length === 0}
        <p class="text-muted px-2 py-3 text-xs">
          Aucun serveur. Connecte-toi à <code>ws://host:port/ws</code>.
        </p>
      {:else}
        {#each app.chatServers as s}
          <div
            class="server-row group"
            class:active={app.chatActiveServerId === s.server_id}
            onclick={() => (app.chatActiveServerId = s.server_id)}
            onkeydown={(e) => {
              if (e.key === "Enter") app.chatActiveServerId = s.server_id;
            }}
            role="button"
            tabindex="0"
          >
            <MessageSquare size={14} class="shrink-0" />
            <span class="flex-1 truncate">{s.server_name}</span>
            <button
              type="button"
              class="text-muted hover:text-danger opacity-0 transition group-hover:opacity-100"
              onclick={(e) => {
                e.stopPropagation();
                disconnect(s.server_id);
              }}
              aria-label="Déconnecter"
            >
              <X size={12} />
            </button>
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Messages -->
  <div class="flex flex-1 flex-col">
    <div class="border-border bg-elevated/40 flex h-12 shrink-0 items-center border-b px-4">
      {#if activeServer}
        <div class="flex flex-col">
          <span class="text-primary text-sm font-medium">{activeServer.server_name}</span>
          <span class="text-muted font-mono text-[10px]">{activeServer.url}</span>
        </div>
      {:else}
        <span class="text-muted text-sm">Sélectionne ou connecte un serveur.</span>
      {/if}
    </div>

    <div bind:this={scrollEl} class="scrollable flex-1 overflow-y-auto px-4 py-3">
      {#if activeServer && activeMessages.length === 0}
        <p class="text-muted py-12 text-center text-sm">
          Aucun message. Démarre la conversation 👋
        </p>
      {/if}
      {#each activeMessages as m (m.id)}
        {@const parent = m.reply_to ? findParent(m.reply_to) : undefined}
        <div class="group flex gap-3 py-1.5" class:thread={!!parent}>
          <div class="text-secondary flex w-20 shrink-0 flex-col items-end">
            <span
              class="font-mono text-xs"
              style="color: {colorForPubkey(m.author_pubkey)}"
            >
              {shortPk(m.author_pubkey)}
            </span>
            <span class="text-muted text-[10px]">{relTime(m.sent_at)}</span>
          </div>
          <div class="flex-1">
            {#if parent}
              <div
                class="text-muted hover:text-secondary mb-0.5 inline-flex max-w-full cursor-pointer items-center gap-1 truncate text-[11px]"
                onclick={() => {
                  const el = document.querySelector(`[data-msg-id="${parent.id}"]`);
                  el?.scrollIntoView({ block: "center", behavior: "smooth" });
                }}
                onkeydown={(e) => {
                  if (e.key === "Enter") {
                    const el = document.querySelector(`[data-msg-id="${parent.id}"]`);
                    el?.scrollIntoView({ block: "center", behavior: "smooth" });
                  }
                }}
                role="button"
                tabindex="0"
                title={parent.content}
              >
                <CornerDownRight size={11} />
                <span style="color: {colorForPubkey(parent.author_pubkey)}">
                  {shortPk(parent.author_pubkey)}
                </span>
                <span class="truncate">{previewOf(parent)}</span>
              </div>
            {:else if m.reply_to}
              <div class="text-muted mb-0.5 inline-flex items-center gap-1 text-[11px]">
                <CornerDownRight size={11} />
                <span class="italic">message hors-scroll</span>
              </div>
            {/if}
            <div class="flex items-start gap-2" data-msg-id={m.id}>
              <p class="text-primary flex-1 text-sm break-words whitespace-pre-wrap">
                {m.content}
              </p>
              <button
                type="button"
                class="text-muted hover:text-accent shrink-0 opacity-0 transition group-hover:opacity-100"
                onclick={() => startReply(m)}
                aria-label="Répondre"
                title="Répondre"
              >
                <Reply size={13} />
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>

    {#if activeServer}
      {#if replyTo}
        <div
          class="border-border bg-base flex items-center gap-2 border-t px-4 py-2 text-xs"
        >
          <CornerDownRight size={12} class="text-muted shrink-0" />
          <span class="text-muted">Réponse à</span>
          <span
            class="font-mono"
            style="color: {colorForPubkey(replyTo.author_pubkey)}"
          >
            {shortPk(replyTo.author_pubkey)}
          </span>
          <span class="text-secondary flex-1 truncate">{previewOf(replyTo)}</span>
          <button
            type="button"
            class="text-muted hover:text-primary"
            onclick={() => (replyTo = null)}
            aria-label="Annuler la réponse"
          >
            <X size={12} />
          </button>
        </div>
      {/if}
      <form
        class="border-border bg-elevated/40 flex gap-2 border-t px-4 py-3"
        onsubmit={(e) => {
          e.preventDefault();
          send();
        }}
      >
        <input
          type="text"
          bind:value={input}
          placeholder="Message..."
          class="bg-base text-primary border-border focus:ring-accent flex-1 rounded-lg border px-3 py-2 text-sm outline-none focus:ring-2"
        />
        <button
          type="submit"
          class="bg-accent hover:bg-accent-hover inline-flex items-center gap-1.5 rounded-lg px-4 text-sm font-medium text-white transition disabled:opacity-50"
          disabled={!input.trim()}
        >
          <Send size={14} />
        </button>
      </form>
    {/if}
  </div>
</div>

{#if dialogOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    onclick={() => (dialogOpen = false)}
    onkeydown={(e) => e.key === "Escape" && (dialogOpen = false)}
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
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-primary text-base font-semibold">Connecter un serveur de chat</h2>
        <button
          type="button"
          class="text-muted hover:text-primary"
          onclick={() => (dialogOpen = false)}
          aria-label="Fermer"
        >
          <X size={18} />
        </button>
      </div>
      <p class="text-secondary mb-3 text-xs">
        URL du serveur communautaire (ws:// ou wss://). L'auth se fait avec ta clé locale.
      </p>
      <input
        type="text"
        bind:value={connectUrl}
        placeholder="ws://127.0.0.1:6970/ws"
        class="bg-base text-primary border-border focus:ring-accent w-full rounded-lg border px-3 py-2 font-mono text-sm outline-none focus:ring-2"
      />
      <div class="mt-5 flex justify-end gap-2">
        <button
          type="button"
          class="text-secondary hover:text-primary px-4 py-2 text-sm"
          onclick={() => (dialogOpen = false)}
        >
          Annuler
        </button>
        <button
          type="button"
          class="bg-accent hover:bg-accent-hover inline-flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium text-white transition disabled:opacity-50"
          disabled={connecting}
          onclick={connect}
        >
          {#if connecting}
            <Loader2 size={14} class="animate-spin" />
          {:else}
            <Plug size={14} />
          {/if}
          Connecter
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .server-row {
    display: flex;
    width: 100%;
    align-items: center;
    gap: 0.5rem;
    border-radius: 0.375rem;
    padding: 0.4rem 0.5rem;
    color: var(--color-secondary);
    text-align: left;
    font-size: 0.8125rem;
    transition: background 0.1s;
  }
  .server-row:hover {
    background: var(--color-overlay);
    color: var(--color-primary);
  }
  .server-row.active {
    background: var(--color-overlay);
    color: var(--color-primary);
    box-shadow: inset 2px 0 0 var(--color-accent);
  }
</style>
