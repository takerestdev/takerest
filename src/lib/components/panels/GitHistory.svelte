<script>
  // @ts-nocheck
  let { projectPath, onOpenCommit, refreshTick } = $props();

  import { gitLog } from '$lib/commands/git.js';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { Loader2 } from '@lucide/svelte';

  let commits = $state([]);
  let loading = $state(false);
  let error = $state('');

  let _reqId = 0;

  $effect(() => {
    refreshTick; // reload when parent signals commit/pull
    if (!projectPath) return;
    const id = ++_reqId;
    loading = true;
    error = '';
    gitLog(projectPath, 200)
      .then(c => { if (id === _reqId) commits = c; })
      .catch(e => { if (id === _reqId) error = e?.message ?? String(e); })
      .finally(() => { if (id === _reqId) loading = false; });
  });

  function formatTime(ts) {
    const d = new Date(ts * 1000);
    const now = Date.now();
    const diff = (now - d.getTime()) / 1000;
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 86400 * 7) return `${Math.floor(diff / 86400)}d ago`;
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: d.getFullYear() !== new Date().getFullYear() ? 'numeric' : undefined });
  }

  function initials(name) {
    return name.split(' ').map(p => p[0]).join('').slice(0, 2).toUpperCase();
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  {#if loading && commits.length === 0}
    <div class="flex items-center justify-center flex-1 gap-2 text-muted-foreground">
      <Loader2 size={14} class="animate-spin" />
      <span class="text-xs">Loading history...</span>
    </div>
  {:else if error}
    <p class="text-xs text-destructive px-3 py-4">{error}</p>
  {:else if commits.length === 0}
    <div class="flex items-center justify-center flex-1 text-muted-foreground">
      <span class="text-xs">No commits yet</span>
    </div>
  {:else}
    <div class="flex-1 min-h-0 overflow-y-auto divide-y divide-border/50">
      {#each commits as commit (commit.hash)}
        <button
          type="button"
          class="w-full text-left px-3 py-2.5 hover:bg-muted/50 transition-colors group"
          onclick={() => onOpenCommit(commit)}
        >
          <div class="flex items-start gap-2">
            <div class="shrink-0 w-6 h-6 rounded-full bg-muted flex items-center justify-center text-[9px] font-semibold text-muted-foreground mt-0.5">
              {initials(commit.authorName)}
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-xs font-medium truncate group-hover:text-foreground text-foreground/90 leading-tight">
                {commit.summary}
              </p>
              <div class="flex items-center gap-1.5 mt-0.5">
                <span class="font-mono text-[10px] text-muted-foreground">{commit.shortHash}</span>
                <span class="text-[10px] text-muted-foreground">·</span>
                <span class="text-[10px] text-muted-foreground truncate">{commit.authorName}</span>
                <span class="text-[10px] text-muted-foreground ml-auto shrink-0">{formatTime(commit.timestamp)}</span>
              </div>
            </div>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>
