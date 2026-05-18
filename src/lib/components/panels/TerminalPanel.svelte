<script>
  // @ts-nocheck
  import { onMount } from 'svelte';
  import { workspace } from '$lib/stores/workspace.svelte.js';
  import { terminalListShells } from '$lib/commands/terminal.js';
  import { Terminal, Plus, ChevronDown, Trash2 } from '@lucide/svelte';
  import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
  import { Input } from '$lib/components/ui/input/index.js';

  let terminalTabs = $derived(workspace.tabs.filter(t => t.type === 'terminal'));

  let shells = $state([]);
  let dropdownOpen = $state(false);

  // Kill confirmation
  let killTarget = $state(null);
  let killOpen = $state(false);

  // Inline rename
  let renamingId = $state(null);
  let renameValue = $state('');
  let renameInputRef = $state(null);

  $effect(() => {
    if (renamingId && renameInputRef) {
      renameInputRef.focus();
      renameInputRef.select();
    }
  });

  onMount(async () => {
    try { shells = await terminalListShells(); } catch {}
  });

  function openTerminal(shell = null) {
    const sessionId = crypto.randomUUID();
    const n = terminalTabs.length + 1;
    const title = shell ? `${shell.name} ${n}` : `Terminal ${n}`;
    workspace.openTab({
      id: `terminal-${sessionId}`,
      type: 'terminal',
      title,
      data: { sessionId, cwd: workspace.folderPath, shell: shell?.program ?? null, shellArgs: shell?.args ?? null },
    });
    dropdownOpen = false;
  }

  function openKill(tab) {
    killTarget = tab;
    killOpen = true;
  }

  function confirmKill() {
    if (killTarget) workspace.closeTab(killTarget.id);
    killTarget = null;
    killOpen = false;
  }

  function startRename(tab) {
    renamingId = tab.id;
    renameValue = tab.title;
  }

  function saveRename() {
    if (renamingId && renameValue.trim()) {
      workspace.renameTab(renamingId, renameValue.trim());
    }
    renamingId = null;
    renameValue = '';
  }

  function handleRenameKey(e) {
    if (e.key === 'Enter') { e.preventDefault(); saveRename(); }
    if (e.key === 'Escape') { renamingId = null; renameValue = ''; }
  }


</script>

<div class="flex flex-col h-full">

  <!-- New Terminal split button -->
  <div class="px-2 py-2 shrink-0 relative">
    <div class="flex items-stretch gap-px">
      <button
        type="button"
        onclick={() => openTerminal(shells[0] ?? null)}
        class="flex-1 flex items-center gap-1.5 px-2 py-1.5 text-xs rounded-l-md border border-dashed border-border/60
          text-muted-foreground hover:text-foreground hover:border-border hover:bg-muted/50 transition-colors"
      >
        <Plus size={12} />
        New Terminal
      </button>
      <button
        type="button"
        aria-label="Pick shell"
        onclick={() => (dropdownOpen = !dropdownOpen)}
        class="flex items-center justify-center px-1.5 rounded-r-md border border-dashed border-border/60
          text-muted-foreground hover:text-foreground hover:border-border hover:bg-muted/50 transition-colors
          {dropdownOpen ? 'bg-muted/50 text-foreground border-border' : ''}"
      >
        <ChevronDown size={12} class="transition-transform {dropdownOpen ? 'rotate-180' : ''}" />
      </button>
    </div>

    {#if dropdownOpen}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="fixed inset-0 z-40" onclick={() => (dropdownOpen = false)}></div>
      <div class="absolute left-2 right-2 top-full mt-1 z-50 rounded-md border bg-popover shadow-md overflow-hidden">
        {#if shells.length === 0}
          <p class="px-3 py-2 text-xs text-muted-foreground">No shells detected</p>
        {:else}
          {#each shells as shell (`${shell.program}-${(shell.args ?? []).join(',')}-${shell.name}`)}
            <button
              type="button"
              onclick={() => openTerminal(shell)}
              class="w-full flex items-center gap-2 px-3 py-2 text-xs text-left
                text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
            >
              <Terminal size={11} class="shrink-0" />
              {shell.name}
            </button>
          {/each}
        {/if}
      </div>
    {/if}
  </div>

  <!-- Terminal list -->
  <div class="flex-1 overflow-y-auto">
    {#each terminalTabs as tab (tab.id)}
      {@const isActive = workspace.activeTabId === tab.id}
      {@const isRenaming = renamingId === tab.id}

      <div
        role="none"
        class="group flex items-center gap-1 pr-1 transition-colors
          {isActive ? 'bg-muted' : 'hover:bg-muted/50'}"
      >
        {#if isRenaming}
          <span class="pl-3 text-muted-foreground flex items-center shrink-0 py-2">
            <Terminal size={12} />
          </span>
          <Input
            bind:ref={renameInputRef}
            type="text"
            bind:value={renameValue}
            onblur={saveRename}
            onkeydown={handleRenameKey}
            class="flex-1 h-6 text-xs px-1.5 py-0 my-1.5"
          />
        {:else}
          <button
            type="button"
            onclick={() => { workspace.activeTabId = tab.id; }}
            ondblclick={() => startRename(tab)}
            class="flex-1 flex items-center gap-2 pl-3 py-2 text-xs text-left min-w-0 transition-colors
              {isActive ? 'text-foreground' : 'text-muted-foreground group-hover:text-foreground'}"
          >
            <Terminal size={12} class="shrink-0" />
            <span class="truncate">{tab.title}</span>
          </button>
        {/if}

        <button
          type="button"
          aria-label="Kill terminal"
          onclick={() => openKill(tab)}
          class="shrink-0 p-1 rounded opacity-0 group-hover:opacity-100 transition-opacity
            text-muted-foreground hover:text-destructive"
        >
          <Trash2 size={11} />
        </button>
      </div>
    {/each}

    {#if terminalTabs.length === 0}
      <div class="flex flex-col items-center gap-2 py-10 px-3 text-center text-muted-foreground">
        <Terminal size={20} class="opacity-30" />
        <p class="text-xs">No open terminals</p>
      </div>
    {/if}
  </div>

</div>

<!-- Kill confirmation dialog -->
<AlertDialog.Root bind:open={killOpen}>
  <AlertDialog.Content class="sm:max-w-sm">
    <AlertDialog.Header>
      <AlertDialog.Title>Kill terminal?</AlertDialog.Title>
      <AlertDialog.Description>
        This will kill the <span class="font-mono text-foreground">{killTarget?.title}</span> process and close the tab.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action
        class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
        onclick={confirmKill}
      >
        Kill Terminal
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
