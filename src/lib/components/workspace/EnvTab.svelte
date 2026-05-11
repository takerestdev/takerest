<script>
    // @ts-nocheck
    let { data, tabId } = $props();

    import { workspace } from '$lib/stores/workspace.svelte.js';
    import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
    import { Button } from '$lib/components/ui/button/index.js';
    import { Input } from '$lib/components/ui/input/index.js';
    import {
        Loader2, Save, Trash2, Plus, ShieldCheck, ShieldOff, FileKey, X,
        Copy, Check,
    } from '@lucide/svelte';
    import {
        readEnvFile, writeEnvFile, deleteEnvFile,
        toggleEnvGitignore, listEnvFiles,
    } from '$lib/commands/env';

    let relPath = $derived(data.relPath);
    let folderPath = $derived(data.folderPath);

    let variables = $state([]);
    let originalVariables = $state([]);
    let loading = $state(true);
    let saving = $state(false);
    let error = $state('');

    /** @type {import('$lib/commands/env').EnvFile | null} */
    let fileMeta = $state(null);

    function parseEnv(content) {
        const lines = content.split('\n');
        const vars = [];
        for (const line of lines) {
            const trimmed = line.trim();
            if (!trimmed || trimmed.startsWith('#')) continue;
            const eqIndex = trimmed.indexOf('=');
            if (eqIndex === -1) continue;
            const key = trimmed.slice(0, eqIndex).trim();
            const value = trimmed.slice(eqIndex + 1).trim();
            if (key) vars.push({ key, value });
        }
        return vars;
    }

    function serializeEnv(vars) {
        return vars.map((v) => `${v.key}=${v.value}`).join('\n') + '\n';
    }

    async function loadFile() {
        if (!folderPath || !relPath) return;
        loading = true;
        error = '';
        try {
            const result = await readEnvFile(folderPath, relPath);
            variables = parseEnv(result.content);
            originalVariables = JSON.parse(JSON.stringify(variables));
            const allFiles = await listEnvFiles(folderPath);
            fileMeta = allFiles.find((f) => f.relPath === relPath) ?? null;
        } catch (e) {
            error = e?.message ?? String(e);
        } finally {
            loading = false;
        }
    }

    $effect(() => {
        if (relPath && folderPath) void loadFile();
    });

    async function handleSave() {
        if (!folderPath || !relPath) return;
        saving = true;
        error = '';
        try {
            await writeEnvFile(folderPath, relPath, serializeEnv(variables));
            originalVariables = JSON.parse(JSON.stringify(variables));
        } catch (e) {
            error = e?.message ?? String(e);
        } finally {
            saving = false;
        }
    }

    function addVariable() {
        variables = [...variables, { key: '', value: '' }];
    }

    function deleteVariable(index) {
        variables = variables.filter((_, i) => i !== index);
    }

    async function handleToggleGitignore() {
        if (!fileMeta) return;
        try {
            await toggleEnvGitignore(folderPath, fileMeta);
            const allFiles = await listEnvFiles(folderPath);
            fileMeta = allFiles.find((f) => f.relPath === relPath) ?? fileMeta;
            workspace.refreshEnvFiles();
        } catch (e) {
            error = e?.message ?? String(e);
        }
    }

    async function handleDelete() {
        if (!folderPath || !relPath) return;
        try {
            await deleteEnvFile(folderPath, relPath);
            workspace.refreshEnvFiles();
            workspace.closeTab(tabId);
        } catch (e) {
            error = e?.message ?? String(e);
        }
    }

    let isDirty = $derived(JSON.stringify(variables) !== JSON.stringify(originalVariables));

    let copiedIndex = $state(-1);
    let gitignoreHovering = $state(false);

    async function copyValue(index) {
        try {
            await navigator.clipboard.writeText(variables[index].value);
            copiedIndex = index;
            setTimeout(() => { copiedIndex = -1; }, 1500);
        } catch (e) {
            console.error('Copy failed', e);
        }
    }
</script>

