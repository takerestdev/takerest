<script>
  // @ts-nocheck
  let { data } = $props();

  import { gitDiffFile } from '$lib/commands/git.js';
  import { Loader2, GitBranch, AlertTriangle } from '@lucide/svelte';

  let relPath = $derived(data.relPath);
  let staged = $derived(data.staged);
  let folderPath = $derived(data.folderPath);

  let diffResult = $state(null);
  let loading = $state(true);
  let error = $state('');
  let loadingMore = $state(false);

  // Flatten hunks + lines into a single array for virtual rendering
  let flatLines = $derived(() => {
    if (!diffResult) return [];
    const out = [];
    for (const hunk of diffResult.hunks) {
      out.push({ type: 'hunk', hunk });
      for (const line of hunk.lines) out.push({ type: 'line', line });
    }
    return out;
  });

  // ── Inline virtual list (fixed 20px rows) ──────────────────────────────────
  const ROW_H = 22;
  const OVERSCAN = 15;

  let scrollEl = $state(null);
  let scrollTop = $state(0);
  let clientHeight = $state(600);

  let startIdx = $derived(Math.max(0, Math.floor(scrollTop / ROW_H) - OVERSCAN));
  let endIdx = $derived(Math.min(flatLines().length - 1, Math.ceil((scrollTop + clientHeight) / ROW_H) + OVERSCAN));
  let totalHeight = $derived(flatLines().length * ROW_H);
  let visibleRows = $derived(
    flatLines().slice(startIdx, endIdx + 1).map((row, i) => ({ ...row, top: (startIdx + i) * ROW_H }))
  );

  // ── Data loading ───────────────────────────────────────────────────────────
  $effect(() => {
    if (!relPath || !folderPath) return;
    loading = true;
    error = '';
    diffResult = null;
    gitDiffFile(folderPath, relPath, staged)
      .then(r => { diffResult = r; })
      .catch(e => { error = e?.message ?? String(e); })
      .finally(() => { loading = false; });
  });

  async function loadMore() {
    loadingMore = true;
    try {
      const current = diffResult?.hunks.reduce((a, h) => a + h.lines.length, 0) ?? 0;
      diffResult = await gitDiffFile(folderPath, relPath, staged, current + 10_000);
    } catch (e) {
      error = e?.message ?? String(e);
    } finally {
      loadingMore = false;
    }
  }

  const LINE_BG   = { added: 'bg-green-500/10 text-green-700 dark:text-green-400', removed: 'bg-red-500/10 text-red-700 dark:text-red-400', context: '' };
  const GUTTER_BG = { added: 'bg-green-500/20', removed: 'bg-red-500/20', context: 'bg-muted/30' };
  const PREFIX    = { added: '+', removed: '-', context: ' ' };
</script>

<div class="h-full flex flex-col overflow-hidden font-mono text-[13px] leading-5">

  <!-- Header -->
  <div class="border-b px-4 py-2.5 flex items-center justify-between bg-background shrink-0">
    <div class="flex items-center gap-2 text-sm font-medium min-w-0">
      <GitBranch size={14} class="shrink-0 text-muted-foreground" />
      <span class="truncate">{relPath}</span>
    </div>
    <div class="flex items-center gap-3 shrink-0">
      {#if diffResult}
        <span class="text-green-600">+{diffResult.totalAdded}</span>
        <span class="text-red-500">-{diffResult.totalRemoved}</span>
        <span class="text-[10px] text-muted-foreground px-1.5 py-0.5 rounded border">
          {staged ? 'staged' : 'unstaged'}
        </span>
      {/if}
    </div>
  </div>

  <!-- States -->
  {#if loading}
    <div class="flex flex-1 items-center justify-center gap-2 text-muted-foreground">
      <Loader2 size={16} class="animate-spin" /><span>Computing diff...</span>
    </div>
  {:else if error}
    <div class="flex flex-1 items-center justify-center gap-2 text-destructive">
      <AlertTriangle size={16} /><span>{error}</span>
    </div>
  {:else if diffResult?.isBinary}
    <div class="flex flex-1 items-center justify-center text-muted-foreground">Binary file changed</div>
  {:else if diffResult?.hunks.length === 0}
    <div class="flex flex-1 items-center justify-center text-muted-foreground">No differences</div>
  {:else}

    <!-- Virtual scroll container -->
    <div
      bind:this={scrollEl}
      bind:clientHeight
      class="flex-1 overflow-auto"
      onscroll={(e) => { scrollTop = e.currentTarget.scrollTop; }}
    >
      <!-- Spacer that holds total height; rows are absolutely positioned inside -->
      <div style="height:{totalHeight}px; position:relative; min-width:max-content;">
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
                <!-- Old line no -->
                <div class="w-12 shrink-0 flex items-center justify-end pr-2 {GUTTER_BG[kind] ?? ''} text-muted-foreground/40 select-none border-r border-border/30 text-[11px]">
                  {line.oldLineno ?? ''}
                </div>
                <!-- New line no -->
                <div class="w-12 shrink-0 flex items-center justify-end pr-2 {GUTTER_BG[kind] ?? ''} text-muted-foreground/40 select-none border-r border-border/30 text-[11px]">
                  {line.newLineno ?? ''}
                </div>
                <!-- Prefix + content -->
                <div class="flex items-center pl-2 whitespace-pre leading-5 overflow-visible">
                  <span class="mr-2 select-none opacity-50">{PREFIX[kind]}</span>{line.content}
                </div>
              </div>
            {/if}

          </div>
        {/each}
      </div>

      <!-- Load more banner (outside the positioned container) -->
      {#if diffResult?.truncated}
        <div class="flex items-center justify-center py-4 border-t sticky bottom-0 bg-background/80 backdrop-blur-sm">
          <button
            type="button"
            onclick={loadMore}
            disabled={loadingMore}
            class="text-xs text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1.5 disabled:opacity-50"
          >
            {#if loadingMore}<Loader2 size={12} class="animate-spin" />{/if}
            Load more ({diffResult.totalAdded + diffResult.totalRemoved} lines total)...
          </button>
        </div>
      {/if}
    </div>
  {/if}

</div>
