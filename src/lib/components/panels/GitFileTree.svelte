<script>
  // @ts-nocheck
  let { nodes, activeFile, onFileClick, onToggle } = $props();

  import { ChevronRight, ChevronDown, Folder, FolderOpen } from '@lucide/svelte';

  let collapsed = $state(new Set());

  function toggle(path) {
    const next = new Set(collapsed);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    collapsed = next;
  }

  const DOT_COLOR = {
    added:    'bg-green-500',
    modified: 'bg-yellow-400',
    deleted:  'bg-red-500',
    renamed:  'bg-blue-400',
  };

  const NAME_COLOR = {
    added:    'text-green-600 dark:text-green-400',
    modified: '',
    deleted:  'text-red-500 line-through opacity-60',
    renamed:  'text-blue-500',
  };

  function conflictColor(file) {
    return file.conflicted ? 'bg-red-500' : (DOT_COLOR[statusKind(file)] ?? 'bg-muted-foreground/40');
  }

  function statusKind(file) {
    return file.indexStatus?.type ?? file.worktreeStatus?.type ?? 'modified';
  }

  function isStaged(file) {
    return file.indexStatus != null;
  }

  // x-position of the indent guide line for ancestor level i
  // Aligns with the center of the folder icon at depth i (icon starts at 10 + i*14 px, ~13px wide)
  function guideX(i) { return 16 + i * 14; }
</script>

{#snippet renderNodes(nodes, depth)}
  {#each nodes as node (node.path)}
    {#if node.type === 'dir'}
      {@const open = !collapsed.has(node.path)}

      <!-- Folder row -->
      <div class="relative">
        {#each Array.from({ length: depth }) as _, i}
          <span
            class="absolute top-0 bottom-0 w-px bg-border/50 pointer-events-none"
            style="left: {guideX(i)}px;"
          ></span>
        {/each}
        <button
          type="button"
          class="w-full flex items-center gap-1.5 py-1 hover:bg-muted/50 text-muted-foreground hover:text-foreground transition-colors text-[13px] select-none"
          style="padding-left: {10 + depth * 14}px; padding-right: 16px;"
          onclick={() => toggle(node.path)}
        >
          {#if open}
            <FolderOpen size={13} class="shrink-0 opacity-60" />
          {:else}
            <Folder size={13} class="shrink-0 opacity-60" />
          {/if}
          <span class="shrink-0 font-medium">{node.name}</span>
          {#if node.count > 0}
            <span class="text-[11px] opacity-50 shrink-0 ml-1">{node.count}</span>
          {/if}
          <span class="flex-1"></span>
          {#if open}
            <ChevronDown size={11} class="shrink-0 opacity-40" />
          {:else}
            <ChevronRight size={11} class="shrink-0 opacity-40" />
          {/if}
        </button>
      </div>

      {#if open}
        {@render renderNodes(node.children, depth + 1)}
      {/if}

    {:else}
      {@const file = node.file}
      {@const kind = statusKind(file)}
      {@const staged = isStaged(file)}
      {@const isActive = activeFile === file.path}

      <!-- File row -->
      <div
        class="relative flex items-center gap-1.5 py-1 group transition-colors
          {isActive ? 'bg-muted' : 'hover:bg-muted/50'}"
        style="padding-left: {10 + depth * 14}px; padding-right: 16px;"
        role="none"
      >
        {#each Array.from({ length: depth }) as _, i}
          <span
            class="absolute top-0 bottom-0 w-px bg-border/50 pointer-events-none"
            style="left: {guideX(i)}px;"
          ></span>
        {/each}

        <!-- Filename -->
        <button
          type="button"
          class="flex-1 text-left text-[13px] truncate min-w-0 transition-colors
            {file.conflicted ? 'text-red-500 dark:text-red-400' : isActive ? 'text-foreground' : 'text-muted-foreground group-hover:text-foreground'}
            {file.conflicted ? '' : NAME_COLOR[kind] ?? ''}"
          onclick={() => onFileClick(file)}
        >
          {node.name}{#if file.conflicted}<span class="ml-1 text-[10px] font-bold opacity-70">!!</span>{/if}
        </button>

        <!-- Status dot -->
        <span
          class="shrink-0 w-2 h-2 rounded-full {conflictColor(file)}"
          title={file.conflicted ? 'merge conflict' : kind}
        ></span>

        <!-- Checkbox (stage toggle) -->
        <button
          type="button"
          aria-label="{staged ? 'Unstage' : 'Stage'} {file.path}"
          onclick={(e) => { e.stopPropagation(); onToggle(file, !staged); }}
          class="shrink-0 w-3.5 h-3.5 ml-1 rounded-[3px] border flex items-center justify-center transition-all
            {staged
              ? 'bg-primary border-primary text-primary-foreground'
              : 'border-muted-foreground/30 hover:border-primary/70 group-hover:border-muted-foreground/60'}"
        >
          {#if staged}
            <svg viewBox="0 0 10 10" class="w-2.5 h-2.5" fill="none" stroke="currentColor" stroke-width="2.2">
              <polyline points="1.5,5 4,8 8.5,2" />
            </svg>
          {/if}
        </button>
      </div>
    {/if}
  {/each}
{/snippet}

<div class="w-full py-0.5">
  {@render renderNodes(nodes, 0)}
</div>
