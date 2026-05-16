<script>
    // @ts-nocheck
    let { data, tabId } = $props();

    import { workspace } from '$lib/stores/workspace.svelte.js';

    import { scanProject, saveReadme } from '$lib/commands/project';
    import { fly } from 'svelte/transition';
    import { quintOut } from 'svelte/easing';
    import { XCircle, FileText, Eye, Code, Edit3 } from '@lucide/svelte';
    import { marked } from 'marked';
    import { openUrl } from '@tauri-apps/plugin-opener';
    import MarkdownEditor from '$lib/components/MarkdownEditor.svelte';

    let folderPath = $derived(data.folderPath);
    let folderName = $derived(folderPath.split(/[\\/]/).filter(Boolean).pop() ?? 'Project');

    let projectInfo = $state(null);
    let isScanning = $state(false);
    let scanError = $state(null);
    let readmeView = $state('preview');

    const renderer = new marked.Renderer();

    renderer.link = ({ href, title, text }) => {
        const titleAttr = title ? ` title="${title}"` : '';
        return `<a href="${href}"${titleAttr} data-external-link>${text}</a>`;
    };

    renderer.code = ({ text, lang, escaped }) => {
        const language = lang || '';
        const dataCode = btoa(unescape(encodeURIComponent(text)));
        const escapedText = escaped
            ? text
            : text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
        const langHtml = language
            ? `<span class="ml-2 text-[10px] font-mono tracking-widest uppercase">${language}</span>`
            : '';
        return `
<div class="relative group/code my-6">
  <pre class="bg-muted border border-border rounded-2xl p-5 text-lg font-mono text-foreground overflow-x-auto leading-relaxed"><code>${escapedText}</code></pre>
  <button
    class="copy-button cursor-pointer absolute top-3 right-3 flex items-center justify-center gap-1.5 rounded-2xl border bg-background shadow-sm px-3 py-1.5 opacity-0 transition-all group-hover/code:opacity-100 hover:bg-primary hover:text-primary-foreground hover:border-primary text-xs font-medium"
    data-code="${dataCode}"
    title="Copy code"
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.75" stroke-linecap="round" stroke-linejoin="round">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
      <path d="M5 15H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
    </svg>
    ${langHtml}
  </button>
</div>`;
    };

    marked.use({ renderer, gfm: true, breaks: true });

    function fixMarkdownTables(md) {
        const lines = md.split('\n');
        const result = [];
        let i = 0;
        let inFence = false;
        let fenceChar = '';
        while (i < lines.length) {
            const line = lines[i];
            const fenceMatch = line.trim().match(/^(`{3,}|~{3,})/);
            if (fenceMatch) {
                const ch = fenceMatch[1][0];
                if (!inFence) { inFence = true; fenceChar = ch; }
                else if (ch === fenceChar) { inFence = false; fenceChar = ''; }
                result.push(line); i++; continue;
            }
            if (inFence || !line.includes('|')) { result.push(line); i++; continue; }
            const block = [];
            let j = i;
            while (j < lines.length && lines[j].includes('|')) { block.push(lines[j]); j++; }
            const isSep = (l) =>
                /^\|?[\s]*[\-:]+[\s]*(\|[\s]*[\-:]+[\s]*)*\|?$/.test(l.trim()) && l.includes('-');
            const hasSep = block.some(isSep);
            if (!hasSep && block.length >= 2) {
                const parts = block[0].split('|').slice(1, -1);
                const cols = parts.length || 1;
                const sep = '| ' + Array(cols).fill('---').join(' | ') + ' |';
                result.push(block[0], sep, ...block.slice(1));
            } else {
                result.push(...block);
            }
            i = j;
        }
        return result.join('\n');
    }

    let renderedReadmeHtml = $derived(
        projectInfo?.rootReadme
            ? marked.parse(fixMarkdownTables(projectInfo.rootReadme))
            : '',
    );

    $effect(() => {
        if (readmeView !== 'preview' || !renderedReadmeHtml) return;
        const timer = setTimeout(() => {
            const buttons = document.querySelectorAll('.copy-button');
            buttons.forEach((button) => {
                button.replaceWith(button.cloneNode(true));
                const freshButton = document.querySelectorAll('.copy-button').item(
                    Array.from(buttons).indexOf(button)
                );
                freshButton?.addEventListener('click', async (e) => {
                    e.stopImmediatePropagation();
                    const dataCode = freshButton.getAttribute('data-code');
                    if (!dataCode) return;
                    try {
                        const codeText = decodeURIComponent(escape(atob(dataCode)));
                        await navigator.clipboard.writeText(codeText);
                        const originalHTML = freshButton.innerHTML;
                        freshButton.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
                        setTimeout(() => { freshButton.innerHTML = originalHTML; }, 2000);
                    } catch (err) {
                        console.error('Failed to copy code:', err);
                    }
                });
            });
        }, 80);
        return () => clearTimeout(timer);
    });

    function handleReadmeClick(event) {
        const anchor = event.target.closest?.('a[data-external-link]');
        if (!anchor) return;
        event.preventDefault();
        const href = anchor.getAttribute('href');
        if (href) openUrl(href);
    }

    // ── FileEditor callbacks (used in edit mode) ───────────────────────────────
    function readmeLoad() {
        return Promise.resolve(projectInfo?.rootReadme ?? '');
    }

    async function readmeSave(content) {
        const currentFolder = folderPath;
        await saveReadme(currentFolder, content.trimEnd() + '\n');
        const info = await scanProject(currentFolder);
        if (folderPath !== currentFolder) return;
        projectInfo = info;
        workspace.gitInfo = info.git ?? null;
        readmeView = 'preview';
        workspace.setTabDirty(tabId, false);
    }

    $effect(() => {
        workspace.fileChangeTicks['README.md']; // reload when README.md changes on disk
        if (!folderPath) { projectInfo = null; return; }
        const currentFolder = folderPath;
        void (async () => {
            isScanning = true;
            scanError = null;
            try {
                const info = await scanProject(currentFolder);
                if (folderPath !== currentFolder) return;
                projectInfo = info;
                workspace.gitInfo = info.git ?? null;
            }
            catch (error) {
                if (folderPath === currentFolder) scanError = error;
            }
            finally { isScanning = false; }
        })();
    });
</script>

<div class="h-full w-full flex flex-col overflow-hidden">

    <!-- Toolbar: hidden in edit mode (MarkdownEditor has its own header) -->
    {#if readmeView !== 'edit'}
    <div class="flex items-center justify-between px-5 border-b shrink-0 h-10">
        <div class="flex items-center gap-2 text-muted-foreground">
            <FileText class="w-3.5 h-3.5" />
            <span class="text-xs font-medium">README.md</span>
            {#if isScanning}
                <span class="w-1.5 h-1.5 rounded-full bg-primary animate-pulse"></span>
            {/if}
        </div>

        {#if projectInfo}
            <div class="flex items-center">
                {#each [['preview', Eye, 'Preview'], ['raw', Code, 'Raw']] as [v, Icon, label]}
                    <button
                        type="button"
                        onclick={() => (readmeView = v)}
                        class="inline-flex items-center gap-1.5 px-3 h-10 text-xs transition-colors border-b-2
                            {readmeView === v
                                ? 'text-foreground border-primary'
                                : 'text-muted-foreground border-transparent hover:text-foreground'}"
                    >
                        <Icon class="w-3 h-3" />{label}
                    </button>
                {/each}
                <button
                    type="button"
                    onclick={() => (readmeView = 'edit')}
                    class="inline-flex items-center gap-1.5 px-3 h-10 text-xs text-muted-foreground hover:text-foreground transition-colors border-b-2 border-transparent"
                >
                    <Edit3 class="w-3 h-3" />Edit
                </button>
            </div>
        {/if}
    </div>
    {/if}

    <!-- Content -->
    {#if readmeView === 'edit'}
        <!-- FileEditor fills the remaining height (below the toolbar header) -->
        <div class="flex-1 overflow-hidden">
            <MarkdownEditor
                title="README.md"
                content={projectInfo?.rootReadme ?? ''}
                onSave={readmeSave}
                onCancel={() => { readmeView = 'preview'; workspace.setTabDirty(tabId, false); }}
                onDirtyChange={(d) => workspace.setTabDirty(tabId, d)}
            />
        </div>
    {:else}
        <div class="flex-1 overflow-auto">
            {#if scanError}
                <div class="m-8 border border-destructive bg-destructive/10 rounded-lg p-5">
                    <div class="flex items-start gap-3">
                        <XCircle class="w-4 h-4 text-destructive mt-0.5 shrink-0" />
                        <div>
                            <p class="text-sm font-semibold text-destructive">Scan failed</p>
                            <pre class="mt-1.5 text-xs text-muted-foreground whitespace-pre-wrap">{scanError.message || scanError}</pre>
                        </div>
                    </div>
                </div>
            {:else if projectInfo}
                {#if projectInfo.rootReadme}
                    {#if readmeView === 'preview'}
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div
                            class="px-10 py-8 prose prose-sm dark:prose-invert max-w-3xl"
                            onclick={handleReadmeClick}
                            in:fly={{ y: 10, duration: 250, easing: quintOut }}
                        >
                            {@html renderedReadmeHtml}
                        </div>
                    {:else if readmeView === 'raw'}
                        <pre class="m-8 bg-muted/50 border border-border rounded-xl p-6 text-sm font-mono overflow-x-auto whitespace-pre leading-relaxed">{projectInfo.rootReadme}</pre>
                    {/if}
                {:else}
                    <div class="flex flex-col items-center justify-center h-full gap-3 text-muted-foreground">
                        <FileText class="w-10 h-10 opacity-30" />
                        <p class="text-sm">No README.md yet</p>
                        <button
                            type="button"
                            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
                            onclick={() => (readmeView = 'edit')}
                        >
                            <Edit3 class="w-3.5 h-3.5" />Create README
                        </button>
                    </div>
                {/if}
            {:else if !isScanning}
                <div class="flex items-center justify-center h-full">
                    <p class="text-sm text-muted-foreground">Waiting for a valid project path…</p>
                </div>
            {/if}
        </div>
    {/if}

</div>

<style>
  /* GFM task list checkboxes in preview — same primary-color override as MarkdownEditor */
  :global(.prose input[type="checkbox"]) {
    color: var(--primary) !important;
    border-color: color-mix(in oklch, var(--primary) 55%, var(--border)) !important;
    border-radius: 0.2rem !important;
  }
</style>