<div class="h-full flex flex-col">
    <!-- Header -->
    <div class="border-b px-6 py-4 flex items-center justify-between bg-background shrink-0">
        <div class="flex items-center gap-3">
            <FileKey class="w-5 h-5 text-muted-foreground" />
            <div class="font-mono text-base font-medium truncate max-w-md">{relPath}</div>
        </div>
        <div class="flex items-center gap-2">
            <!-- Gitignore toggle: single pill that shows state and toggles on click -->
            {#if fileMeta}
                <button
                    type="button"
                    onclick={handleToggleGitignore}
                    onmouseenter={() => (gitignoreHovering = true)}
                    onmouseleave={() => (gitignoreHovering = false)}
                    class="inline-flex items-center gap-1.5 h-7 px-2.5 rounded-full text-xs font-medium border transition-all
                        {fileMeta.inGitignore
                            ? 'border-green-600/30 text-green-600 hover:bg-destructive hover:text-destructive-foreground hover:border-destructive'
                            : 'border-border text-muted-foreground hover:border-border hover:text-foreground hover:bg-muted'}"
                >
                    {#if fileMeta.inGitignore}
                        <ShieldCheck class="w-3.5 h-3.5" />
                        {gitignoreHovering ? 'Remove from .gitignore' : 'in .gitignore'}
                    {:else}
                        <ShieldOff class="w-3.5 h-3.5" />
                        {gitignoreHovering ? 'Add to .gitignore' : 'not ignored'}
                    {/if}
                </button>
            {/if}

            <AlertDialog.Root>
                <AlertDialog.Trigger>
                    {#snippet child({ props })}
                        <Button {...props} variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground hover:text-destructive hover:bg-destructive/10">
                            <Trash2 class="w-4 h-4" />
                        </Button>
                    {/snippet}
                </AlertDialog.Trigger>
                <AlertDialog.Content>
                    <AlertDialog.Header>
                        <AlertDialog.Title>Delete {relPath}?</AlertDialog.Title>
                        <AlertDialog.Description>This action cannot be undone.</AlertDialog.Description>
                    </AlertDialog.Header>
                    <AlertDialog.Footer>
                        <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
                        <AlertDialog.Action onclick={handleDelete} class="bg-destructive">
                            Delete file
                        </AlertDialog.Action>
                    </AlertDialog.Footer>
                </AlertDialog.Content>
            </AlertDialog.Root>

            {#if saving}
                <Button disabled size="sm">
                    <Loader2 class="w-4 h-4 mr-2 animate-spin" />Saving...
                </Button>
            {:else if isDirty}
                <Button onclick={handleSave} size="sm">
                    <Save class="w-4 h-4 mr-2" />Save
                </Button>
            {:else}
                <span class="inline-flex items-center gap-1.5 text-xs text-muted-foreground px-1">
                    <Check class="w-3.5 h-3.5" />Saved
                </span>
            {/if}
        </div>
    </div>

    <!-- Editor Table -->
    <div class="flex-1 p-6 overflow-auto">
        {#if loading}
            <div class="flex h-full items-center justify-center text-muted-foreground">
                <Loader2 class="w-8 h-8 animate-spin" />
            </div>
        {:else if error}
            <div class="text-destructive text-center py-8">{error}</div>
        {:else}
            <div class="max-w-3xl mx-auto">
                <div class="grid grid-cols-[1fr_2fr_auto_auto] gap-2 items-center text-sm font-medium text-muted-foreground mb-2 px-2">
                    <div>KEY</div>
                    <div>VALUE</div>
                    <div class="w-8"></div>
                    <div class="w-8"></div>
                </div>
                {#each variables as variable, index (index)}
                    <div class="grid grid-cols-[1fr_2fr_auto_auto] gap-2 items-center mb-2 group">
                        <Input bind:value={variable.key} placeholder="KEY_NAME" class="font-mono" />
                        <Input bind:value={variable.value} placeholder="value here" class="font-mono" />
                        <Button
                            variant="ghost"
                            size="icon"
                            class="text-muted-foreground hover:text-foreground"
                            title="Copy value"
                            onclick={() => copyValue(index)}
                        >
                            {#if copiedIndex === index}
                                <Check class="w-4 h-4 text-green-600" />
                            {:else}
                                <Copy class="w-4 h-4" />
                            {/if}
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            class="text-muted-foreground hover:text-destructive hover:bg-destructive/10"
                            onclick={() => deleteVariable(index)}
                        >
                            <X class="w-4 h-4" />
                        </Button>
                    </div>
                {/each}
                <Button variant="outline" onclick={addVariable} class="mt-4 w-full">
                    <Plus class="w-4 h-4 mr-2" />Add variable
                </Button>
            </div>
        {/if}
    </div>

    <!-- Unsaved indicator -->
    {#if isDirty}
        <div class="px-6 py-3 border-t text-amber-600 text-xs flex items-center gap-2 bg-amber-50 dark:bg-amber-950">
            <span class="font-medium">● Unsaved changes</span>
        </div>
    {/if}
</div>
