<script>
  // @ts-nocheck
  let { data } = $props();

  import { tick } from 'svelte';
  import { gitCommitFiles, gitDiffCommitFile, gitReadBlobAtCommit } from '$lib/commands/git.js';
  import { GitCommit, User, Clock, Hash, Loader2, AlertTriangle } from '@lucide/svelte';

  let commit     = $derived(data.commit);
  let folderPath = $derived(data.folderPath);
  let parentHash = $derived(commit.parents[0] ?? null);

  // ── State ──────────────────────────────────────────────────────────────────
  let files         = $state([]);
  let filesLoading  = $state(true);
  let filesError    = $state('');
  // Plain object instead of Map — Svelte 5's object property reactivity is
  // reliable from async callbacks; Map.get() tracking in $derived.by is not.
  // path → null (pending) | { error: string, result: any|null } (settled)
  let fileDiffs     = $state({});
  // path → null (loading) | { error, before, after } (settled image blobs)
  let imageBlobs    = $state({});
  let activeFileIdx = $state(0);

  let scrollEl     = $state(null);
  let sidebarEl    = $state(null);
  let scrollTop    = $state(0);
  let clientHeight = $state(500);
  let clientWidth  = $state(800);

  // ── Fetch queue — plain JS, zero reactivity ────────────────────────────────
  let _gen       = 0;
  let _queue     = [];
  let _inFlight  = 0;
  let _requested = new Set();
  const MAX_CONCURRENT = 4;

  function resetQueue() { _gen++; _queue = []; _inFlight = 0; _requested = new Set(); }

  function enqueueFile(file) {
    if (_requested.has(file.path)) return;
    _requested.add(file.path);
    _queue.push({ file, gen: _gen });
    drain();
  }

  function drain() {
    while (_inFlight < MAX_CONCURRENT && _queue.length > 0) {
      const { file, gen } = _queue.shift();
      _inFlight++;
      gitDiffCommitFile(folderPath, commit.hash, file.path, parentHash)
        .then(result => {
          if (gen !== _gen) return;
          fileDiffs[file.path] = { error: '', result };
          if (result.isImage) fetchImageBlob(file, gen);
          tick().then(applyPendingScroll); // correct scroll if heights changed
        })
        .catch(e => {
          if (gen !== _gen) return;
          fileDiffs[file.path] = { error: e?.message ?? String(e), result: null };
        })
        .finally(() => { if (gen === _gen) { _inFlight--; drain(); } });
    }
  }

  function fetchImageBlob(file, gen) {
    const kind = fileKind(file);
    const tasks = [];
    if (kind !== 'added' && parentHash) {
      tasks.push(
        gitReadBlobAtCommit(folderPath, parentHash, file.path)
          .then(b => ({ before: b }))
          .catch(() => ({ before: null }))
      );
    }
    if (kind !== 'deleted') {
      tasks.push(
        gitReadBlobAtCommit(folderPath, commit.hash, file.path)
          .then(b => ({ after: b }))
          .catch(() => ({ after: null }))
      );
    }
    Promise.all(tasks).then(results => {
      if (gen !== _gen) return;
      imageBlobs[file.path] = { error: '', ...Object.assign({}, ...results) };
    }).catch(e => {
      if (gen !== _gen) return;
      imageBlobs[file.path] = { error: e?.message ?? String(e), before: null, after: null };
    });
  }

  // ── Load file list on commit change ───────────────────────────────────────
  $effect(() => {
    if (!commit?.hash || !folderPath) return;
    resetQueue();
    filesLoading = true; filesError = ''; files = []; fileDiffs = {}; imageBlobs = {};
    activeFileIdx = 0; scrollTop = 0;

    const gen = _gen;
    gitCommitFiles(folderPath, commit.hash)
      .then(f => {
        if (gen !== _gen) return;
        // Initialise all as null (pending) before setting files,
        // so allRows sees them immediately when files becomes non-empty.
        const pending = {};
        for (const file of f) pending[file.path] = null;
        fileDiffs = pending;
        files = f;
        filesLoading = false;
      })
      .catch(e => {
        if (gen !== _gen) return;
        filesError = e?.message ?? String(e); filesLoading = false;
      });
  });

  // ── Row heights ────────────────────────────────────────────────────────────
  const FILE_H        = 40;
  const ROW_H         = 22;
  const PLACEHOLDER_H = 40;
  const IMAGE_H       = 320;
  const OVERSCAN_PX   = 400;
  const PRELOAD_AHEAD = 5;

  // ── Flat row array ─────────────────────────────────────────────────────────
  let allRows = $derived.by(() => {
    const rows = [];
    for (let fi = 0; fi < files.length; fi++) {
      const file = files[fi];
      const diff = fileDiffs[file.path]; // null = pending, undefined = not ready, obj = settled
      rows.push({ type: 'file-header', fi, file });
      if (!diff) {
        rows.push({ type: 'loading', fi });
      } else if (diff.error) {
        rows.push({ type: 'error', fi, error: diff.error });
      } else if (diff.result?.isBinary) {
        rows.push({ type: 'binary', fi });
      } else if (diff.result?.isImage) {
        rows.push({ type: 'image', fi, file });
      } else if (!diff.result || diff.result.hunks.length === 0) {
        rows.push({ type: 'empty', fi });
      } else {
        for (const hunk of diff.result.hunks) {
          rows.push({ type: 'hunk', fi, hunk });
          for (const line of hunk.lines) rows.push({ type: 'line', fi, line });
        }
      }
    }
    return rows;
  });

  // Cumulative top positions (length = allRows.length + 1)
  let positions = $derived.by(() => {
    const n   = allRows.length;
    const pos = new Float64Array(n + 1);
    let off = 0;
    for (let i = 0; i < n; i++) {
      pos[i] = off;
      const r = allRows[i];
      off += r.type === 'file-header' ? FILE_H
           : r.type === 'image' ? IMAGE_H
           : (r.type === 'loading' || r.type === 'binary' || r.type === 'empty' || r.type === 'error') ? PLACEHOLDER_H
           : ROW_H;
    }
    pos[n] = off;
    return pos;
  });

  let totalHeight = $derived(positions.length > 0 ? positions[positions.length - 1] : 0);

  // Global row index for each file's header
  let fileHeaderPos = $derived.by(() => {
    const arr = new Array(files.length).fill(-1);
    for (let i = 0; i < allRows.length; i++) {
      const r = allRows[i];
      if (r.type === 'file-header' && arr[r.fi] === -1) arr[r.fi] = i;
    }
    return arr;
  });

  // ── Virtualization (binary search) ─────────────────────────────────────────
  let startIdx = $derived.by(() => {
    if (allRows.length === 0) return 0;
    const target = Math.max(0, scrollTop - OVERSCAN_PX);
    let lo = 0, hi = allRows.length - 1;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (positions[mid + 1] <= target) lo = mid + 1; else hi = mid;
    }
    return lo;
  });

  let endIdx = $derived.by(() => {
    if (allRows.length === 0) return 0;
    const target = scrollTop + clientHeight + OVERSCAN_PX;
    let lo = 0, hi = allRows.length - 1;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (positions[mid] < target) lo = mid; else hi = mid - 1;
    }
    return Math.min(allRows.length - 1, lo);
  });

  let visibleRows = $derived(
    allRows.slice(startIdx, endIdx + 1).map((row, i) => ({
      ...row,
      _idx:   startIdx + i,
      top:    positions[startIdx + i],
      height: positions[startIdx + i + 1] - positions[startIdx + i]
    }))
  );

  // Max line length for horizontal scroll width
  let maxCharLen = $derived.by(() => {
    let max = 80;
    for (const d of Object.values(fileDiffs)) {
      if (d?.result?.hunks) {
        for (const h of d.result.hunks) {
          for (const l of h.lines) {
            const len = l.content.replace(/\t/g, '    ').length;
            if (len > max) max = len;
          }
        }
      }
    }
    return max;
  });

  // ── Lazy load — reads visibleRows only, _requested is non-reactive ──────────
  $effect(() => {
    if (files.length === 0) return;
    let maxFi = -1;
    const seen = new Set();
    for (const row of visibleRows) {
      seen.add(row.fi);
      if (row.fi > maxFi) maxFi = row.fi;
    }
    for (const fi of seen) {
      const f = files[fi];
      if (f && !_requested.has(f.path)) enqueueFile(f);
    }
    for (let fi = maxFi + 1; fi <= Math.min(files.length - 1, maxFi + PRELOAD_AHEAD); fi++) {
      const f = files[fi];
      if (f && !_requested.has(f.path)) enqueueFile(f);
    }
  });

  // ── Scroll handler + active file tracking ──────────────────────────────────
  let _rafId = null;
  $effect(() => () => { if (_rafId) cancelAnimationFrame(_rafId); });

  function onScroll(e) {
    const st = e.currentTarget.scrollTop;
    if (_rafId) cancelAnimationFrame(_rafId);
    _rafId = requestAnimationFrame(() => {
      scrollTop = st;
      // Last file header at or above the top of the viewport
      const ref = st + 2;
      let active = 0;
      for (let fi = 0; fi < files.length; fi++) {
        const ri = fileHeaderPos[fi];
        if (ri >= 0 && positions[ri] <= ref) active = fi;
        else break;
      }
      activeFileIdx = active;
      _rafId = null;
    });
  }

  // Keep active file button visible in sidebar
  $effect(() => {
    const fi = activeFileIdx;
    if (!sidebarEl) return;
    const btn = sidebarEl.querySelector(`[data-fi="${fi}"]`);
    if (btn) btn.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  });

  let _pendingScrollFi = -1;
  let _pendingScrollTimer = null;

  async function scrollToFile(fi) {
    // Cancel previous pending navigation
    if (_pendingScrollTimer) { clearTimeout(_pendingScrollTimer); _pendingScrollTimer = null; }
    _pendingScrollFi = fi;
    // Enqueue diffs for all files above the target so heights stabilise quickly
    for (let i = 0; i <= Math.min(fi, files.length - 1); i++) {
      const f = files[i];
      if (f && !_requested.has(f.path)) enqueueFile(f);
    }
    await tick();
    applyPendingScroll();
    // After 1.5 s positions are stable — stop re-scrolling
    _pendingScrollTimer = setTimeout(() => { _pendingScrollFi = -1; _pendingScrollTimer = null; }, 1500);
  }

  function applyPendingScroll() {
    if (_pendingScrollFi < 0 || !scrollEl) return;
    const ri = fileHeaderPos[_pendingScrollFi];
    if (ri >= 0) scrollEl.scrollTo({ top: positions[ri], behavior: 'instant' });
  }

  // ── Helpers ────────────────────────────────────────────────────────────────
  function formatDate(ts) {
    return new Date(ts * 1000).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
  }

  const DOT_COLOR  = { added: 'bg-green-500', modified: 'bg-yellow-400', deleted: 'bg-red-500', renamed: 'bg-blue-400' };
  const NAME_COLOR = { added: 'text-green-600 dark:text-green-400', modified: '', deleted: 'text-red-500 opacity-60', renamed: 'text-blue-500' };
  const LINE_BG    = { added: 'bg-green-500/10 text-green-700 dark:text-green-400', removed: 'bg-red-500/10 text-red-700 dark:text-red-400', context: '' };
  const GUTTER_BG  = { added: 'bg-green-500/20', removed: 'bg-red-500/20', context: 'bg-muted/30' };
  const PREFIX     = { added: '+', removed: '-', context: ' ' };

  function fileKind(file) { return file.indexStatus?.type ?? 'modified'; }
  function blobUrl(b) { return `data:${b.mime};base64,${b.data}`; }
