<script>
  // @ts-nocheck
  import { useSearchParams, createSearchParamsSchema } from "runed/kit";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import * as Resizable from "$lib/components/ui/resizable/index.js";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import {
    FileLock2,
    Plus,
    FileKey,
    ShieldCheck,
    ShieldOff,
    Loader2,
  } from "@lucide/svelte";
  import { listEnvFiles, createEnvFile } from "$lib/commands/env";

  /** @type {{ children: import('svelte').Snippet }} */
  let { children } = $props();

  const schema = createSearchParamsSchema({
    path: { type: "string", default: "" },
  });
  const params = useSearchParams(schema, { pushHistory: false });
  let folderPath = $derived(params.path);

  // Current active file from URL
  let currentEnvFile = $derived(
    $page.params.envfile ?? null
  );

  // Env file list
  /** @type {import('$lib/commands/env').EnvFile[]} */
  let envFiles = $state([]);
  let listLoading = $state(false);

  // Dialog state
  let dialogOpen = $state(false);
  let newFileSuffix = $state("");
  let addToGitignore = $state(true);
  let creating = $state(false);
  let createError = $state("");

  // Derived: full filename preview
  let newFileName = $derived(
    newFileSuffix.trim() ? `.env.${newFileSuffix.trim()}` : ".env"
  );

  // Load env files whenever folderPath changes
  $effect(() => {
    // This line is the key — forces re-run on navigation between list ↔ file view
    const trigger = `${folderPath}|${$page.params.envfile ?? 'list'}`;

    if (!folderPath) {
      envFiles = [];
      return;
    }

    void (async () => {
      listLoading = true;
      try {
        envFiles = await listEnvFiles(folderPath);
      } catch (e) {
        console.error("Failed to list env files", e);
      } finally {
        listLoading = false;
      }
    })();
  });

  async function handleCreate() {
    if (!folderPath) return;
    createError = "";
    creating = true;
    try {
      const relPath = newFileSuffix.trim()
        ? `.env.${newFileSuffix.trim()}`
        : ".env";

      const created = await createEnvFile(folderPath, {
        relPath,
        content: "",
        addToGitignore,
      });

      // Refresh list
      envFiles = await listEnvFiles(folderPath);

      dialogOpen = false;
      newFileSuffix = "";
      addToGitignore = true;

      // Navigate to the new file
      goto(`/app/env/${encodeURIComponent(created.relPath)}?path=${encodeURIComponent(folderPath)}`);
    } catch (e) {
      createError = e?.message ?? String(e);
    } finally {
      creating = false;
    }
  }

  function isActive(relPath) {
    // currentEnvFile is the [...envfile] rest param joined with /
    return currentEnvFile === relPath;
  }
</script>

