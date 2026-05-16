<script module>
  // Persists across mount/unmount — survives tool-switching and sidebar close/reopen.
  // null = never loaded this session; set after first successful fetch.
  let _cache = null;
</script>

<script>
  // @ts-nocheck
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { workspace } from '$lib/stores/workspace.svelte.js';
  import {
    dockerListContainers, dockerListImages, dockerListComposeFiles,
    dockerContainerStart, dockerContainerStop, dockerContainerRestart,
    dockerContainerRemove, dockerImageRemove,
    dockerComposeUp, dockerComposeDown,
    dockerPing,
    dockerWatchEvents, dockerStopWatchEvents,
  } from '$lib/commands/docker.js';
  import {
    Play, Square, RefreshCw, Trash2, ChevronDown, ChevronRight,
    Loader2, AlertCircle, ArrowUp, ArrowDown, Terminal,
  } from '@lucide/svelte';

  let projectPath = $derived(workspace.folderPath);

  // Section open/close — containers first and open; compose/images secondary and closed
  let containersOpen = $state(true);
  let composeOpen    = $state(false);
  let imagesOpen     = $state(false);

  // Data — hydrated from cache immediately so UI is instant on remount
  let composeFiles = $state([]);
  let containers   = $state(_cache?.containers ?? []);
  let images       = $state(_cache?.images ?? []);
  let imagesLoaded = false; // tracks whether images have been fetched this instance

  // Spinner only when there's truly nothing to show yet
  let loading = $state(_cache === null);
  let error   = $state('');

  // Engine
  let engineRunning    = $state(_cache?.engineRunning ?? null);

  let busyIds = $state(new Set());

  function setBusy(id, val) {
    busyIds = val
      ? new Set([...busyIds, id])
      : new Set([...busyIds].filter(x => x !== id));
  }

  // ── Data loading ──────────────────────────────────────────────────────────

  async function loadAll() {
    if (!projectPath) return;
    error = '';

    // Compose files are project-scoped (FS scan, very fast — always re-fetch)
    dockerListComposeFiles(projectPath).then(cf => { composeFiles = cf; }).catch(() => {});

    // Show spinner only on first-ever load (no cache yet)
    if (_cache === null) loading = true;

    // Reset images so they re-fetch when section is open
    imagesLoaded = false;

    try {
      const c = await dockerListContainers();
      containers = c;
      engineRunning = true;
      _cache = { containers: c, images: _cache?.images ?? [], engineRunning: true };
      error = '';
    } catch {
      containers = [];
      engineRunning = false;
      _cache = { containers: [], images: [], engineRunning: false };
    }

    loading = false;

    // If images section is already open, refresh images now
    if (imagesOpen && engineRunning) {
      imagesLoaded = true;
      void loadImages();
    }
  }

  async function loadImages() {
    try {
      const imgs = await dockerListImages();
      images = imgs;
      if (_cache) _cache = { ..._cache, images: imgs };
    } catch {}
  }

  // Lazy-load images when the section is first opened
  $effect(() => {
    if (imagesOpen && !imagesLoaded && engineRunning === true) {
      imagesLoaded = true;
      void loadImages();
    }
  });

  async function loadContainers() {
    if (!engineRunning) return;
    try {
      const c = await dockerListContainers();
      containers = c;
      if (_cache) _cache = { ..._cache, containers: c };
      if (imagesOpen) void loadImages();
    } catch {}
  }

  // ── Docker event watch — replaces polling ────────────────────────────────
  let unlistenDockerEvent = null;
  let eventRefreshTimer   = null;

  async function startEventWatch() {
    unlistenDockerEvent?.();
    unlistenDockerEvent = null;
    unlistenDockerEvent = await listen('docker:event', () => {
      clearTimeout(eventRefreshTimer);
      eventRefreshTimer = setTimeout(() => void loadContainers(), 300);
    });
    try { await dockerWatchEvents(); } catch {}
  }

  function stopEventWatch() {
    clearTimeout(eventRefreshTimer);
    unlistenDockerEvent?.();
    unlistenDockerEvent = null;
    void dockerStopWatchEvents().catch(() => {});
  }

  $effect(() => {
    if (engineRunning === true)  void startEventWatch();
    else if (engineRunning === false) stopEventWatch();
  });

  // ── Engine start / stop — disabled, unreliable on Windows ───────────────
  // async function startEngine() { ... }
  // async function stopEngine() { ... }

  // Initial load + reload when project changes
  $effect(() => {
    projectPath;
    void loadAll();
  });

  onMount(() => {
    return () => stopEventWatch();
  });

  // ── Container actions ─────────────────────────────────────────────────────

  async function containerAction(id, fn) {
    setBusy(id, true);
    try {
      await fn();
      await loadContainers();
      workspace.bumpDocker();
    } catch (e) {
      error = e?.toString?.() ?? String(e);
    } finally {
      setBusy(id, false);
    }
  }

  function openLogsTab(c) {
    const name = c.names[0] ?? c.short_id;
    workspace.openTab({
      id: `docker-logs::${c.id}`,
      type: 'docker-logs',
      title: name,
      data: { containerId: c.id, containerName: name },
    });
  }

  async function removeImage(img) {
    setBusy(img.id, true);
    try {
      await dockerImageRemove(img.id, false);
      images = images.filter(i => i.id !== img.id);
      if (_cache) _cache = { ..._cache, images };
    } catch (e) {
      error = e?.toString?.() ?? String(e);
    } finally {
      setBusy(img.id, false);
    }
  }

  async function composeAction(file, fn) {
    setBusy(file.rel_path, true);
    error = '';
    try {
      await fn();
      await loadContainers();
      workspace.bumpDocker();
    } catch (e) {
      error = e?.toString?.() ?? String(e);
    } finally {
      setBusy(file.rel_path, false);
    }
  }

  // ── Helpers ───────────────────────────────────────────────────────────────

  function stateDot(state) {
    switch (state) {
      case 'running':    return 'bg-green-500';
      case 'paused':     return 'bg-yellow-400';
      case 'restarting': return 'bg-orange-400';
      case 'created':    return 'bg-blue-400';
      default:           return 'bg-muted-foreground/40';
    }
  }

  function fmtBytes(bytes) {
    if (bytes >= 1_073_741_824) return (bytes / 1_073_741_824).toFixed(1) + ' GB';
    if (bytes >= 1_048_576)     return (bytes / 1_048_576).toFixed(0) + ' MB';
    return (bytes / 1024).toFixed(0) + ' KB';
  }

  function primaryTag(img) {
    const tags = img.repo_tags.filter(t => t !== '<none>:<none>');
    return tags[0] ?? '<none>';
  }
