<script>
  // @ts-nocheck
  let { data } = $props();

  import { gitCommitFiles, gitDiffCommitFile } from '$lib/commands/git.js';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { GitCommit, User, Clock, Hash, Loader2, AlertTriangle } from '@lucide/svelte';

  let commit = $derived(data.commit);
  let folderPath = $derived(data.folderPath);
  let parentHash = $derived(commit.parents[0] ?? null);

  // File list
  let files = $state([]);
  let filesLoading = $state(true);
  let filesError = $state('');
  let selectedFile = $state(null);

  // Diff state
  let diffResult = $state(null);
  let diffLoading = $state(false);
  let diffError = $state('');

  $effect(() => {
    if (!commit?.hash || !folderPath) return;
    filesLoading = true; filesError = ''; files = []; selectedFile = null;
    gitCommitFiles(folderPath, commit.hash)
      .then(f => { files = f; if (f.length > 0) selectFile(f[0]); })
      .catch(e => { filesError = e?.message ?? String(e); })
      .finally(() => { filesLoading = false; });
  });

  async function selectFile(file) {
    selectedFile = file;
    diffResult = null; diffError = ''; diffLoading = true; scrollTop = 0;
    try {
      diffResult = await gitDiffCommitFile(folderPath, commit.hash, file.path, parentHash);
    } catch (e) {
      diffError = e?.message ?? String(e);
    } finally {
      diffLoading = false;
    }
  }

  function formatDate(ts) {
    return new Date(ts * 1000).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
  }

  // Status dot colors
  const DOT_COLOR = { added: 'bg-green-500', modified: 'bg-yellow-400', deleted: 'bg-red-500', renamed: 'bg-blue-400' };
  const NAME_COLOR = { added: 'text-green-600 dark:text-green-400', modified: '', deleted: 'text-red-500 opacity-60', renamed: 'text-blue-500' };

  function fileKind(file) {
    return file.indexStatus?.type ?? 'modified';
  }

  // ── Diff virtual list ──────────────────────────────────────────────────────
  const ROW_H = 22;
  const OVERSCAN = 15;

  let scrollEl      = $state(null);
  let leftScrollEl  = $state(null);
  let rightScrollEl = $state(null);
  let splitSyncing  = false;
  let scrollTop     = $state(0);
  let clientHeight  = $state(500);
  let diffMode      = $state('unified'); // 'unified' | 'split'

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

  let activeRows  = $derived(diffMode === 'split' ? buildSplit(diffResult) : buildUnified(diffResult));

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

  const LINE_BG   = { added: 'bg-green-500/10 text-green-700 dark:text-green-400', removed: 'bg-red-500/10 text-red-700 dark:text-red-400', context: '' };
  const GUTTER_BG = { added: 'bg-green-500/20', removed: 'bg-red-500/20', context: 'bg-muted/30' };
  const EMPTY_BG  = 'bg-muted/5';
  const PREFIX    = { added: '+', removed: '-', context: ' ' };
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

  <!-- Body: file list + diff -->
  <div class="flex flex-1 min-h-0 overflow-hidden">

    <!-- File list sidebar -->
    <div class="w-64 shrink-0 border-r flex flex-col min-h-0">
      <div class="px-3 py-2 border-b shrink-0">
        <span class="text-[11px] font-medium text-muted-foreground uppercase tracking-wide">
          {#if filesLoading}Loading...{:else}{files.length} file{files.length !== 1 ? 's' : ''} changed{/if}
        </span>
      </div>
      <div class="flex-1 min-h-0 overflow-y-auto">
        {#if filesError}
          <p class="text-xs text-destructive px-3 py-3">{filesError}</p>
        {:else}
          <div class="py-1">
            {#each files as file (file.path)}
              {@const kind = fileKind(file)}
              <button
                type="button"
                onclick={() => selectFile(file)}
                class="w-full flex items-center gap-2 px-3 py-1.5 text-left text-[13px] transition-colors
                  {selectedFile?.path === file.path ? 'bg-muted text-foreground' : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'}"
              >
                <span class="shrink-0 w-1.5 h-1.5 rounded-full {DOT_COLOR[kind] ?? 'bg-muted-foreground/40'}"></span>
                <span class="truncate flex-1 {NAME_COLOR[kind] ?? ''}">{file.path.split('/').pop()}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <!-- Diff view -->
    <div class="flex-1 min-w-0 overflow-hidden flex flex-col font-mono text-[13px]">
      {#if !selectedFile}
        <div class="flex flex-1 items-center justify-center text-muted-foreground text-sm">Select a file to view its diff</div>
      {:else if diffLoading}
        <div class="flex flex-1 items-center justify-center gap-2 text-muted-foreground">
          <Loader2 size={15} class="animate-spin" /><span class="text-sm">Computing diff…</span>
        </div>
      {:else if diffError}
        <div class="flex flex-1 items-center justify-center gap-2 text-destructive">
          <AlertTriangle size={15} /><span class="text-sm">{diffError}</span>
        </div>
      {:else if diffResult?.isBinary}
        <div class="flex flex-1 items-center justify-center text-muted-foreground text-sm">Binary file</div>
      {:else if !diffResult || diffResult.hunks.length === 0}
        <div class="flex flex-1 items-center justify-center text-muted-foreground text-sm">No differences</div>
      {:else}
        <!-- Diff header -->
        <div class="border-b px-4 py-2 flex items-center gap-3 bg-background shrink-0 font-sans">
          <span class="text-sm truncate text-muted-foreground flex-1 min-w-0">{selectedFile.path}</span>
          <div class="flex items-center gap-3 shrink-0 text-xs">
            <span class="text-green-600">+{diffResult.totalAdded}</span>
            <span class="text-red-500">-{diffResult.totalRemoved}</span>
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

        {#if diffMode === 'unified'}
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
                        <span class="mr-2 select-none opacity-50">{PREFIX[kind]}</span>{line.content}
                      </div>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
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
        {/if}
      {/if}
    </div>
  </div>
</div>
