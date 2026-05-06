<script>
// @ts-nocheck

    /** @type {{ data: import('./$types').LayoutData, children: import('svelte').Snippet }} */
    let { data, children } = $props();
    import { Minus, Square, X } from "@lucide/svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { platform } from "@tauri-apps/plugin-os";
    import Logo from "$lib/components/logo.svelte";

    let isWindows = $state(false);
    let appWindow;

    import { onMount } from "svelte";

    onMount(async () => {
        appWindow = getCurrentWindow();
        isWindows = (await platform()) === "windows";
    });

    async function isWindowsPlatform(params) {
        return (await platform()) === "windows";
    }
    
    import { useSearchParams, createSearchParamsSchema } from "runed/kit";

    const schema = createSearchParamsSchema({
        path: { type: "string", default: "" },
    });

    const params = useSearchParams(schema, {
        pushHistory: false,
    });

    let folderPath = $state(params.path);
    let folderName = $state(folderPath.split(/[\\/]/).filter(Boolean).pop() ?? "Project");
</script>

<div class="h-screen w-full flex flex-col text-foreground bg-background">
  <!-- ── HEADER ── -->
  <header
    data-tauri-drag-region
    class="grid grid-cols-3 w-full items-center border-b shrink-0"
  >
    <!-- Left: empty spacer -->
    <div></div>

    <!-- Center: logo + name -->
    <div
      data-tauri-drag-region
      class="flex items-center px-4 py-2.5 justify-center gap-2.5"
    >
      <Logo />
      <span class="font-semibold text-sm tracking-tight">takerest ~ {folderName}</span>
    </div>

    <!-- Right: Windows controls -->
    <div data-tauri-drag-region class="flex items-center justify-end">
      {#if isWindows}
        <button
          aria-label="Minimize window"
          type="button"
          class="w-11 h-9 flex items-center justify-center hover:bg-muted transition-colors"
          onclick={() => appWindow.minimize()}
          title="Minimize"
        >
          <Minus size={14} />
        </button>
        <button
          aria-label="Maximize window"
          type="button"
          class="w-11 h-9 flex items-center justify-center hover:bg-muted transition-colors"
          onclick={() => appWindow.toggleMaximize()}
          title="Maximize"
        >
          <Square size={14} />
        </button>
        <button
          aria-label="Close window"
          type="button"
          class="w-11 h-9 flex items-center justify-center hover:bg-red-500 hover:text-white transition-colors"
          onclick={() => appWindow.close()}
          title="Close"
        >
          <X size={14} />
        </button>
      {/if}
    </div>
  </header>
{@render children()}
</div>