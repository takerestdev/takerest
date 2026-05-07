<script>
    // @ts-nocheck
    import { page } from "$app/stores";
    import { goto } from "$app/navigation";

    import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import {
        Loader2,
        Save,
        Trash2,
        Plus,
        ShieldCheck,
        ShieldOff,
        FileKey,
        X,
    } from "@lucide/svelte";

    import {
        readEnvFile,
        writeEnvFile,
        deleteEnvFile,
        toggleEnvGitignore,
        listEnvFiles,
    } from "$lib/commands/env";

    let currentRelPath = $derived($page.params.envfile ?? "");
    let folderPath = $derived($page.url.searchParams.get("path") ?? "");

    let variables = $state([]); // [{ key: string, value: string }]
    let originalVariables = $state([]); // for dirty check
    let loading = $state(true);
    let saving = $state(false);
    let error = $state("");

    /** @type {import('$lib/commands/env').EnvFile | null} */
    let fileMeta = $state(null);

    // ── Parse .env content into key/value array ─────────────────────
    function parseEnv(content) {
        const lines = content.split("\n");
        const vars = [];
        for (const line of lines) {
            const trimmed = line.trim();
            if (!trimmed || trimmed.startsWith("#")) continue; // skip comments/empty for now

            const eqIndex = trimmed.indexOf("=");
            if (eqIndex === -1) continue;

            const key = trimmed.slice(0, eqIndex).trim();
            const value = trimmed.slice(eqIndex + 1).trim();
            if (key) vars.push({ key, value });
        }
        return vars;
    }

    // ── Serialize back to .env string ───────────────────────────────
    function serializeEnv(vars) {
        return vars.map((v) => `${v.key}=${v.value}`).join("\n") + "\n";
    }

    async function loadFile() {
        if (!folderPath || !currentRelPath) return;
        loading = true;
        error = "";

        try {
            const result = await readEnvFile(folderPath, currentRelPath);
            variables = parseEnv(result.content);
            originalVariables = JSON.parse(JSON.stringify(variables));

            // Refresh metadata (especially gitignore status)
            const allFiles = await listEnvFiles(folderPath);
            fileMeta =
                allFiles.find((f) => f.relPath === currentRelPath) ?? null;
        } catch (e) {
            error = e?.message ?? String(e);
        } finally {
            loading = false;
        }
    }

    $effect(() => {
        if (currentRelPath && folderPath) void loadFile();
    });

    // ── Save ───────────────────────────────────────────────────────
    async function handleSave() {
        if (!folderPath || !currentRelPath) return;
        saving = true;
        error = "";

        try {
            const newContent = serializeEnv(variables);
            await writeEnvFile(folderPath, currentRelPath, newContent);
            originalVariables = JSON.parse(JSON.stringify(variables));
        } catch (e) {
            error = e?.message ?? String(e);
        } finally {
            saving = false;
        }
    }

    // ── Add new variable ───────────────────────────────────────────
    function addVariable() {
        variables = [...variables, { key: "", value: "" }];
    }

    // ── Delete variable ────────────────────────────────────────────
    function deleteVariable(index) {
        variables = variables.filter((_, i) => i !== index);
    }

    // ── Toggle gitignore ───────────────────────────────────────────
    async function handleToggleGitignore() {
        if (!fileMeta) return;
        try {
            await toggleEnvGitignore(folderPath, fileMeta);

            // Refresh metadata
            const allFiles = await listEnvFiles(folderPath);
            fileMeta =
                allFiles.find((f) => f.relPath === currentRelPath) ?? fileMeta;

            // ← IMPORTANT: Refresh sidebar in layout
            await goto(
                `/app/env/${currentRelPath}?path=${encodeURIComponent(folderPath)}`,
                { replaceState: true },
            );
        } catch (e) {
            error = e?.message ?? String(e);
        }
    }

    // ── Delete file ────────────────────────────────────────────────
    async function handleDelete() {
        if (!folderPath || !currentRelPath) return;
        try {
            await deleteEnvFile(folderPath, currentRelPath);
            // This navigation will now properly refresh the sidebar
            goto(`/app/env?path=${encodeURIComponent(folderPath)}`);
        } catch (e) {
            error = e?.message ?? String(e);
        }
    }
</script>

