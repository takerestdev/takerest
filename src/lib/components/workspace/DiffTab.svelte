<script>
  // @ts-nocheck
  let { data } = $props();

  import { gitDiffFile } from '$lib/commands/git.js';
  import { workspace } from '$lib/stores/workspace.svelte.js';
  import { Loader2, GitBranch, AlertTriangle } from '@lucide/svelte';

  let relPath    = $derived(data.relPath);
  let staged     = $derived(data.staged);
  let folderPath = $derived(data.folderPath);

  let diffResult  = $state(null);
  let loading     = $state(true);
  let error       = $state('');
  let loadingMore = $state(false);
  let diffMode    = $state('unified'); // 'unified' | 'split'

  let _reqId = 0;

  $effect(() => {
    workspace.fileChangeTicks[relPath]; // reload when file changes on disk (watcher)
    workspace.gitRefreshTick;           // reload when external git op changes the index
    if (!relPath || !folderPath) return;
    const id = ++_reqId;
    loading = true; error = ''; diffResult = null;
    gitDiffFile(folderPath, relPath, staged)
      .then(r  => { if (id === _reqId) diffResult = r; })
      .catch(e => { if (id === _reqId) error = e?.message ?? String(e); })
      .finally(() => { if (id === _reqId) loading = false; });
  });

  async function loadMore() {
    loadingMore = true;
    try {
      const current = diffResult?.hunks.reduce((a, h) => a + h.lines.length, 0) ?? 0;
      diffResult = await gitDiffFile(folderPath, relPath, staged, current + 10_000);
    } catch (e) { error = e?.message ?? String(e); }
    finally { loadingMore = false; }
  }

  function buildUnified(dr) {
    if (!dr) return [];
    const out = [];
    for (const hunk of dr.hunks) {
      out.push({ type: 'hunk', hunk });
      for (const line of hunk.lines) out.push({ type: 'line', line });
    }
    return out;
  }

  function buildSplit(dr) {
    if (!dr) return [];
    const out = [];
    for (const hunk of dr.hunks) {
      out.push({ type: 'hunk', hunk });
      let rem = [], add = [];
      function flush() {
        const n = Math.max(rem.length, add.length);
        for (let i = 0; i < n; i++)
          out.push({ type: 'split', left: rem[i] ?? null, right: add[i] ?? null });
        rem = []; add = [];
      }
      for (const line of hunk.lines) {
        if (line.kind === 'context') { flush(); out.push({ type: 'split', left: line, right: line }); }
        else if (line.kind === 'removed') rem.push(line);
        else add.push(line);
      }
      flush();
    }
    return out;
  }

  // ── Virtual list ──────────────────────────────────────────────────────────
  const ROW_H   = 22;
  const OVERSCAN = 15;

  let scrollEl      = $state(null);
  let leftScrollEl  = $state(null);
  let rightScrollEl = $state(null);
  let splitSyncing  = false;
  let scrollTop     = $state(0);
  let clientHeight  = $state(600);

  let activeRows  = $derived(diffMode === 'split' ? buildSplit(diffResult) : buildUnified(diffResult));

  // Max line length in characters (tabs expanded to 4 spaces) — drives min-width via ch units
  let maxCharLen = $derived(
    diffResult
      ? diffResult.hunks.flatMap(h => h.lines)
          .reduce((max, l) => Math.max(max, l.content.replace(/\t/g, '    ').length), 0)
      : 0
  );
  let startIdx    = $derived(Math.max(0, Math.floor(scrollTop / ROW_H) - OVERSCAN));
  let endIdx      = $derived(Math.min(activeRows.length - 1, Math.ceil((scrollTop + clientHeight) / ROW_H) + OVERSCAN));
  let totalHeight = $derived(activeRows.length * ROW_H);
  let visibleRows = $derived(
    activeRows.slice(startIdx, endIdx + 1).map((row, i) => ({ ...row, top: (startIdx + i) * ROW_H }))
  );

  function onUnifiedScroll(e) { scrollTop = e.currentTarget.scrollTop; }
  function onLeftScroll(e) {
    scrollTop = e.currentTarget.scrollTop;
    if (!splitSyncing && rightScrollEl) {
      splitSyncing = true;
      rightScrollEl.scrollTop = e.currentTarget.scrollTop;
      splitSyncing = false;
    }
  }
  function onRightScroll(e) {
    scrollTop = e.currentTarget.scrollTop;
    if (!splitSyncing && leftScrollEl) {
      splitSyncing = true;
      leftScrollEl.scrollTop = e.currentTarget.scrollTop;
      splitSyncing = false;
    }
  }

  // ── Style maps ─────────────────────────────────────────────────────────────
  const LINE_BG   = { added: 'bg-green-500/10 text-green-700 dark:text-green-400', removed: 'bg-red-500/10 text-red-700 dark:text-red-400', context: '' };
  const GUTTER_BG = { added: 'bg-green-500/20', removed: 'bg-red-500/20', context: 'bg-muted/30' };
  const EMPTY_BG  = 'bg-muted/5';
