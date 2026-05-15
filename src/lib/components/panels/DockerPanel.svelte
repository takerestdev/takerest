<script>
  // @ts-nocheck
  import { onMount } from 'svelte';
  import { workspace } from '$lib/stores/workspace.svelte.js';
  import {
    dockerListContainers, dockerListImages, dockerListComposeFiles,
    dockerContainerStart, dockerContainerStop, dockerContainerRestart,
    dockerContainerRemove, dockerImageRemove,
    dockerComposeUp, dockerComposeDown,
    dockerPing, dockerStartEngine, dockerStopEngine,
  } from '$lib/commands/docker.js';
  import {
    Play, Square, RefreshCw, Trash2, ChevronDown, ChevronRight,
    Loader2, AlertCircle, ArrowUp, ArrowDown, Terminal, PowerOff, Power,
  } from '@lucide/svelte';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';

  let projectPath = $derived(workspace.folderPath);

  // Section open/close
  let composeOpen = $state(true);
  let containersOpen = $state(true);
  let imagesOpen = $state(true);

  // Data
  let composeFiles = $state([]);
  let containers = $state([]);
  let images = $state([]);

  // Loading/error
  let loading = $state(false);
  let error = $state('');

  // Engine status: null = unknown, true = running, false = stopped
  let engineRunning = $state(null);
  let engineBusy = $state(false);
  let engineStarting = $state(false); // distinct from busy — means we're waiting for daemon
  let engineStartElapsed = $state(0); // seconds since start was clicked

  // Per-item busy state (keyed by id/path)
  let busyIds = $state(new Set());

  function setBusy(id, val) {
    busyIds = val
      ? new Set([...busyIds, id])
      : new Set([...busyIds].filter(x => x !== id));
  }

  async function loadAll() {
    if (!projectPath) return;
    loading = true;
    error = '';

    // Compose file scan is purely filesystem — runs regardless of Docker status
    dockerListComposeFiles(projectPath)
      .then(cf => { composeFiles = cf; })
      .catch(() => {});

    // Quick ping first so we know engine state immediately
    engineRunning = await dockerPing();

    if (engineRunning) {
      try {
        const [c, imgs] = await Promise.all([
          dockerListContainers(),
          dockerListImages().catch(() => []),
        ]);
        containers = c;
        images = imgs;
      } catch (e) {
        engineRunning = false;
        error = e?.toString?.() ?? String(e);
      }
    } else {
      containers = [];
      images = [];
    }

    loading = false;
  }

  async function pollTick() {
    if (!projectPath) return;
    const alive = await dockerPing();
    if (alive !== engineRunning) {
      engineRunning = alive;
      if (alive) void loadAll(); // engine came back online — refresh everything
    }
    if (alive) {
      try { containers = await dockerListContainers(); } catch { /* silent */ }
    }
  }

  async function startEngine() {
    engineBusy = true;
    engineStarting = false;
    engineStartElapsed = 0;
    error = '';

    // Real wall-clock timer — accurate regardless of how long each ping takes
    const t0 = Date.now();
    const clockTimer = setInterval(() => {
      engineStartElapsed = Math.floor((Date.now() - t0) / 1000);
    }, 500);

    try {
      await dockerStartEngine();

      // Docker Desktop with WSL2 backend can take 60-90 seconds to fully initialize
      engineStarting = true;
      let started = false;
      const deadline = Date.now() + 120_000; // 2 minute hard cap

      while (Date.now() < deadline) {
        await new Promise(r => setTimeout(r, 2000));
        if (await dockerPing()) {
          engineRunning = true;
          engineStarting = false;
          void loadAll();
          started = true;
          break;
        }
      }

      if (!started) {
        engineRunning = await dockerPing();
        engineStarting = false;
        error = `Docker did not respond after ${engineStartElapsed}s. Check Docker Desktop manually.`;
      }
    } catch (e) {
      engineStarting = false;
      error = e?.toString?.() ?? String(e);
    } finally {
      clearInterval(clockTimer);
      engineStartElapsed = 0;
      engineBusy = false;
    }
  }

  async function stopEngine() {
    engineBusy = true;
    error = '';
    try {
      await dockerStopEngine();
      await new Promise(r => setTimeout(r, 2000));
      engineRunning = await dockerPing();
      if (!engineRunning) { containers = []; images = []; }
    } catch (e) {
      error = e?.toString?.() ?? String(e);
    } finally {
      engineBusy = false;
    }
  }

  onMount(() => {
    void loadAll();
    const interval = setInterval(() => void pollTick(), 4000);
    return () => clearInterval(interval);
  });

  // Reload when workspace switches project
  $effect(() => {
    projectPath;
    void loadAll();
  });

  // ── Container actions ──────────────────────────────────────────────────────

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

  // ── Image actions ──────────────────────────────────────────────────────────

  async function removeImage(img) {
    setBusy(img.id, true);
    try {
      await dockerImageRemove(img.id, false);
      images = images.filter(i => i.id !== img.id);
    } catch (e) {
      error = e?.toString?.() ?? String(e);
    } finally {
      setBusy(img.id, false);
    }
  }

  // ── Compose actions ────────────────────────────────────────────────────────

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

  async function loadContainers() {
    if (!engineRunning) return;
    try { containers = await dockerListContainers(); } catch { /* silent */ }
  }

  // ── Helpers ────────────────────────────────────────────────────────────────

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
    {:else if engineStarting}
      <span class="w-1.5 h-1.5 rounded-full bg-yellow-400 animate-pulse shrink-0"></span>
      <span class="text-[11px] text-yellow-400/80 flex-1">
        Starting Docker…{engineStartElapsed > 0 ? ` (${engineStartElapsed}s)` : ''}
      </span>
      <Loader2 size={11} class="animate-spin text-muted-foreground shrink-0" />
    {:else if engineRunning}
      <span class="w-1.5 h-1.5 rounded-full bg-green-500 shrink-0"></span>
      <span class="text-[11px] text-foreground/70 flex-1">Engine running</span>
      <button
        type="button"
        title="Stop Docker engine"
        disabled={engineBusy}
        onclick={stopEngine}
        class="flex items-center gap-1 px-2 py-0.5 rounded text-[10px] text-muted-foreground
               hover:text-destructive hover:bg-destructive/10 transition-colors disabled:opacity-40"
      >
        {#if engineBusy}<Loader2 size={10} class="animate-spin" />{:else}<PowerOff size={10} />{/if}
        Stop
      </button>
    {:else}
      <span class="w-1.5 h-1.5 rounded-full bg-red-500 shrink-0"></span>
      <span class="text-[11px] text-muted-foreground flex-1">Engine stopped</span>
      <button
        type="button"
        title="Start Docker engine"
        disabled={engineBusy}
        onclick={startEngine}
        class="flex items-center gap-1 px-2 py-0.5 rounded text-[10px] text-green-500
               hover:bg-green-500/15 transition-colors disabled:opacity-40"
      >
        {#if engineBusy}<Loader2 size={10} class="animate-spin" />{:else}<Power size={10} />{/if}
        Start
      </button>
    {/if}
  </div>

  <!-- Error banner -->
  {#if error}
    <div class="px-3 py-1.5 flex items-start gap-1.5 bg-destructive/10 border-b shrink-0">
      <AlertCircle size={11} class="text-destructive shrink-0 mt-0.5" />
      <span class="text-destructive text-[11px] leading-tight break-all">{error}</span>
    </div>
  {/if}

  <ScrollArea class="flex-1">
    <div class="pb-3">

      <!-- ── COMPOSE section ─────────────────────────────────────────────── -->
      <div class="border-b">
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
                <!-- Up -->
                <button
                  type="button"
                  title="docker compose up -d"
                  disabled={busy}
                  onclick={() => composeAction(file, () => dockerComposeUp(projectPath, file.rel_path))}
                  class="flex items-center justify-center w-5 h-5 rounded text-green-500 hover:bg-green-500/15 transition-colors disabled:opacity-40"
                >
                  {#if busy}
                    <Loader2 size={11} class="animate-spin" />
                  {:else}
                    <ArrowUp size={11} />
                  {/if}
                </button>
                <!-- Down -->
                <button
                  type="button"
                  title="docker compose down"
                  disabled={busy}
                  onclick={() => composeAction(file, () => dockerComposeDown(projectPath, file.rel_path))}
                  class="flex items-center justify-center w-5 h-5 rounded text-red-400 hover:bg-red-500/15 transition-colors disabled:opacity-40"
                >
                  {#if busy}
                    <Loader2 size={11} class="animate-spin" />
                  {:else}
                    <ArrowDown size={11} />
                  {/if}
                </button>
              </div>
            {/each}
          {/if}
        {/if}
      </div>

      <!-- ── CONTAINERS section ──────────────────────────────────────────── -->
      <div class="border-b">
        <button
          type="button"
          onclick={() => (containersOpen = !containersOpen)}
          class="w-full flex items-center gap-1.5 px-3 py-1.5 hover:bg-muted/40 transition-colors"
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
          {#if containers.length === 0 && !loading}
            <p class="px-6 pb-2 text-[11px] text-muted-foreground/60">No containers</p>
          {:else}
            {#each containers as c (c.id)}
              {@const busy = busyIds.has(c.id)}
              {@const running = c.state === 'running'}
              {@const name = c.names[0] ?? c.short_id}
              <div class="flex items-center gap-1.5 px-3 py-1 group hover:bg-muted/30 transition-colors">
                <!-- Status dot -->
                <span class="shrink-0 w-1.5 h-1.5 rounded-full {stateDot(c.state)}"></span>

                <!-- Name (click = open logs) -->
                <button
                  type="button"
                  onclick={() => openLogsTab(c)}
                  title="{c.image}\n{c.status}"
                  class="flex-1 text-left truncate text-[11px] font-mono text-foreground/90 hover:text-foreground transition-colors"
                >{name}</button>

                <!-- Actions (visible on hover) -->
                <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                  <!-- Start / Stop -->
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

                  <!-- Restart -->
                  <button
                    type="button" title="Restart"
                    disabled={busy}
                    onclick={() => containerAction(c.id, () => dockerContainerRestart(c.id))}
                    class="flex items-center justify-center w-5 h-5 rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors disabled:opacity-40"
                  >
                    <RefreshCw size={11} />
                  </button>

                  <!-- Logs tab -->
                  <button
                    type="button" title="Open logs"
                    onclick={() => openLogsTab(c)}
                    class="flex items-center justify-center w-5 h-5 rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
                  >
                    <Terminal size={11} />
                  </button>

                  <!-- Remove -->
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
        {/if}
      </div>

      <!-- ── IMAGES section ──────────────────────────────────────────────── -->
      <div>
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
        {/if}
      </div>

    </div>
  </ScrollArea>
</div>
