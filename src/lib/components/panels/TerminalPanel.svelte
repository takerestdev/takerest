<script>
  // @ts-nocheck
  import { onMount } from 'svelte';
  import { workspace } from '$lib/stores/workspace.svelte.js';
  import { terminalListShells } from '$lib/commands/terminal.js';
  import { Terminal, Plus, ChevronDown } from '@lucide/svelte';

  let terminalTabs = $derived(workspace.tabs.filter(t => t.type === 'terminal'));

  let shells = $state([]);
  let dropdownOpen = $state(false);

  onMount(async () => {
    try {
      shells = await terminalListShells();
    } catch {}
  });

  function openTerminal(shell = null) {
    const sessionId = crypto.randomUUID();
    const n = terminalTabs.length + 1;
    const title = shell ? `${shell.name} ${n}` : `Terminal ${n}`;
    workspace.openTab({
      id: `terminal-${sessionId}`,
      type: 'terminal',
      title,
      data: {
        sessionId,
        cwd: workspace.folderPath,
        shell: shell?.program ?? null,
        shellArgs: shell?.args ?? null,
      },
    });
    dropdownOpen = false;
  }
</script>

<div class="flex flex-col h-full">

  <!-- New Terminal button row -->
  <div class="px-2 py-2 shrink-0 relative">
    <div class="flex items-stretch gap-px">

      <!-- Main "New Terminal" button — opens with first/default shell -->
      <button
        type="button"
        onclick={() => openTerminal(shells[0] ?? null)}
        class="flex-1 flex items-center gap-1.5 px-2 py-1.5 text-xs rounded-l-md border border-dashed border-border/60
          text-muted-foreground hover:text-foreground hover:border-border hover:bg-muted/50 transition-colors"
      >
        <Plus size={12} />
        New Terminal
      </button>

      <!-- Chevron — opens shell picker -->
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

    <!-- Shell picker dropdown -->
    {#if dropdownOpen}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="fixed inset-0 z-40" onclick={() => (dropdownOpen = false)}></div>
      <div class="absolute left-2 right-2 top-full mt-1 z-50 rounded-md border bg-popover shadow-md overflow-hidden">
        {#if shells.length === 0}
          <p class="px-3 py-2 text-xs text-muted-foreground">No shells detected</p>
        {:else}
          {#each shells as shell (shell.program)}
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

  <!-- Open terminals list -->
  <div class="flex-1 overflow-y-auto">
    {#each terminalTabs as tab (tab.id)}
      <button
        type="button"
        onclick={() => { workspace.activeTabId = tab.id; }}
        class="w-full flex items-center gap-2 px-3 py-2 text-left text-xs transition-colors
          {workspace.activeTabId === tab.id
            ? 'bg-muted text-foreground'
            : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
      >
        <Terminal size={12} class="shrink-0" />
        <span class="flex-1 truncate">{tab.title}</span>
      </button>
    {/each}

    {#if terminalTabs.length === 0}
      <div class="flex flex-col items-center gap-2 py-10 px-3 text-center text-muted-foreground">
        <Terminal size={20} class="opacity-30" />
        <p class="text-xs">No open terminals</p>
      </div>
    {/if}
  </div>

</div>
