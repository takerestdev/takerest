<script>
  // @ts-nocheck

  /** @type {{ data: import('./$types').LayoutData, children: import('svelte').Snippet }} */
  let { data, children } = $props();
  import {
    Minus,
    Square,
    X,
    Globe,
    Database,
    KeySquare,
    Box,
    GitBranch,
    Container,
    Braces,
  } from "@lucide/svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { platform } from "@tauri-apps/plugin-os";
  import Logo from "$lib/components/logo.svelte";
  import Themetoggle from "$lib/components/themetoggle.svelte";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import { page } from "$app/stores";

  let isWindows = $state(false);
  let appWindow;

  // Derive active tab from current URL pathname
  let activeTab = $derived.by(() => {
    const pathname = $page.url.pathname;
    const match = pathname.match(/^\/app\/(\w+)/);
    return match ? match[1] : null;
  });

  import { onMount } from "svelte";

  onMount(async () => {
    appWindow = getCurrentWindow();
    isWindows = (await platform()) === "windows";
  });

  import { useSearchParams, createSearchParamsSchema } from "runed/kit";

  const schema = createSearchParamsSchema({
    path: { type: "string", default: "" },
  });

  const params = useSearchParams(schema, {
    pushHistory: false,
  });

  let folderPath = $derived(params.path);
  let encodedPath = $derived(encodeURIComponent(folderPath));
  let folderName = $derived.by(
    () => folderPath.split(/[\\/]/).filter(Boolean).pop() ?? "Project",
  );

  const tabs = [
    { id: "api", icon: Globe, label: "API" },
    { id: "db", icon: Database, label: "Database" },
    { id: "kv", icon: Braces, label: "Cache" },
    { id: "s3", icon: Box, label: "Storage" },
    { id: "git", icon: GitBranch, label: "Git" },
    { id: "docker", icon: Container, label: "Docker" },
    { id: "env", icon: KeySquare, label: "Env" },
  ];
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
      <span class="font-semibold text-sm tracking-tight">{folderName}</span>
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

  <!-- ── BODY: Sidebar + Content ── -->
  <div class="flex flex-1 overflow-hidden">
    <!-- Vertical icon sidebar -->
    <aside
      class="w-12 shrink-0 border-r flex flex-col items-center bg-background/50"
    >
      <!-- TakeRest logo at top — links to /app -->
      <Tooltip.Provider>
        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({ props })}
              <a
                {...props}
                href="/app?path={encodedPath}"
                aria-label="Home"
                class="w-full flex justify-center py-3 mb-1 transition-colors {activeTab ===
                null
                  ? 'text-foreground'
                  : 'text-muted-foreground hover:text-foreground/80'}"
              >
                <div class="size-7 flex items-center justify-center">
                  <Logo active={activeTab === null} />
                </div>
              </a>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Content side="right">Home</Tooltip.Content>
        </Tooltip.Root>

        <!-- Tool tabs -->
        <nav class="flex flex-col items-center gap-0.5 flex-1 w-full">
          {#each tabs as tab}
            <Tooltip.Root>
              <Tooltip.Trigger>
                {#snippet child({ props })}
                  <a
                    {...props}
                    href="/app/{tab.id}?path={encodedPath}"
                    aria-label={tab.label}
                    class="w-full h-10 flex items-center justify-center relative transition-colors duration-150 no-underline {activeTab ===
                    tab.id
                      ? 'text-foreground'
                      : 'text-muted-foreground hover:text-foreground/80'}"
                  >
                    <!-- Active indicator bar -->
                    {#if activeTab === tab.id}
                      <span
                        class="absolute left-0 top-1.5 bottom-1.5 w-0.5 bg-primary rounded-r"
                      ></span>
                    {/if}
                    <tab.icon
                      size={20}
                      strokeWidth={activeTab === tab.id ? 2.2 : 1.5}
                    />
                  </a>
                {/snippet}
              </Tooltip.Trigger>
              <Tooltip.Content side="right">{tab.label}</Tooltip.Content>
            </Tooltip.Root>
          {/each}
        </nav>

        <!-- Theme toggle at bottom -->
        <div class="pb-3 pt-1">
          <Themetoggle ghost={true} />
        </div>
      </Tooltip.Provider>
    </aside>

    <!-- Main content area -->
    <main class="flex-1 overflow-hidden">
      {@render children()}
    </main>
  </div>
</div>