<div class="h-full flex flex-col">
    <!-- Header -->
    <div
        class="border-b px-6 py-4 flex items-center justify-between bg-background shrink-0"
    >
        <div class="flex items-center gap-3">
            <FileKey class="w-5 h-5 text-muted-foreground" />
            <div class="font-mono text-base font-medium truncate max-w-md">
                {currentRelPath}
            </div>

            {#if fileMeta}
                <Badge
                    variant={fileMeta.inGitignore ? "default" : "secondary"}
                    class="flex items-center gap-1"
                >
                    {#if fileMeta.inGitignore}
                        <ShieldCheck class="w-3 h-3" /> Ignored
                    {:else}
                        <ShieldOff class="w-3 h-3" /> Not ignored
                    {/if}
                </Badge>
            {/if}
        </div>

        <div class="flex items-center gap-2">
            <!-- Gitignore toggle -->
            <Button
                variant="outline"
                size="sm"
                onclick={handleToggleGitignore}
                disabled={!fileMeta}
            >
                {#if fileMeta?.inGitignore}
                    Remove from .gitignore
                {:else}
                    Add to .gitignore
                {/if}
            </Button>

            <!-- Delete file -->
            <AlertDialog.Root>
                <AlertDialog.Trigger asChild>
                    <Button variant="destructive" size="sm">
                        <Trash2 class="w-4 h-4" />
                    </Button>
                </AlertDialog.Trigger>
                <AlertDialog.Content>
                    <AlertDialog.Header>
                        <AlertDialog.Title
                            >Delete {currentRelPath}?</AlertDialog.Title
                        >
                        <AlertDialog.Description>
                            This action cannot be undone.
                        </AlertDialog.Description>
                    </AlertDialog.Header>
                    <AlertDialog.Footer>
                        <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
                        <AlertDialog.Action
                            onclick={handleDelete}
                            class="bg-destructive"
                        >
                            Delete file
                        </AlertDialog.Action>
                    </AlertDialog.Footer>
                </AlertDialog.Content>
            </AlertDialog.Root>

            <!-- Save -->
            <Button
                onclick={handleSave}
                disabled={saving ||
                    JSON.stringify(variables) ===
                        JSON.stringify(originalVariables)}
            >
                {#if saving}
                    <Loader2 class="w-4 h-4 mr-2 animate-spin" />
                    Saving...
                {:else}
                    <Save class="w-4 h-4 mr-2" />
                    Save
                {/if}
            </Button>
        </div>
    </div>

    <!-- Editor Table -->
    <div class="flex-1 p-6 overflow-auto">
        {#if loading}
            <div
                class="flex h-full items-center justify-center text-muted-foreground"
            >
                <Loader2 class="w-8 h-8 animate-spin" />
            </div>
        {:else if error}
            <div class="text-destructive text-center py-8">{error}</div>
        {:else}
            <div class="max-w-3xl mx-auto">
                <div
                    class="grid grid-cols-[1fr_2fr_auto] gap-2 items-center text-sm font-medium text-muted-foreground mb-2 px-2"
                >
                    <div>KEY</div>
                    <div>VALUE</div>
                    <div class="w-8"></div>
                </div>

                {#each variables as variable, index (index)}
                    <div
                        class="grid grid-cols-[1fr_2fr_auto] gap-2 items-center mb-2 group"
                    >
                        <Input
                            bind:value={variable.key}
                            placeholder="KEY_NAME"
                            class="font-mono"
                        />
                        <Input
                            bind:value={variable.value}
                            placeholder="value here"
                            class="font-mono"
                        />
                        <Button
                            variant="ghost"
                            size="icon"
                            class="text-destructive hover:bg-primary-foreground"
                            onclick={() => deleteVariable(index)}
                        >
                            <X class="w-4 h-4" />
                        </Button>
                    </div>
                {/each}

                <!-- Add button -->
                <Button
                    variant="outline"
                    onclick={addVariable}
                    class="mt-4 w-full"
                >
                    <Plus class="w-4 h-4 mr-2" />
                    Add variable
                </Button>
            </div>
        {/if}
    </div>

    <!-- Unsaved indicator -->
    {#if JSON.stringify(variables) !== JSON.stringify(originalVariables)}
        <div
            class="px-6 py-3 border-t text-amber-600 text-xs flex items-center gap-2 bg-amber-50 dark:bg-amber-950"
        >
            <span class="font-medium">● Unsaved changes</span>
        </div>
    {/if}
</div>
