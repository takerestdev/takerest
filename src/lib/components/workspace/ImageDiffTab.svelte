<script>
  // @ts-nocheck
  let { data } = $props();

  import { gitReadBlobWorktree, gitReadBlobHead } from '$lib/commands/git.js';
  import { Loader2, AlertTriangle } from '@lucide/svelte';

  let relPath = $derived(data.relPath);
  let folderPath = $derived(data.folderPath);
  let changeKind = $derived(data.changeKind ?? 'modified'); // added | modified | deleted

  let beforeBlob = $state(null);
  let afterBlob = $state(null);
  let loading = $state(true);
  let error = $state('');

  $effect(() => {
    if (!relPath || !folderPath) return;
    loading = true;
    error = '';

    const tasks = [];
    if (changeKind !== 'added') {
      tasks.push(
        gitReadBlobHead(folderPath, relPath)
          .then(b => { beforeBlob = b; })
          .catch(() => { beforeBlob = null; })
      );
    }
    if (changeKind !== 'deleted') {
      tasks.push(
        gitReadBlobWorktree(folderPath, relPath)
          .then(b => { afterBlob = b; })
          .catch(() => { afterBlob = null; })
      );
    }

    Promise.all(tasks)
      .catch(e => { error = e?.message ?? String(e); })
      .finally(() => { loading = false; });
  });

  function dataUrl(blob) {
    return `data:${blob.mime};base64,${blob.data}`;
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Header -->
  <div class="border-b px-4 py-2.5 flex items-center gap-2 bg-background shrink-0">
    <span class="text-sm font-medium truncate">{relPath}</span>
    <span class="text-[10px] text-muted-foreground px-1.5 py-0.5 rounded border ml-auto">image</span>
  </div>

  {#if loading}
    <div class="flex flex-1 items-center justify-center gap-2 text-muted-foreground">
      <Loader2 size={16} class="animate-spin" />
    </div>
  {:else if error}
    <div class="flex flex-1 items-center justify-center gap-2 text-destructive">
      <AlertTriangle size={16} /><span>{error}</span>
    </div>
  {:else}
    <div class="flex-1 overflow-auto p-6">
      {#if changeKind === 'added'}
        <!-- Only new image -->
        <div class="flex flex-col items-center gap-2">
          <p class="text-xs text-green-600 font-medium">Added</p>
          {#if afterBlob}
            <img src={dataUrl(afterBlob)} alt="Added" class="max-w-full max-h-[70vh] object-contain rounded border shadow-sm" />
          {:else}
            <div class="flex items-center justify-center h-32 border rounded text-muted-foreground text-xs">Not available</div>
          {/if}
        </div>
      {:else if changeKind === 'deleted'}
        <!-- Only old image with red overlay -->
        <div class="flex flex-col items-center gap-2">
          <p class="text-xs text-red-500 font-medium">Deleted</p>
          {#if beforeBlob}
            <div class="relative inline-block">
              <img src={dataUrl(beforeBlob)} alt="Deleted" class="max-w-full max-h-[70vh] object-contain rounded border shadow-sm opacity-60" />
              <div class="absolute inset-0 bg-red-500/10 rounded border border-red-500/30"></div>
            </div>
          {:else}
            <div class="flex items-center justify-center h-32 border rounded text-muted-foreground text-xs">Not available</div>
          {/if}
        </div>
      {:else}
        <!-- Before / After side by side -->
        <div class="grid grid-cols-2 gap-6 max-w-5xl mx-auto">
          <div class="flex flex-col gap-2">
            <p class="text-xs text-muted-foreground font-medium text-center">Before</p>
            {#if beforeBlob}
              <img src={dataUrl(beforeBlob)} alt="Before" class="w-full object-contain rounded border shadow-sm" />
            {:else}
              <div class="flex items-center justify-center h-32 border rounded text-muted-foreground text-xs">Not available</div>
            {/if}
          </div>
          <div class="flex flex-col gap-2">
            <p class="text-xs text-muted-foreground font-medium text-center">After</p>
            {#if afterBlob}
              <img src={dataUrl(afterBlob)} alt="After" class="w-full object-contain rounded border shadow-sm" />
            {:else}
              <div class="flex items-center justify-center h-32 border rounded text-muted-foreground text-xs">Not available</div>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
