<script>
  // @ts-nocheck
  let { data, tabId } = $props();

  import { listen } from '@tauri-apps/api/event';
  import { workspace } from '$lib/stores/workspace.svelte.js';
  import { dockerStartLogStream, dockerStopLogStream } from '$lib/commands/docker.js';
  import { Terminal, Search, X, Trash2, Loader2, ChevronsDown } from '@lucide/svelte';

  let containerId    = $derived(data.containerId);
  let containerName  = $derived(data.containerName);

  // ── Log state ──────────────────────────────────────────────────────────────
  let logs       = $state([]);   // { timestamp, message, stream }[]
  let searchQuery = $state('');
  let loading     = $state(false);
  let streaming   = $state(false);

  // ── Active / inactive tracking ─────────────────────────────────────────────
  let isActive = $derived(workspace.activeTabId === tabId);

  let sessionId = 0; // incremented on each activation to cancel stale listeners

  $effect(() => {
    if (isActive) {
      void activate();
    } else {
      void deactivate();
    }
    return () => { void deactivate(); };
  });

  let unlistenLog = null;
  let unlistenEnd = null;

  async function activate() {
    if (streaming) return;
    const mySession = ++sessionId;

    logs = [];
    loading = true;
    streaming = true;
    atBottom = true;

    // Subscribe to events BEFORE starting stream so we don't miss lines
    unlistenLog = await listen(`docker:log:${containerId}`, (event) => {
      if (sessionId !== mySession) return;
      batchPush(event.payload);
    });

    unlistenEnd = await listen(`docker:log-end:${containerId}`, () => {
      if (sessionId !== mySession) return;
      loading = false;
    });

    try {
      await dockerStartLogStream(containerId, 2000);
    } catch (e) {
      if (sessionId === mySession) {
        batchPush({ timestamp: null, message: `[error: ${e}]`, stream: 'stderr' });
        loading = false;
      }
    }
  }

  async function deactivate() {
    if (!streaming) return;
    streaming = false;
    sessionId++; // invalidate any pending async work

    unlistenLog?.();
    unlistenEnd?.();
    unlistenLog = null;
    unlistenEnd = null;

    try { await dockerStopLogStream(containerId); } catch {}
    loading = false;
  }

  // ── Log batching (collapse rapid bursts into one tick) ─────────────────────
  let pending = [];
  let batchFrame = null;

  function batchPush(line) {
    pending.push(line);
    if (!batchFrame) {
      batchFrame = requestAnimationFrame(() => {
        batchFrame = null;
        const batch = pending;
        pending = [];
        for (const l of batch) {
          if (logs.length >= 20_000) logs.splice(0, 1);
          logs.push(l);
        }
        if (atBottom && scrollEl) {
          scrollEl.scrollTop = scrollEl.scrollHeight;
        }
      });
    }
  }

  function clearLogs() {
    logs = [];
  }

  // ── Search ─────────────────────────────────────────────────────────────────
  let filteredLogs = $derived(
    searchQuery
      ? logs.filter(l => l.message.toLowerCase().includes(searchQuery.toLowerCase()))
      : logs
  );

  // ── Virtual scroll ─────────────────────────────────────────────────────────
  const ROW_H   = 20;
  const OVERSCAN = 25;

  let scrollEl    = $state(null);
  let scrollTop   = $state(0);
  let clientH     = $state(600);
  let atBottom    = $state(true);

  let totalHeight = $derived(filteredLogs.length * ROW_H);
  let startIdx    = $derived(Math.max(0, Math.floor(scrollTop / ROW_H) - OVERSCAN));
  let endIdx      = $derived(Math.min(filteredLogs.length - 1, Math.ceil((scrollTop + clientH) / ROW_H) + OVERSCAN));
  let visibleRows = $derived(
    filteredLogs.slice(startIdx, endIdx + 1).map((row, i) => ({ ...row, top: (startIdx + i) * ROW_H }))
  );

  function onScroll(e) {
    const el = e.currentTarget;
    scrollTop = el.scrollTop;
    atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 80;
  }

  function scrollToBottom() {
    if (scrollEl) { scrollEl.scrollTop = scrollEl.scrollHeight; atBottom = true; }
  }

  // ── Timestamp formatting ───────────────────────────────────────────────────
  function fmtTs(ts) {
    if (!ts) return '';
    try {
      const d = new Date(ts);
      const hh = String(d.getHours()).padStart(2, '0');
      const mm = String(d.getMinutes()).padStart(2, '0');
      const ss = String(d.getSeconds()).padStart(2, '0');
      const ms = String(d.getMilliseconds()).padStart(3, '0');
      return `${hh}:${mm}:${ss}.${ms}`;
    } catch { return ts.slice(11, 23); }
  }

  // ── Search highlight (XSS-safe) ────────────────────────────────────────────
  function escHtml(s) {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  function highlight(msg, q) {
    if (!q) return escHtml(msg);
    const escaped = escHtml(msg);
    const escapedQ = escHtml(q).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    return escaped.replace(
      new RegExp(escapedQ, 'gi'),
      '<mark class="bg-yellow-400/30 text-yellow-200 rounded-sm">$&</mark>'
    );
  }
</script>

<div class="h-full flex flex-col overflow-hidden font-mono text-[12px] leading-5 bg-background">

  <!-- Header -->
  <div class="flex items-center gap-2 px-3 py-1.5 border-b shrink-0 font-sans">
    <Terminal size={13} class="text-muted-foreground shrink-0" />
    <span class="text-sm font-medium flex-1 truncate">{containerName}</span>

    {#if loading}
      <Loader2 size={12} class="animate-spin text-muted-foreground shrink-0" />
    {/if}

    <span class="text-[10px] text-muted-foreground shrink-0">
      {searchQuery ? `${filteredLogs.length} / ${logs.length}` : logs.length} lines
    </span>

    <button
      type="button"
      title="Clear"
      onclick={clearLogs}
      class="text-muted-foreground hover:text-foreground transition-colors shrink-0"
    >
      <Trash2 size={13} />
    </button>
  </div>

  <!-- Search bar -->
  <div class="relative flex items-center border-b shrink-0">
    <Search size={12} class="absolute left-3 text-muted-foreground pointer-events-none" />
    <input
      type="text"
      placeholder="Search logs…"
      bind:value={searchQuery}
      class="w-full h-7 pl-8 pr-8 bg-transparent text-[12px] font-mono text-foreground placeholder:text-muted-foreground/50 outline-none"
    />
    {#if searchQuery}
      <button
        type="button"
        onclick={() => (searchQuery = '')}
        class="absolute right-2 text-muted-foreground hover:text-foreground transition-colors"
      >
        <X size={11} />
      </button>
    {/if}
  </div>

  <!-- Virtual log list -->
  <div
    bind:this={scrollEl}
    bind:clientHeight={clientH}
    class="flex-1 overflow-auto"
    onscroll={onScroll}
  >
    <div style="height:{totalHeight}px; position:relative; min-width:100%;">
      {#each visibleRows as row (row.top)}
        <div
          style="position:absolute; top:{row.top}px; left:0; right:0; height:{ROW_H}px;"
          class="flex items-start px-3 gap-3 hover:bg-muted/20 transition-colors"
        >
          <!-- Timestamp -->
          <span class="shrink-0 text-[10px] text-muted-foreground/50 select-none tabular-nums w-28 pt-px">
            {fmtTs(row.timestamp)}
          </span>
          <!-- Message -->
          <span
            class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis
              {row.stream === 'stderr' ? 'text-red-400/90' : 'text-foreground/85'}"
          >
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html highlight(row.message, searchQuery)}
          </span>
        </div>
      {/each}

      {#if filteredLogs.length === 0}
        <div class="absolute inset-0 flex items-center justify-center text-muted-foreground/40 font-sans text-sm">
          {#if loading}
            Fetching logs…
          {:else if searchQuery}
            No matches for "{searchQuery}"
          {:else}
            No logs yet
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <!-- Scroll-to-bottom button when user has scrolled up -->
  {#if !atBottom && logs.length > 0}
    <button
      type="button"
      onclick={scrollToBottom}
      class="absolute bottom-4 right-4 flex items-center gap-1.5 px-2.5 py-1 rounded-full
        bg-primary text-primary-foreground text-[11px] font-sans shadow-lg hover:opacity-90 transition-opacity"
    >
      <ChevronsDown size={12} />
      <span>Jump to bottom</span>
    </button>
  {/if}

</div>

<style>
  mark {
    background: transparent;
  }
</style>