</script>

<div class="h-full flex flex-col overflow-hidden font-mono text-[13px] leading-5">

  <!-- Header -->
  <div class="border-b px-4 py-2 flex items-center gap-3 bg-background shrink-0 font-sans">
    <div class="flex items-center gap-2 text-sm font-medium min-w-0 flex-1">
      <GitBranch size={14} class="shrink-0 text-muted-foreground" />
      <span class="truncate">{relPath}</span>
    </div>
    <div class="flex items-center gap-3 shrink-0">
      {#if diffResult}
        <span class="text-xs text-green-600">+{diffResult.totalAdded}</span>
        <span class="text-xs text-red-500">-{diffResult.totalRemoved}</span>
        <span class="text-[10px] text-muted-foreground px-1.5 py-0.5 rounded border">
          {staged ? 'staged' : 'unstaged'}
        </span>
      {/if}
      <div class="flex items-center border rounded overflow-hidden text-[11px] select-none">
        <button type="button"
          onclick={() => { diffMode = 'unified'; scrollTop = 0; }}
          class="px-2 py-0.5 transition-colors {diffMode === 'unified' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground'}"
        >Unified</button>
        <div class="w-px bg-border h-3"></div>
        <button type="button"
          onclick={() => { diffMode = 'split'; scrollTop = 0; }}
          class="px-2 py-0.5 transition-colors {diffMode === 'split' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground'}"
        >Split</button>
      </div>
    </div>
  </div>

  <!-- States -->
  {#if loading}
    <div class="flex flex-1 items-center justify-center gap-2 text-muted-foreground">
      <Loader2 size={16} class="animate-spin" /><span class="text-sm">Computing diff…</span>
    </div>
  {:else if error}
    <div class="flex flex-1 items-center justify-center gap-2 text-destructive">
      <AlertTriangle size={16} /><span class="text-sm">{error}</span>
    </div>
  {:else if diffResult?.isBinary}
    <div class="flex flex-1 items-center justify-center text-muted-foreground text-sm">Binary file changed</div>
  {:else if !diffResult || diffResult.hunks.length === 0}
    <div class="flex flex-1 items-center justify-center text-muted-foreground text-sm">No differences</div>
  {:else if diffMode === 'unified'}

    <!-- ── Unified virtual scroll ── -->
    <div bind:this={scrollEl} bind:clientHeight class="flex-1 overflow-auto" onscroll={onUnifiedScroll}>
      <div style="height:{totalHeight}px; position:relative; min-width:calc({maxCharLen}ch + 150px);">
        {#each visibleRows as row (row.top)}
          <div style="position:absolute; top:{row.top}px; left:0; right:0; height:{ROW_H}px; display:flex; align-items:stretch;">
            {#if row.type === 'hunk'}
              <div class="flex-1 flex items-center bg-blue-500/5 border-y border-blue-500/20 px-3 text-blue-500/70 text-[11px] whitespace-nowrap">
                @@ -{row.hunk.oldStart},{row.hunk.oldLines} +{row.hunk.newStart},{row.hunk.newLines} @@
              </div>
            {:else}
              {@const line = row.line}
              {@const kind = line.kind}
              <div class="flex flex-1 min-w-0 {LINE_BG[kind] ?? ''}">
                <div class="w-12 shrink-0 flex items-center justify-end pr-2 {GUTTER_BG[kind] ?? ''} text-muted-foreground/40 select-none border-r border-border/30 text-[11px]">{line.oldLineno ?? ''}</div>
                <div class="w-12 shrink-0 flex items-center justify-end pr-2 {GUTTER_BG[kind] ?? ''} text-muted-foreground/40 select-none border-r border-border/30 text-[11px]">{line.newLineno ?? ''}</div>
                <div class="flex items-center pl-2 whitespace-pre leading-5 overflow-visible">
                  <span class="mr-2 select-none opacity-50">{kind === 'added' ? '+' : kind === 'removed' ? '-' : ' '}</span>{line.content}
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
      {#if diffResult?.truncated}
        <div class="flex items-center justify-center py-4 border-t sticky bottom-0 bg-background/80 backdrop-blur-sm font-sans">
          <button type="button" onclick={loadMore} disabled={loadingMore}
            class="text-xs text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1.5 disabled:opacity-50">
            {#if loadingMore}<Loader2 size={12} class="animate-spin" />{/if}
            Load more ({diffResult.totalAdded + diffResult.totalRemoved} lines total)…
          </button>
        </div>
      {/if}
    </div>

  {:else}

    <!-- ── Split view: two independent panes ── -->
    <div class="flex flex-1 overflow-hidden">

      <!-- Left pane (old) -->
      <div bind:this={leftScrollEl} bind:clientHeight class="flex-1 overflow-auto border-r border-border/40" onscroll={onLeftScroll}>
        <div style="height:{totalHeight}px; position:relative; min-width:calc({maxCharLen}ch + 90px);">
          {#each visibleRows as row (row.top)}
            <div style="position:absolute; top:{row.top}px; left:0; right:0; height:{ROW_H}px; display:flex; align-items:stretch;">
              {#if row.type === 'hunk'}
                <div class="flex-1 flex items-center bg-blue-500/5 border-y border-blue-500/20 px-3 text-blue-500/70 text-[11px] whitespace-nowrap">
                  @@ -{row.hunk.oldStart},{row.hunk.oldLines} +{row.hunk.newStart},{row.hunk.newLines} @@
                </div>
              {:else}
                {@const L = row.left}
                <div class="flex flex-1 {L ? (L.kind === 'removed' ? LINE_BG.removed : LINE_BG.context) : EMPTY_BG}">
                  <div class="w-10 shrink-0 flex items-center justify-end pr-1.5 {L ? (L.kind === 'removed' ? GUTTER_BG.removed : GUTTER_BG.context) : 'bg-muted/10'} text-muted-foreground/40 select-none border-r border-border/30 text-[11px]">
                    {L?.oldLineno ?? ''}
                  </div>
                  {#if L}
                    <div class="flex items-center pl-2 whitespace-pre leading-5 overflow-visible">
                      <span class="mr-1.5 select-none opacity-50">{L.kind === 'removed' ? '-' : ' '}</span>{L.content}
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>

      <!-- Right pane (new) -->
      <div bind:this={rightScrollEl} class="flex-1 overflow-auto" onscroll={onRightScroll}>
        <div style="height:{totalHeight}px; position:relative; min-width:calc({maxCharLen}ch + 90px);">
          {#each visibleRows as row (row.top)}
            <div style="position:absolute; top:{row.top}px; left:0; right:0; height:{ROW_H}px; display:flex; align-items:stretch;">
              {#if row.type === 'hunk'}
                <div class="flex-1 flex items-center bg-blue-500/5 border-y border-blue-500/20 px-3 text-blue-500/70 text-[11px] whitespace-nowrap">
                  @@ -{row.hunk.oldStart},{row.hunk.oldLines} +{row.hunk.newStart},{row.hunk.newLines} @@
                </div>
              {:else}
                {@const R = row.right}
                <div class="flex flex-1 {R ? (R.kind === 'added' ? LINE_BG.added : LINE_BG.context) : EMPTY_BG}">
                  <div class="w-10 shrink-0 flex items-center justify-end pr-1.5 {R ? (R.kind === 'added' ? GUTTER_BG.added : GUTTER_BG.context) : 'bg-muted/10'} text-muted-foreground/40 select-none border-r border-border/30 text-[11px]">
                    {R?.newLineno ?? ''}
                  </div>
                  {#if R}
                    <div class="flex items-center pl-2 whitespace-pre leading-5 overflow-visible">
                      <span class="mr-1.5 select-none opacity-50">{R.kind === 'added' ? '+' : ' '}</span>{R.content}
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    </div>

    {#if diffResult?.truncated}
      <div class="flex items-center justify-center py-3 border-t bg-background/80 backdrop-blur-sm font-sans shrink-0">
        <button type="button" onclick={loadMore} disabled={loadingMore}
          class="text-xs text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1.5 disabled:opacity-50">
          {#if loadingMore}<Loader2 size={12} class="animate-spin" />{/if}
          Load more ({diffResult.totalAdded + diffResult.totalRemoved} lines total)…
        </button>
      </div>
    {/if}

  {/if}

</div>