<div class="h-full w-full overflow-hidden">
  <Resizable.PaneGroup direction="horizontal" class="h-full">
    <!-- ── Left: File List ──────────────────────────────────── -->
    <Resizable.Pane defaultSize={28} minSize={18} maxSize={45}>
      <div class="h-full flex flex-col border-r">
        <!-- Header -->
        <div class="flex items-center justify-between px-4 py-3 border-b shrink-0">
          <div class="flex items-center gap-2">
            <FileKey class="w-4 h-4 text-muted-foreground" />
            <span class="text-sm font-semibold">Env Files</span>
            {#if envFiles.length > 0}
              <Badge variant="secondary" class="text-xs h-4 px-1.5">
                {envFiles.length}
              </Badge>
            {/if}
          </div>

          <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7"
            onclick={() => (dialogOpen = true)}
            title="New env file"
          >
            <Plus class="w-4 h-4" />
          </Button>
        </div>

        <!-- File List -->
        <ScrollArea class="flex-1">
            <div class="p-2 space-y-0.5">
                {#if listLoading}
                <div class="flex items-center justify-center py-8 text-muted-foreground">
                    <Loader2 class="w-4 h-4 animate-spin mr-2" />
                    <span class="text-xs">Scanning...</span>
                </div>
                {:else if envFiles.length === 0}
                <div class="flex flex-col items-center justify-center py-10 gap-3 text-muted-foreground px-4">
                    <FileLock2 class="w-8 h-8 opacity-40" />
                    <p class="text-xs text-center">No .env files found.<br />Create one to get started.</p>
                    <Button
                    variant="outline"
                    size="sm"
                    class="text-xs h-7"
                    onclick={() => (dialogOpen = true)}
                    >
                    <Plus class="w-3 h-3 mr-1" />
                    New file
                    </Button>
                </div>
                {:else}
                {#each envFiles as file (file.relPath)}
                    <a
                    class="w-full text-left rounded-md transition-colors"
                    href={`/app/env/${encodeURIComponent(file.relPath)}?path=${encodeURIComponent(folderPath)}`}
                    >
                    <div class="flex items-center justify-between gap-2 p-1.5 rounded-md transition-colors group-hover:bg-muted
                    {isActive(file.relPath)
                            ? 'bg-primary/90 '
                            : ''}
                    ">
                        <span class="{isActive(file.relPath) ? 'text-primary-foreground' : 'text-foreground'}">
                        {file.name}
                        </span>

                        {#if file.inGitignore}
                        <ShieldCheck
                            class=" pr-2 {isActive(file.relPath)
                            ? 'text-primary-foreground/70'
                            : 'text-green-600'}"
                            title="In .gitignore"
                        />
                        {:else}
                        <ShieldOff
                            class="w-3.5 h-3.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity
                                {isActive(file.relPath)
                                    ? 'text-primary-foreground/50'
                                    : 'text-muted-foreground'}"
                            title="Not in .gitignore"
                        />
                        {/if}
                    </div>

                    {#if file.relPath !== file.name}
                        <p class="text-[10px] mt-0.5 truncate
                                {isActive(file.relPath)
                                    ? 'text-primary-foreground/60'
                                    : 'text-muted-foreground'}">
                        {file.relPath}
                        </p>
                    {/if}
                    </a>
                {/each}
                {/if}
            </div>
            </ScrollArea>

        <!-- Footer hint -->
        <div class="px-4 py-2 border-t shrink-0">
          <p class="text-[10px] text-muted-foreground">
            <ShieldCheck class="w-3 h-3 inline mr-1 text-green-600" />= in .gitignore
          </p>
        </div>
      </div>
    </Resizable.Pane>

    <Resizable.Handle withHandle />

    <!-- ── Right: Page content (page.svelte or [envfile]/page.svelte) ── -->
    <Resizable.Pane defaultSize={72}>
      <div class="h-full overflow-auto">
        {@render children()}
      </div>
    </Resizable.Pane>
  </Resizable.PaneGroup>
</div>

<!-- ── Create Dialog ─────────────────────────────────────────────── -->
<Dialog.Root bind:open={dialogOpen}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>New Environment File</Dialog.Title>
      <Dialog.Description>
        Create a new <span class="font-mono text-foreground">.env</span> file in your project.
      </Dialog.Description>
    </Dialog.Header>

    <div class="space-y-5 py-2">
      <!-- Filename builder -->
      <div class="space-y-2">
        <Label>Filename</Label>
        <div class="flex items-center gap-0 rounded-md border focus-within:ring-2 focus-within:ring-ring overflow-hidden">
          <span class="px-3 py-2 text-sm font-mono bg-muted border-r text-muted-foreground select-none">
            .env.
          </span>
          <Input
            class="border-0 rounded-none shadow-none focus-visible:ring-0 font-mono text-sm"
            placeholder="local"
            bind:value={newFileSuffix}
            onkeydown={(e) => e.key === "Enter" && handleCreate()}
            autofocus
          />
        </div>
        <p class="text-xs text-muted-foreground">
          Will create: <span class="font-mono text-foreground">{newFileName}</span>
        </p>
      </div>

      <!-- Gitignore toggle -->
      <div class="flex items-start gap-3">
        <Checkbox
          id="gitignore-check"
          bind:checked={addToGitignore}
          class="mt-0.5"
        />
        <div class="space-y-0.5">
          <Label for="gitignore-check" class="cursor-pointer">Add to .gitignore</Label>
          <p class="text-xs text-muted-foreground">
            Recommended — keeps secrets out of version control.
          </p>
        </div>
      </div>

      {#if createError}
        <p class="text-xs text-destructive bg-destructive/10 border border-destructive/20 rounded-md px-3 py-2">
          {createError}
        </p>
      {/if}
    </div>

    <Dialog.Footer class="gap-2">
      <Button
        variant="outline"
        onclick={() => { dialogOpen = false; createError = ""; }}
        disabled={creating}
      >
        Cancel
      </Button>
      <Button onclick={handleCreate} disabled={creating || !folderPath}>
        {#if creating}
          <Loader2 class="w-3.5 h-3.5 mr-1.5 animate-spin" />
          Creating...
        {:else}
          <Plus class="w-3.5 h-3.5 mr-1.5" />
          Create
        {/if}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>