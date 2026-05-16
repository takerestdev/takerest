<script>
  // @ts-nocheck
  let { data } = $props();

  import { listen } from '@tauri-apps/api/event';
  import { dockerStartLogStream, dockerStopLogStream, dockerExecCmd } from '$lib/commands/docker.js';
  import { Terminal, Search, X, Trash2, ChevronsDown, Clock, Loader2, FileKey, Copy, Check } from '@lucide/svelte';
  import * as Dialog from '$lib/components/ui/dialog/index.js';

  let containerId    = $derived(data.containerId);
  let containerName  = $derived(data.containerName);

  // ── Log state ──────────────────────────────────────────────────────────────
  let logs       = $state([]);   // { timestamp, message, stream }[]
  let searchQuery = $state('');
  let loading     = $state(true);   // true until first docker:log-end fires
  let streaming   = false;          // plain var — must NOT be $state (would cause effect loop)

  // ── UI prefs ───────────────────────────────────────────────────────────────
  let showTimestamps = $state(true);

  let sessionId = 0;

  // Stream starts once on mount, stays alive while tab exists (block/hidden),
  // stops only when the tab is closed (component unmounts). This preserves
  // scroll position and accumulates new logs in the background.
  $effect(() => {
    void activate();
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

    unlistenLog = await listen(`docker:log:${containerId}`, (event) => {
      if (sessionId !== mySession) return;
      batchPush(event.payload);
    });

    unlistenEnd = await listen(`docker:log-end:${containerId}`, () => {
      if (sessionId !== mySession) return;
      loading = false;
    });

    try {
      await dockerStartLogStream(containerId, 0); // 0 = all logs
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
    sessionId++;

    if (batchFrame !== null) { cancelAnimationFrame(batchFrame); batchFrame = null; }
    pending.length = 0;

    unlistenLog?.();
    unlistenEnd?.();
    unlistenLog = null;
    unlistenEnd = null;

    try { await dockerStopLogStream(containerId); } catch {}
    loading = false;
  }

  // ── Log batching ──────────────────────────────────────────────────────────
  let pending = [];
  let batchFrame = null;

  function batchPush(line) {
    pending.push(line);
    if (!batchFrame) {
      batchFrame = requestAnimationFrame(() => {
        batchFrame = null;
        const batch = pending.splice(0);
        const combined = [...logs, ...batch];
        logs = combined.length > 20_000
          ? combined.slice(combined.length - 20_000)
          : combined;
        if (atBottom) {
          requestAnimationFrame(() => {
            if (scrollEl) scrollEl.scrollTop = scrollEl.scrollHeight;
          });
        }
      });
    }
  }

  function clearLogs() {
    if (batchFrame !== null) { cancelAnimationFrame(batchFrame); batchFrame = null; }
    pending.length = 0;
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

  // ── Exec command ──────────────────────────────────────────────────────────
  let cmdInput   = $state('');
  let cmdRunning = $state(false);

  async function execCmd() {
    const cmd = cmdInput.trim();
    if (!cmd || cmdRunning) return;
    cmdInput = '';
    cmdRunning = true;

    const ts = new Date().toISOString();
    batchPush({ timestamp: ts, message: `$ ${cmd}`, stream: 'cmd' });

    try {
      const result = await dockerExecCmd(containerId, cmd);
      if (result.stdout) {
        for (const line of result.stdout.split('\n')) {
          batchPush({ timestamp: ts, message: line, stream: 'stdout' });
        }
      }
      if (result.stderr) {
        for (const line of result.stderr.split('\n')) {
          batchPush({ timestamp: ts, message: line, stream: 'stderr' });
        }
      }
    } catch (e) {
      batchPush({ timestamp: ts, message: `[exec error: ${e}]`, stream: 'stderr' });
    } finally {
      cmdRunning = false;
    }
  }

  // ── Env vars modal ────────────────────────────────────────────────────────
  let envOpen    = $state(false);
  let envVars    = $state([]);   // { key, value }[] — value is editable
  let envLoading = $state(false);
  let envError   = $state('');
  let envSearch  = $state('');
  let copiedKeys = $state(new Set());
  let copiedAll  = $state(false);

  let filteredEnv = $derived(
    envSearch
      ? envVars.filter(e =>
          e.key.toLowerCase().includes(envSearch.toLowerCase()) ||
          e.value.toLowerCase().includes(envSearch.toLowerCase()))
      : envVars
  );

  async function openEnv() {
    envOpen = true;
    if (envVars.length > 0) return; // cached — re-open is instant
    envLoading = true;
    envError = '';
    try {
      const result = await dockerExecCmd(containerId, 'printenv | sort');
      const src = result.stdout || result.stderr;
      let idCounter = 0;
      envVars = src
        .split('\n')
        .filter(Boolean)
        .map(line => {
          const eq = line.indexOf('=');
          return eq === -1
            ? { id: idCounter++, key: line, value: '' }
            : { id: idCounter++, key: line.slice(0, eq), value: line.slice(eq + 1) };
        });
      if (!result.stdout && result.stderr) envError = result.stderr;
    } catch (e) {
      envError = String(e);
    } finally {
      envLoading = false;
    }
  }

  function copyRow(id, key, value) {
    navigator.clipboard.writeText(`${key}=${value}`);
    copiedKeys = new Set([...copiedKeys, id]);
    setTimeout(() => {
      copiedKeys = new Set([...copiedKeys].filter(k => k !== id));
    }, 1500);
  }

  function copyAll() {
    const text = envVars.map(e => `${e.key}=${e.value}`).join('\n');
    navigator.clipboard.writeText(text);
    copiedAll = true;
    setTimeout(() => { copiedAll = false; }, 1500);
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

<div class="h-full flex flex-col overflow-hidden font-mono text-[12px] leading-5 bg-background relative">

  <!-- Header -->
  <div class="flex items-center gap-2 px-3 py-1.5 border-b shrink-0 font-sans">
    <Terminal size={13} class="text-muted-foreground shrink-0" />
    <span class="text-sm font-medium flex-1 truncate">{containerName}</span>

    <span class="text-[10px] text-muted-foreground shrink-0">
      {searchQuery ? `${filteredLogs.length} / ${logs.length}` : logs.length} lines
    </span>

    <button
      type="button"
      title="View environment variables"
      onclick={openEnv}
      class="flex items-center gap-1 text-[10px] font-sans text-muted-foreground hover:text-foreground transition-colors shrink-0 px-1.5 py-0.5 rounded hover:bg-muted/60"
    >
      <FileKey size={11} />
      .env
    </button>

    <button
      type="button"
      title={showTimestamps ? 'Hide timestamps' : 'Show timestamps'}
      onclick={() => (showTimestamps = !showTimestamps)}
      class="transition-colors shrink-0 {showTimestamps ? 'text-foreground/60' : 'text-muted-foreground/30'}"
    >
      <Clock size={12} />
    </button>

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
      class="w-full h-7 pl-8 pr-8 bg-transparent text-[12px] font-mono text-foreground placeholder:text-muted-foreground/50 outline-none focus:ring-0 border-0 focus:border-0"
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
    class="flex-1 overflow-auto relative"
    onscroll={onScroll}
  >
    <div style="height:{totalHeight}px; position:relative; min-width:100%;">
      {#each visibleRows as row (row.top)}
        <div
          style="position:absolute; top:{row.top}px; left:0; right:0; height:{ROW_H}px;"
          class="flex items-start px-3 {showTimestamps ? 'gap-2' : ''} hover:bg-muted/20 transition-colors"
        >
          {#if showTimestamps}
            <span class="shrink-0 text-[10px] text-muted-foreground/50 select-none tabular-nums w-18.5 pt-px">
              {fmtTs(row.timestamp)}
            </span>
          {/if}
          <span
            class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis
              {row.stream === 'stderr' ? 'text-red-400/90' : row.stream === 'cmd' ? 'text-cyan-400/80 font-semibold' : 'text-foreground/85'}"
          >
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html highlight(row.message, searchQuery)}
          </span>
        </div>
      {/each}
    </div>

    <!-- Empty state — positioned relative to the scroll container, not the 0-height inner div -->
    {#if filteredLogs.length === 0}
      <div class="absolute inset-0 flex items-center justify-center text-muted-foreground/35 font-sans text-[11px] select-none">
        {#if loading}
          connecting…
        {:else if searchQuery}
          no matches for "{searchQuery}"
        {:else}
          no logs
        {/if}
      </div>
    {/if}
  </div>

  <!-- Scroll-to-bottom button -->
  {#if !atBottom && logs.length > 0}
    <button
      type="button"
      onclick={scrollToBottom}
      class="absolute bottom-16 right-4 flex items-center gap-1.5 px-2.5 py-1 rounded-full
        bg-primary text-primary-foreground text-[11px] font-sans shadow-lg hover:opacity-90 transition-opacity"
    >
      <ChevronsDown size={12} />
      <span>Jump to bottom</span>
    </button>
  {/if}

  <!-- Command input bar -->
  <div class="flex items-center gap-2 px-3 py-1.5 border-t shrink-0 bg-muted/20">
    <span class="text-cyan-400/80 select-none shrink-0">$</span>
    <input
      type="text"
      placeholder="exec command…"
      bind:value={cmdInput}
      disabled={cmdRunning}
      onkeydown={(e) => e.key === 'Enter' && execCmd()}
      class="flex-1 bg-transparent text-[12px] font-mono text-foreground placeholder:text-muted-foreground/40
        outline-none focus:ring-0 border-0 focus:border-0 disabled:opacity-50"
    />
    <button
      type="button"
      onclick={execCmd}
      disabled={!cmdInput.trim() || cmdRunning}
      class="shrink-0 flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-sans
        bg-muted hover:bg-muted/80 text-muted-foreground hover:text-foreground transition-colors
        disabled:opacity-40 disabled:cursor-not-allowed"
    >
      {#if cmdRunning}
        <Loader2 size={11} class="animate-spin" />
      {:else}
        Run
      {/if}
    </button>
  </div>

</div>

<!-- Env vars modal -->
<Dialog.Root bind:open={envOpen}>
  <Dialog.Content class="sm:max-w-4xl w-full font-sans">
    <Dialog.Header>
      <div class="flex items-center justify-between">
        <Dialog.Title class="flex items-center gap-2 text-sm">
          <FileKey size={14} class="text-muted-foreground" />
          {containerName} — environment
        </Dialog.Title>
        <button
          type="button"
          onclick={copyAll}
          disabled={envLoading || envVars.length === 0}
          class="flex items-center gap-1.5 px-2.5 py-1 rounded text-xs mr-6
            {copiedAll ? 'text-green-500 bg-green-500/10' : 'text-muted-foreground hover:text-foreground bg-muted/50 hover:bg-muted'}
            transition-colors disabled:opacity-40"
        >
          {#if copiedAll}
            <Check size={11} />
            Copied!
          {:else}
            <Copy size={11} />
            Copy all as .env
          {/if}
        </button>
      </div>
    </Dialog.Header>

    <!-- Search -->
    <div class="relative flex items-center border rounded-md mt-2">
      <Search size={12} class="absolute left-3 text-muted-foreground pointer-events-none" />
      <input
        type="text"
        placeholder="Filter variables…"
        bind:value={envSearch}
        class="w-full h-8 pl-8 pr-3 bg-transparent text-xs outline-none focus:ring-0 border-0 focus:border-0"
      />
      {#if envSearch}
        <button type="button" onclick={() => (envSearch = '')} class="absolute right-2 text-muted-foreground hover:text-foreground">
          <X size={11} />
        </button>
      {/if}
    </div>

    <div class="overflow-y-auto max-h-[65vh] mt-2 rounded-md border bg-muted/10">
      {#if envLoading}
        <div class="flex items-center justify-center py-16 gap-2 text-muted-foreground text-sm">
          <Loader2 size={14} class="animate-spin" />
          Loading…
        </div>
      {:else if envError}
        <p class="p-4 text-destructive text-xs font-mono">{envError}</p>
      {:else if filteredEnv.length === 0}
        <p class="p-4 text-muted-foreground text-xs text-center">
          {envSearch ? 'No matches' : 'No environment variables found'}
        </p>
      {:else}
        <!-- Column headers -->
        <div class="grid grid-cols-[minmax(160px,28%)_1fr_auto] items-center px-3 py-1.5 border-b bg-muted/30 text-[10px] uppercase tracking-wider text-muted-foreground font-semibold">
          <span>Key</span>
          <span>Value</span>
          <span class="w-8"></span>
        </div>
        {#each filteredEnv as row (row.id)}
          {@const copied = copiedKeys.has(row.id)}
          <div class="grid grid-cols-[minmax(160px,28%)_1fr_auto] items-center px-3 border-b last:border-0 hover:bg-muted/30 transition-colors group font-mono text-[12px]">
            <input
              type="text"
              bind:value={row.key}
              placeholder="KEY"
              class="w-full py-2 pr-3 bg-transparent text-primary/90 placeholder:text-muted-foreground/40
                outline-none focus:ring-0 border-0 focus:border-0 focus:bg-muted/20 rounded px-1 -mx-1 transition-colors"
            />
            <input
              type="text"
              bind:value={row.value}
              placeholder="value"
              class="w-full py-2 bg-transparent text-foreground/85 outline-none focus:ring-0 border-0
                focus:border-0 focus:bg-muted/20 rounded px-1 -mx-1 transition-colors"
            />
            <div class="flex items-center justify-end w-8 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                type="button"
                onclick={() => copyRow(row.id, row.key, row.value)}
                title="Copy KEY=VALUE"
                class="flex items-center justify-center w-7 h-7 rounded transition-colors
                  {copied ? 'text-green-500' : 'text-muted-foreground hover:text-foreground hover:bg-muted'}"
              >
                {#if copied}<Check size={12} />{:else}<Copy size={12} />{/if}
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <div class="flex items-center justify-between mt-1.5">
      <p class="text-[10px] text-muted-foreground">
        {filteredEnv.length} variable{filteredEnv.length !== 1 ? 's' : ''}
        {#if envSearch && filteredEnv.length !== envVars.length} of {envVars.length}{/if}
      </p>
    </div>
  </Dialog.Content>
</Dialog.Root>

<style>
  mark {
    background: transparent;
  }
</style>
