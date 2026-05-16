<script>
  // @ts-nocheck
  let { data, tabId } = $props();

  import { readProjectFile, writeProjectFile } from '$lib/commands/files.js';
  import { workspace } from '$lib/stores/workspace.svelte.js';
  import { untrack } from 'svelte';
  import MarkdownEditor from '$lib/components/MarkdownEditor.svelte';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Loader2, XCircle, ChevronDown, ChevronRight, Plus, X, Tags } from '@lucide/svelte';

  let projectPath = $derived(data.folderPath);
  let relPath     = $derived(data.relPath);

  let loading   = $state(true);
  let loadError = $state('');
  let body      = $state('');

  /** @type {{ key: string, value: string }[]} */
  let frontmatterFields = $state([]);
  let originalFieldsJson = $state('[]');

  let fmOpen = $state(false);
  let bodyDirty = $state(false);

  let fmDirty = $derived(
    JSON.stringify(frontmatterFields.map(f => ({ key: f.key, value: f.value }))) !== originalFieldsJson
  );

  $effect(() => {
    const dirty = bodyDirty || fmDirty;
    untrack(() => workspace.setTabDirty(tabId, dirty));
  });

  /** @param {string} raw @returns {{ fields: {key:string,value:string}[], body: string }} */
  function parseFrontmatter(raw) {
    if (!raw.startsWith('---\n') && !raw.startsWith('---\r\n')) {
      return { fields: [], body: raw };
    }
    const rest = raw.slice(4);
    const closeIdx = rest.indexOf('\n---');
    if (closeIdx === -1) return { fields: [], body: raw };
    const yamlBlock = rest.slice(0, closeIdx);
    let bodyPart = rest.slice(closeIdx + 4);
    if (bodyPart.startsWith('\n')) bodyPart = bodyPart.slice(1);
    const fields = [];
    for (const line of yamlBlock.split('\n')) {
      const ci = line.indexOf(':');
      if (ci === -1) continue;
      const k = line.slice(0, ci).trim();
      const v = line.slice(ci + 1).trim();
      if (k) fields.push({ key: k, value: v });
    }
    return { fields, body: bodyPart };
  }

  /** @param {{ key: string, value: string }[]} fields @returns {string} */
  function serializeFrontmatter(fields) {
    const valid = fields.filter(f => f.key.trim());
    if (valid.length === 0) return '';
    return `---\n${valid.map(f => `${f.key}: ${f.value}`).join('\n')}\n---\n\n`;
  }

  $effect(() => {
    if (projectPath && relPath) void load();
  });

  async function load() {
    loading = true;
    loadError = '';
    try {
      const raw = await readProjectFile(projectPath, relPath);
      const parsed = parseFrontmatter(raw);
      frontmatterFields = parsed.fields;
      originalFieldsJson = JSON.stringify(parsed.fields.map(f => ({ key: f.key, value: f.value })));
      fmOpen = parsed.fields.length > 0;
      body = parsed.body;
    } catch (e) {
      loadError = e?.message ?? String(e);
    } finally {
      loading = false;
    }
  }

  async function handleSave(bodyMd) {
    const validFields = frontmatterFields.filter(f => f.key.trim());
    const fm = serializeFrontmatter(validFields);
    await writeProjectFile(projectPath, relPath, fm + bodyMd);
    originalFieldsJson = JSON.stringify(validFields.map(f => ({ key: f.key, value: f.value })));
    if (frontmatterFields.some(f => !f.key.trim())) {
      frontmatterFields = validFields;
    }
  }

  function addField() {
    frontmatterFields = [...frontmatterFields, { key: '', value: '' }];
  }

  /** @param {number} idx */
  function removeField(idx) {
    frontmatterFields = frontmatterFields.filter((_, i) => i !== idx);
  }
</script>

{#if loading}
  <div class="h-full flex items-center justify-center gap-2 text-muted-foreground">
    <Loader2 size={14} class="animate-spin" />
    <span class="text-xs">Loading…</span>
  </div>
{:else if loadError}
  <div class="m-8 border border-destructive bg-destructive/10 rounded-lg p-5 flex items-start gap-3">
    <XCircle class="w-4 h-4 text-destructive mt-0.5 shrink-0" />
    <pre class="text-xs text-muted-foreground whitespace-pre-wrap">{loadError}</pre>
  </div>
{:else}
  <div class="h-full flex flex-col overflow-hidden">

    <!-- Frontmatter panel -->
    <div class="border-b shrink-0 bg-muted/20">
      <button
        type="button"
        onclick={() => (fmOpen = !fmOpen)}
        class="w-full flex items-center gap-2 px-4 py-2 text-xs text-muted-foreground hover:text-foreground hover:bg-muted/30 transition-colors"
      >
        {#if fmOpen}
          <ChevronDown size={12} class="shrink-0" />
        {:else}
          <ChevronRight size={12} class="shrink-0" />
        {/if}
        <Tags size={12} class="shrink-0" />
        <span class="font-medium">Frontmatter</span>
        {#if fmDirty}
          <span class="w-1.5 h-1.5 rounded-full bg-amber-500 shrink-0"></span>
        {/if}
        {#if !fmOpen}
          {#if frontmatterFields.length > 0}
            <span class="ml-auto text-[10px] text-muted-foreground/70">{frontmatterFields.length} {frontmatterFields.length === 1 ? 'field' : 'fields'}</span>
          {:else}
            <span class="ml-auto text-[10px] text-muted-foreground/50">no fields</span>
          {/if}
        {/if}
      </button>

      {#if fmOpen}
        <div class="px-4 pb-3 pt-1">
          {#if frontmatterFields.length > 0}
            <div class="grid grid-cols-[1fr_2fr_auto] gap-x-2 gap-y-1.5 mb-2 items-center text-[10px] font-medium text-muted-foreground/60 uppercase tracking-wider px-1">
              <div>Key</div>
              <div>Value</div>
              <div class="w-6"></div>
            </div>
            {#each frontmatterFields as field, i (i)}
              <div class="grid grid-cols-[1fr_2fr_auto] gap-2 items-center mb-1.5">
                <Input bind:value={field.key} placeholder="key" class="h-7 text-xs font-mono" />
                <Input bind:value={field.value} placeholder="value" class="h-7 text-xs font-mono" />
                <button
                  type="button"
                  onclick={() => removeField(i)}
                  aria-label="Remove field"
                  class="w-7 h-7 flex items-center justify-center rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors shrink-0"
                >
                  <X size={13} />
                </button>
              </div>
            {/each}
          {:else}
            <p class="text-xs text-muted-foreground/50 mb-2 px-1">No frontmatter fields.</p>
          {/if}
          <button
            type="button"
            onclick={addField}
            class="mt-1 flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors px-1"
          >
            <Plus size={12} />Add field
          </button>
        </div>
      {/if}
    </div>

    <!-- Markdown editor -->
    <div class="flex-1 overflow-hidden min-h-0">
      <MarkdownEditor
        content={body}
        title={relPath.split('/').pop()}
        onSave={handleSave}
        onDirtyChange={(d) => { bodyDirty = d; }}
      />
    </div>

  </div>
{/if}