</script>

<div class="h-full flex flex-col overflow-hidden text-xs">

  <!-- Toolbar -->
  <div class="flex items-center justify-between px-3 py-1.5 border-b shrink-0">
    {#if loading}
      <Loader2 size={12} class="animate-spin text-muted-foreground" />
    {:else}
      <span class="text-[10px] text-muted-foreground">{containers.length} containers</span>
    {/if}
    <button
      type="button"
      title="Refresh"
      onclick={() => void loadAll()}
      class="text-muted-foreground hover:text-foreground transition-colors"
    >
      <RefreshCw size={12} />
    </button>
  </div>

  <!-- Engine status bar -->
  <div class="flex items-center gap-2 px-3 py-1.5 border-b shrink-0">
    {#if engineRunning === null}
      <span class="w-1.5 h-1.5 rounded-full bg-muted-foreground/40 animate-pulse shrink-0"></span>
      <span class="text-[11px] text-muted-foreground flex-1">Checking…</span>
    {:else if engineRunning}
      <span class="w-1.5 h-1.5 rounded-full bg-green-500 shrink-0"></span>
      <span class="text-[11px] text-foreground/70 flex-1">Engine running</span>
    {:else}
      <span class="w-1.5 h-1.5 rounded-full bg-red-500 shrink-0"></span>
      <span class="text-[11px] text-muted-foreground flex-1">Engine stopped</span>
    {/if}
  </div>

  <!-- Error banner -->
  {#if error}
    <div class="px-3 py-1.5 flex items-start gap-1.5 bg-destructive/10 border-b shrink-0">
      <AlertCircle size={11} class="text-destructive shrink-0 mt-0.5" />
      <span class="text-destructive text-[11px] leading-tight break-all">{error}</span>
    </div>
  {/if}

  <!-- ── CONTAINERS — takes all remaining space, scrolls internally ───────── -->
  <div class="flex-1 flex flex-col overflow-hidden min-h-0">
    <button
      type="button"
      onclick={() => (containersOpen = !containersOpen)}
      class="w-full flex items-center gap-1.5 px-3 py-1.5 border-b hover:bg-muted/40 transition-colors shrink-0"
    >
      {#if containersOpen}
        <ChevronDown size={11} class="text-muted-foreground shrink-0" />
      {:else}
        <ChevronRight size={11} class="text-muted-foreground shrink-0" />
      {/if}
      <span class="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        Containers ({containers.length})
      </span>
    </button>

    {#if containersOpen}
      <div class="flex-1 overflow-y-auto">
        {#if containers.length === 0 && !loading}
          <p class="px-6 py-2 text-[11px] text-muted-foreground/60">No containers</p>
        {:else}
          {#each containers as c (c.id)}
            {@const busy = busyIds.has(c.id)}
            {@const running = c.state === 'running'}
            {@const name = c.names[0] ?? c.short_id}
            <div
              class="flex items-center gap-1.5 px-3 py-1 group hover:bg-muted/30 transition-colors cursor-pointer"
              role="button"
              tabindex="0"
              title="{c.image}\n{c.status}"
              onclick={() => openLogsTab(c)}
              onkeydown={(e) => e.key === 'Enter' && openLogsTab(c)}
            >
              <span class="shrink-0 w-1.5 h-1.5 rounded-full {stateDot(c.state)}"></span>

              <span class="flex-1 truncate text-[11px] font-mono text-foreground/90">{name}</span>

              <div
                class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 pointer-events-none group-hover:pointer-events-auto transition-opacity"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
              >
                {#if running}
                  <button
                    type="button" title="Stop"
                    disabled={busy}
                    onclick={() => containerAction(c.id, () => dockerContainerStop(c.id))}
                    class="flex items-center justify-center w-5 h-5 rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors disabled:opacity-40"
                  >
                    {#if busy}<Loader2 size={11} class="animate-spin" />{:else}<Square size={11} />{/if}
                  </button>
                {:else}
                  <button
                    type="button" title="Start"
                    disabled={busy}
                    onclick={() => containerAction(c.id, () => dockerContainerStart(c.id))}
                    class="flex items-center justify-center w-5 h-5 rounded text-green-500 hover:bg-green-500/15 transition-colors disabled:opacity-40"
                  >
                    {#if busy}<Loader2 size={11} class="animate-spin" />{:else}<Play size={11} />{/if}
                  </button>
                {/if}

                <button
                  type="button" title="Restart"
                  disabled={busy}
                  onclick={() => containerAction(c.id, () => dockerContainerRestart(c.id))}
                  class="flex items-center justify-center w-5 h-5 rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors disabled:opacity-40"
                >
                  <RefreshCw size={11} />
                </button>

                <button
                  type="button" title="Open logs"
                  onclick={() => openLogsTab(c)}
                  class="flex items-center justify-center w-5 h-5 rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
                >
                  <Terminal size={11} />
                </button>

                <button
                  type="button" title="Remove"
                  disabled={busy}
                  onclick={() => containerAction(c.id, () => dockerContainerRemove(c.id, true))}
                  class="flex items-center justify-center w-5 h-5 rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors disabled:opacity-40"
                >
                  <Trash2 size={11} />
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  </div>

  <!-- ── COMPOSE — pinned to bottom, expands upward ────────────────────────── -->
  <div class="shrink-0 border-t">
    <button
      type="button"
      onclick={() => (composeOpen = !composeOpen)}
      class="w-full flex items-center gap-1.5 px-3 py-1.5 hover:bg-muted/40 transition-colors"
    >
      {#if composeOpen}
        <ChevronDown size={11} class="text-muted-foreground shrink-0" />
      {:else}
        <ChevronRight size={11} class="text-muted-foreground shrink-0" />
      {/if}
      <span class="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        Compose ({composeFiles.length})
      </span>
    </button>

    {#if composeOpen}
      <div class="overflow-y-auto max-h-44">
        {#if composeFiles.length === 0}
          <p class="px-6 pb-2 text-[11px] text-muted-foreground/60">No compose files found</p>
        {:else}
          {#each composeFiles as file (file.rel_path)}
            {@const busy = busyIds.has(file.rel_path)}
            <div class="flex items-center gap-1 px-3 py-1 group hover:bg-muted/30 transition-colors">
              <button
                type="button"
                onclick={() => workspace.openTab({
                  id: `file-edit::${file.rel_path}`,
                  type: 'file-edit',
                  title: file.name,
                  data: { projectPath, relPath: file.rel_path, language: 'yaml' },
                })}
                class="flex-1 truncate text-left text-[11px] text-foreground/80 font-mono hover:text-foreground transition-colors"
                title={file.rel_path}
              >
                {file.name}
              </button>
              <button
                type="button"
                title="docker compose up -d"
                disabled={busy}
                onclick={() => composeAction(file, () => dockerComposeUp(projectPath, file.rel_path))}
                class="flex items-center justify-center w-5 h-5 rounded text-green-500 hover:bg-green-500/15 transition-colors disabled:opacity-40"
              >
                {#if busy}<Loader2 size={11} class="animate-spin" />{:else}<ArrowUp size={11} />{/if}
              </button>
              <button
                type="button"
                title="docker compose down"
                disabled={busy}
                onclick={() => composeAction(file, () => dockerComposeDown(projectPath, file.rel_path))}
                class="flex items-center justify-center w-5 h-5 rounded text-red-400 hover:bg-red-500/15 transition-colors disabled:opacity-40"
              >
                {#if busy}<Loader2 size={11} class="animate-spin" />{:else}<ArrowDown size={11} />{/if}
              </button>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  </div>

  <!-- ── IMAGES — pinned to bottom ─────────────────────────────────────────── -->
  <div class="shrink-0 border-t">
    <button
      type="button"
      onclick={() => (imagesOpen = !imagesOpen)}
      class="w-full flex items-center gap-1.5 px-3 py-1.5 hover:bg-muted/40 transition-colors"
    >
      {#if imagesOpen}
        <ChevronDown size={11} class="text-muted-foreground shrink-0" />
      {:else}
        <ChevronRight size={11} class="text-muted-foreground shrink-0" />
      {/if}
      <span class="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        Images ({images.length})
      </span>
    </button>

    {#if imagesOpen}
      <div class="overflow-y-auto max-h-44">
        {#if images.length === 0}
          <p class="px-6 pb-2 text-[11px] text-muted-foreground/60">No images</p>
        {:else}
          {#each images as img (img.id)}
            {@const busy = busyIds.has(img.id)}
            <div class="flex items-center gap-1.5 px-3 py-1 group hover:bg-muted/30 transition-colors">
              <div class="flex-1 min-w-0">
                <p class="truncate text-[11px] font-mono text-foreground/90">{primaryTag(img)}</p>
                <p class="text-[10px] text-muted-foreground/60">{fmtBytes(img.size)}</p>
              </div>
              <button
                type="button" title="Remove image"
                disabled={busy}
                onclick={() => removeImage(img)}
                class="flex items-center justify-center w-5 h-5 rounded opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-all disabled:opacity-40"
              >
                {#if busy}<Loader2 size={11} class="animate-spin" />{:else}<Trash2 size={11} />{/if}
              </button>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  </div>

</div>
