<script>
  // @ts-nocheck
  import { workspace } from '$lib/stores/workspace.svelte.js';
  import { listDocFiles, deleteDocFile } from '$lib/commands/files.js';
  import { FileText, ChevronRight, ChevronDown, Loader2, BookOpen } from '@lucide/svelte';
  import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
  import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { toast } from 'svelte-sonner';

  let projectPath = $derived(workspace.folderPath);
  let files = $state([]);
  let loading = $state(true);
  let error = $state('');
  let expanded = $state(new Set());

  let deleteTarget = $state(null);
  let deleteConfirmOpen = $state(false);
  let deleting = $state(false);

  function absPath(relPath) {
    const base = projectPath ?? '';
    const sep = base.includes('\\') ? '\\' : '/';
    return base.replace(/[/\\]$/, '') + sep + relPath.replace(/\//g, sep);
  }

  async function copyToClipboard(text) {
    try { await navigator.clipboard.writeText(text); } catch {}
  }

  async function showInExplorer(relPath) {
    try { await revealItemInDir(absPath(relPath)); } catch (e) { console.error(e); }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    deleting = true;
    try {
      await deleteDocFile(projectPath, deleteTarget);
      workspace.closeTab(`doc::${deleteTarget}`);
      await load();
      deleteConfirmOpen = false;
      deleteTarget = null;
      toast.success('File deleted');
    } catch (e) {
      toast.error(e?.message ?? String(e));
    } finally {
      deleting = false;
    }
  }

  function buildTree(paths) {
    const root = [];
    const dirMap = new Map();
    for (const relPath of paths) {
      const parts = relPath.split('/');
      let arr = root;
      for (let i = 0; i < parts.length - 1; i++) {
        const key = parts.slice(0, i + 1).join('/');
        if (!dirMap.has(key)) {
          const node = { type: 'dir', name: parts[i], key, children: [] };
          arr.push(node);
          dirMap.set(key, node);
        }
        arr = dirMap.get(key).children;
      }
      arr.push({ type: 'file', name: parts[parts.length - 1], path: relPath });
    }
    return root;
  }

  async function load() {
    if (!projectPath) return;
    loading = true; error = '';
    try {
      files = await listDocFiles(projectPath);
      const tree = buildTree(files);
      for (const node of tree) {
        if (node.type === 'dir') expanded.add(node.key);
      }
      expanded = new Set(expanded);
    } catch (e) {
      error = e?.message ?? String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    workspace.worktreeChangeTick;
    if (projectPath) void load();
  });

  let tree = $derived(buildTree(files));

  function toggleDir(key) {
    if (expanded.has(key)) expanded.delete(key); else expanded.add(key);
    expanded = new Set(expanded);
  }

  function openFile(relPath) {
    const fileName = relPath.split('/').pop();
    workspace.openTab({
      id: `doc::${relPath}`,
      type: 'doc',
      title: fileName,
      data: { relPath, folderPath: projectPath },
    });
  }
</script>

<div class="h-full flex flex-col overflow-hidden select-none">
  {#if loading}
    <div class="flex items-center justify-center py-10 gap-2 text-muted-foreground">
      <Loader2 size={14} class="animate-spin" />
      <span class="text-sm">Scanning…</span>
    </div>
  {:else if error}
    <p class="text-sm text-destructive px-3 py-4">{error}</p>
  {:else if files.length === 0}
    <div class="flex flex-col items-center justify-center py-12 gap-2 text-muted-foreground">
      <BookOpen size={28} class="opacity-30" />
      <p class="text-sm">No markdown files found</p>
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto py-1">
      {#snippet renderNodes(nodes, depth)}
        {#each nodes as node (node.type === 'dir' ? node.key : node.path)}
          {#if node.type === 'dir'}
            <button
              type="button"
              onclick={() => toggleDir(node.key)}
              class="w-full h-7 flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors"
              style="padding-left: {0.5 + depth * 1.1}rem; padding-right: 0.5rem"
            >
              {#if expanded.has(node.key)}
                <ChevronDown size={13} class="shrink-0" />
              {:else}
                <ChevronRight size={13} class="shrink-0" />
              {/if}
              <span class="truncate font-medium">{node.name}</span>
            </button>
            {#if expanded.has(node.key)}
              {@render renderNodes(node.children, depth + 1)}
            {/if}
          {:else}
            {@const isActive = workspace.activeTabId === `doc::${node.path}`}
            <ContextMenu.Root>
              <ContextMenu.Trigger class="block w-full">
                <button
                  type="button"
                  onclick={() => openFile(node.path)}
                  class="w-full h-7 flex items-center gap-2 text-sm transition-colors
                    {isActive
                      ? 'text-foreground bg-muted/70'
                      : 'text-muted-foreground hover:text-foreground hover:bg-muted/40'}"
                  style="padding-left: {0.9 + depth * 1.1}rem; padding-right: 0.5rem"
                >
                  <FileText size={13} class="shrink-0 opacity-60" />
                  <span class="truncate">{node.name}</span>
                </button>
              </ContextMenu.Trigger>
              <ContextMenu.Content class="w-56">
                <ContextMenu.Item onclick={() => copyToClipboard(absPath(node.path))}>
                  Copy file path
                </ContextMenu.Item>
                <ContextMenu.Item onclick={() => copyToClipboard(node.path)}>
                  Copy relative path
                </ContextMenu.Item>
                <ContextMenu.Separator />
                <ContextMenu.Item onclick={() => showInExplorer(node.path)}>
                  Show in Explorer
                </ContextMenu.Item>
                <ContextMenu.Separator />
                <ContextMenu.Item
                  class="text-destructive focus:text-destructive focus:bg-destructive/10"
                  onclick={() => { deleteTarget = node.path; deleteConfirmOpen = true; }}
                >
                  Delete file
                </ContextMenu.Item>
              </ContextMenu.Content>
            </ContextMenu.Root>
          {/if}
        {/each}
      {/snippet}
      {@render renderNodes(tree, 0)}
    </div>
  {/if}
</div>

<AlertDialog.Root bind:open={deleteConfirmOpen}>
  <AlertDialog.Content class="sm:max-w-sm">
    <AlertDialog.Header>
      <AlertDialog.Title>Delete {deleteTarget?.split('/').pop()}?</AlertDialog.Title>
      <AlertDialog.Description>
        This will permanently delete <span class="font-mono text-foreground">{deleteTarget}</span> from disk. This cannot be undone.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel disabled={deleting}>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action
        class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
        disabled={deleting}
        onclick={handleDelete}
      >
        {#if deleting}<Loader2 size={13} class="mr-1.5 animate-spin inline" />{/if}
        Delete
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