</script>

<div class="h-full flex flex-col overflow-hidden font-sans">

  <!-- Commit header -->
  <div class="border-b px-5 py-3.5 bg-background shrink-0">
    <div class="flex items-start gap-3">
      <GitCommit size={16} class="shrink-0 text-muted-foreground mt-0.5" />
      <div class="flex-1 min-w-0">
        <h2 class="text-sm font-semibold leading-snug">{commit.summary}</h2>
        {#if commit.body}
          <p class="text-xs text-muted-foreground mt-1 whitespace-pre-wrap">{commit.body}</p>
        {/if}
      </div>
    </div>
    <div class="mt-3 flex flex-wrap gap-x-5 gap-y-1 text-xs text-muted-foreground">
      <span class="flex items-center gap-1.5"><Hash size={10} /><span class="font-mono">{commit.hash.slice(0, 12)}</span></span>
      <span class="flex items-center gap-1.5"><User size={10} />{commit.authorName}</span>
      <span class="flex items-center gap-1.5"><Clock size={10} />{formatDate(commit.timestamp)}</span>
      {#if commit.parents.length > 0}
        <span class="flex items-center gap-1.5 text-muted-foreground/60">
          Parents:
          {#each commit.parents as p}<span class="font-mono">{p.slice(0, 8)}</span>{/each}
        </span>
      {/if}
    </div>
  </div>

  <!-- Body: sidebar + diff document -->
  <div class="flex flex-1 min-h-0 overflow-hidden">

    <!-- File list sidebar -->
    <div class="w-64 shrink-0 border-r flex flex-col min-h-0">
      <div class="px-3 py-2 border-b shrink-0">
        <span class="text-[11px] font-medium text-muted-foreground uppercase tracking-wide">
          {#if filesLoading}Loading…{:else}{files.length} file{files.length !== 1 ? 's' : ''} changed{/if}
        </span>
      </div>
      <div class="flex-1 min-h-0 overflow-y-auto" bind:this={sidebarEl}>
        {#if filesError}
          <p class="text-xs text-destructive px-3 py-3">{filesError}</p>
        {:else}
          <div class="py-1">
            {#each files as file, fi (file.path)}
              {@const kind = fileKind(file)}
              <button
                type="button"
                data-fi={fi}
                onclick={() => scrollToFile(fi)}
                class="w-full flex items-center gap-2 px-3 py-1.5 text-left text-[13px] transition-colors
                  {activeFileIdx === fi ? 'bg-muted text-foreground' : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'}"
              >
                <span class="shrink-0 w-1.5 h-1.5 rounded-full {DOT_COLOR[kind] ?? 'bg-muted-foreground/40'}"></span>
                <span class="truncate flex-1 {NAME_COLOR[kind] ?? ''}">{file.path.split('/').pop()}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <!-- All-files diff document -->
    {#if filesLoading}
      <div class="flex flex-1 items-center justify-center gap-2 text-muted-foreground">
        <Loader2 size={15} class="animate-spin" /><span class="text-sm">Loading files…</span>
      </div>
    {:else if filesError}
      <div class="flex flex-1 items-center justify-center gap-2 text-destructive">
        <AlertTriangle size={15} /><span class="text-sm">{filesError}</span>
      </div>
    {:else if files.length === 0}
      <div class="flex flex-1 items-center justify-center text-muted-foreground text-sm">No files changed</div>
    {:else}
      <div
        bind:this={scrollEl}
        bind:clientHeight
        bind:clientWidth
        class="flex-1 min-w-0 overflow-auto font-mono text-[13px]"
        onscroll={onScroll}
      >
        <div style="height:{totalHeight}px; position:relative; min-width:calc({maxCharLen}ch + 150px);">
          {#each visibleRows as row (row._idx)}
            <div style="position:absolute; top:{row.top}px; left:0; right:0; height:{row.height}px; display:flex; align-items:stretch;">

              {#if row.type === 'file-header'}
                {@const kind = fileKind(row.file)}
                {@const diff = fileDiffs[row.file.path]}
                <div class="flex flex-1 items-center gap-2.5 px-4 bg-muted/40 border-t border-border font-sans">
                  <span class="shrink-0 w-1.5 h-1.5 rounded-full {DOT_COLOR[kind] ?? 'bg-muted-foreground/40'}"></span>
                  <span class="flex-1 min-w-0 truncate text-[13px] font-medium text-foreground">{row.file.path}</span>
                  {#if diff?.result && !diff.result.isBinary}
                    <span class="shrink-0 text-[11px] tabular-nums text-green-600 dark:text-green-400">+{diff.result.totalAdded}</span>
                    <span class="shrink-0 text-[11px] tabular-nums text-red-500">-{diff.result.totalRemoved}</span>
                  {/if}
                </div>

              {:else if row.type === 'loading'}
                <div class="flex flex-1 items-center justify-center gap-1.5 text-muted-foreground/40 font-sans text-xs">
                  <Loader2 size={11} class="animate-spin" />Computing diff…
                </div>

              {:else if row.type === 'error'}
                <div class="flex flex-1 items-center gap-2 px-4 text-destructive/70 font-sans text-xs">
                  <AlertTriangle size={12} />{row.error}
                </div>

              {:else if row.type === 'binary'}
                <div class="flex flex-1 items-center px-4 text-muted-foreground/50 font-sans text-xs italic">Binary file changed</div>

              {:else if row.type === 'image'}
                {@const blob = imageBlobs[row.file.path]}
                {@const kind = fileKind(row.file)}
                <!-- Sticky wrapper pinned to the scroll viewport: the row spans the
                     full (1500px+) inner width, so the image must be confined to the
                     visible width or object-fit:contain centers it off-screen. -->
                <div style="position:sticky; left:0; flex:none; width:{clientWidth}px; height:100%; display:flex; align-items:stretch;">
                  {#if !blob}
                    <div class="flex flex-1 items-center justify-center gap-1.5 text-muted-foreground/40 font-sans text-xs">
                      <Loader2 size={11} class="animate-spin" />Loading image…
                    </div>
                  {:else if blob.error}
                    <div class="flex flex-1 items-center justify-center gap-2 text-destructive/70 font-sans text-xs">
                      <AlertTriangle size={12} />{blob.error}
                    </div>
                  {:else if (kind === 'added' || kind === 'modified' || kind === 'renamed') && blob.after}
                    <img src={blobUrl(blob.after)} alt="" style="flex:1; min-width:0; height:100%; object-fit:contain; padding:16px; box-sizing:border-box;" />
                  {:else if blob.before}
                    <img src={blobUrl(blob.before)} alt="" style="flex:1; min-width:0; height:100%; object-fit:contain; padding:16px; box-sizing:border-box;" />
                  {:else}
                    <div class="flex flex-1 items-center justify-center font-sans text-xs text-muted-foreground">Not available</div>
                  {/if}
                </div>

              {:else if row.type === 'empty'}
                <div class="flex flex-1 items-center px-4 text-muted-foreground/50 font-sans text-xs italic">No differences</div>

              {:else if row.type === 'hunk'}
                <div class="flex-1 flex items-center bg-blue-500/5 border-y border-blue-500/20 px-3 text-blue-500/70 text-[11px] whitespace-nowrap">
                  @@ -{row.hunk.oldStart},{row.hunk.oldLines} +{row.hunk.newStart},{row.hunk.newLines} @@
                </div>

              {:else if row.type === 'line'}
                {@const line = row.line}
                {@const kind = line.kind}
                <div class="flex flex-1 min-w-0 {LINE_BG[kind] ?? ''}">
                  <div class="w-12 shrink-0 flex items-center justify-end pr-2 {GUTTER_BG[kind] ?? ''} text-muted-foreground/40 select-none border-r border-border/30 text-[11px]">{line.oldLineno ?? ''}</div>
                  <div class="w-12 shrink-0 flex items-center justify-end pr-2 {GUTTER_BG[kind] ?? ''} text-muted-foreground/40 select-none border-r border-border/30 text-[11px]">{line.newLineno ?? ''}</div>
                  <div class="flex items-center pl-2 whitespace-pre leading-5 overflow-visible">
                    <span class="mr-2 select-none opacity-50">{PREFIX[kind]}</span>{line.content}
                  </div>
                </div>

              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
