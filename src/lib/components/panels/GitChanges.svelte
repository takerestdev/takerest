<script>
  // @ts-nocheck
  let { projectPath, currentBranch, onOpenDiff } = $props();

  import { workspace } from '$lib/stores/workspace.svelte.js';
  import {
    gitStatus, gitStageFile, gitUnstageFile, gitStageAll, gitUnstageAll, gitCommit,
  } from '$lib/commands/git.js';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Textarea } from '$lib/components/ui/textarea/index.js';
  import * as Dialog from '$lib/components/ui/dialog/index.js';
  import GitFileTree from './GitFileTree.svelte';
  import { Loader2, RefreshCw, CheckSquare, Square } from '@lucide/svelte';

  let files = $state([]);
  let loading = $state(false);
  let error = $state('');
  let summary = $state('');
  let body = $state('');
  let committing = $state(false);
  let commitError = $state('');

  let commitOpen = $state(false);
  let stagedCount = $derived(files.filter(f => f.indexStatus).length);
  let unstagedCount = $derived(files.filter(f => f.worktreeStatus && !f.indexStatus).length + files.filter(f => f.worktreeStatus && f.indexStatus).length);
  let activeFile = $derived(
    workspace.tabs.find(t => t.id === workspace.activeTabId && t.type === 'git-diff')?.data?.relPath ?? null
  );

  async function load() {
    if (!projectPath) return;
    loading = true;
    error = '';
    try {
      files = await gitStatus(projectPath);
    } catch (e) {
      error = e?.message ?? String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (projectPath) void load();
  });

  /** Build a tree from flat file list */
  function buildTree(files) {
    const root = [];
    const dirMap = new Map();

    function getDir(parts, depth, parentArr) {
      if (depth >= parts.length - 1) return parentArr;
      const key = parts.slice(0, depth + 1).join('/');
      if (!dirMap.has(key)) {
        const node = { type: 'dir', name: parts[depth], path: key, children: [], count: 0 };
        parentArr.push(node);
        dirMap.set(key, node);
      }
      return dirMap.get(key).children;
    }

    for (const file of files) {
      const parts = file.path.split('/');
      if (parts.length === 1) {
        root.push({ type: 'file', name: parts[0], path: file.path, file });
      } else {
        let arr = root;
        for (let i = 0; i < parts.length - 1; i++) {
          arr = getDir(parts, i, arr);
        }
        arr.push({ type: 'file', name: parts[parts.length - 1], path: file.path, file });
      }
    }

    // Count changed files per dir
    function countDir(nodes) {
      let total = 0;
      for (const n of nodes) {
        if (n.type === 'file') total++;
        else { n.count = countDir(n.children); total += n.count; }
      }
      return total;
    }
    countDir(root);
    return root;
  }

  let tree = $derived(buildTree(files));

  async function handleToggle(file, shouldStage) {
    try {
      if (shouldStage) await gitStageFile(projectPath, file.path);
      else await gitUnstageFile(projectPath, file.path);
      await load();
    } catch (e) {
      error = e?.message ?? String(e);
    }
  }

  async function handleStageAll() {
    try { await gitStageAll(projectPath); await load(); }
    catch (e) { error = e?.message ?? String(e); }
  }

  async function handleUnstageAll() {
    try { await gitUnstageAll(projectPath); await load(); }
    catch (e) { error = e?.message ?? String(e); }
  }

  async function handleCommit() {
    if (!summary.trim() || stagedCount === 0) return;
    committing = true;
    commitError = '';
    try {
      await gitCommit(projectPath, summary.trim(), body.trim() || undefined);
      summary = '';
      body = '';
      commitOpen = false;
      await load();
    } catch (e) {
      commitError = e?.message ?? String(e);
    } finally {
      committing = false;
    }
  }

  function handleFileClick(file) {
    const staged = !!file.indexStatus;
    onOpenDiff(file, staged);
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Toolbar -->
  <div class="flex items-center justify-between px-2 py-1.5 border-b shrink-0">
    <span class="text-xs text-muted-foreground">
      {stagedCount} staged · {files.length - stagedCount} changed
    </span>
    <div class="flex items-center gap-1">
      {#if files.some(f => f.worktreeStatus)}
        <button type="button" title="Stage all" onclick={handleStageAll}
          class="p-1 rounded hover:bg-muted transition-colors text-muted-foreground hover:text-foreground">
          <CheckSquare size={13} />
        </button>
      {/if}
      {#if stagedCount > 0}
        <button type="button" title="Unstage all" onclick={handleUnstageAll}
          class="p-1 rounded hover:bg-muted transition-colors text-muted-foreground hover:text-foreground">
          <Square size={13} />
        </button>
      {/if}
      <button type="button" title="Refresh" onclick={load}
        class="p-1 rounded hover:bg-muted transition-colors text-muted-foreground hover:text-foreground {loading ? 'animate-spin' : ''}">
        <RefreshCw size={13} />
      </button>
    </div>
  </div>

  <!-- File tree -->
  <ScrollArea class="flex-1 min-h-0 overflow-hidden">
    {#if loading}
      <div class="flex items-center justify-center py-10 text-muted-foreground gap-2">
        <Loader2 size={14} class="animate-spin" />
        <span class="text-xs">Loading...</span>
      </div>
    {:else if error}
      <p class="text-xs text-destructive px-3 py-4">{error}</p>
    {:else if files.length === 0}
      <div class="flex flex-col items-center justify-center py-12 gap-2 text-muted-foreground">
        <span class="text-xs">No changes</span>
      </div>
    {:else}
      <GitFileTree
        nodes={tree}
        {activeFile}
        onFileClick={handleFileClick}
        onToggle={handleToggle}
      />
    {/if}
  </ScrollArea>

  <!-- Commit trigger -->
  <div class="border-t shrink-0 p-2">
    <Button
      class="w-full h-8 text-xs"
      disabled={stagedCount === 0}
      onclick={() => { commitOpen = true; commitError = ''; }}
    >
      Commit {stagedCount > 0 ? `${stagedCount} staged` : 'staged files'} to {currentBranch}
    </Button>
  </div>
</div>

<!-- Commit dialog -->
<Dialog.Root bind:open={commitOpen}>
  <Dialog.Content class="sm:max-w-sm">
    <Dialog.Header>
      <Dialog.Title>Commit to {currentBranch}</Dialog.Title>
    </Dialog.Header>
    <div class="space-y-2 py-1">
      <Input
        placeholder="Summary (required)"
        bind:value={summary}
        class="text-sm"
        autofocus
        onkeydown={(e) => e.key === 'Enter' && !e.shiftKey && handleCommit()}
      />
      <Textarea
        placeholder="Description (optional)"
        bind:value={body}
        rows={3}
        class="text-sm resize-none"
      />
      {#if commitError}
        <p class="text-xs text-destructive">{commitError}</p>
      {/if}
    </div>
    <Dialog.Footer class="gap-2">
      <Button variant="outline" onclick={() => { commitOpen = false; commitError = ''; }}>Cancel</Button>
      <Button
        disabled={!summary.trim() || stagedCount === 0 || committing}
        onclick={handleCommit}
      >
        {#if committing}<Loader2 size={13} class="mr-1.5 animate-spin" />{/if}
        Commit {stagedCount} file{stagedCount !== 1 ? 's' : ''}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
