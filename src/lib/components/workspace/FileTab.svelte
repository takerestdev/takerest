<script>
  // @ts-nocheck
  /**
   * Reusable file editor tab.
   * data: { projectPath: string, relPath: string, language?: string }
   * language is auto-detected from extension when omitted.
   */
  let { data } = $props();

  import { readProjectFile, writeProjectFile } from '$lib/commands/files.js';
  import { EditorView, basicSetup } from 'codemirror';
  import { yaml } from '@codemirror/lang-yaml';
  import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
  import { languages } from '@codemirror/language-data';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { Save, Undo2, Loader2, AlertTriangle, FileCode } from '@lucide/svelte';

  let projectPath = $derived(data.projectPath);
  let relPath     = $derived(data.relPath);
  let language    = $derived(data.language ?? detectLang(relPath));
  let fileName    = $derived(relPath?.split(/[\\/]/).pop() ?? relPath);

  // ── State ──────────────────────────────────────────────────────────────────
  let originalContent = $state('');
  let loading  = $state(true);
  let saving   = $state(false);
  let error    = $state('');
  let isDirty  = $state(false);

  let editorEl = $state(null);
  let view     = null; // plain var — managed entirely by $effect

  // ── Load file ──────────────────────────────────────────────────────────────
  $effect(() => {
    const path = projectPath;
    const rel  = relPath;
    if (!path || !rel) return;

    loading = true;
    error   = '';
    isDirty = false;

    readProjectFile(path, rel)
      .then(c => { originalContent = c; })
      .catch(e => { error = String(e); })
      .finally(() => { loading = false; });
  });

  // ── Create / recreate editor once content is loaded ────────────────────────
  $effect(() => {
    if (!editorEl || loading || error) return;

    view?.destroy();

    const orig = originalContent; // capture for closure

    view = new EditorView({
      doc: orig,
      extensions: [
        basicSetup,
        getLangExt(language),
        EditorView.theme({
          '&': {
            height: '100%',
            backgroundColor: 'transparent',
            color: 'hsl(var(--foreground))',
            fontFamily: 'Geist Mono, ui-monospace, monospace',
            fontSize: '13px',
          },
          '.cm-content':          { padding: '1rem 0.5rem', lineHeight: '1.65' },
          '.cm-gutters':          { backgroundColor: 'hsl(var(--muted) / 0.3)', color: 'hsl(var(--muted-foreground) / 0.6)', border: 'none' },
          '.cm-activeLineGutter': { backgroundColor: 'hsl(var(--muted) / 0.5)' },
          '.cm-activeLine':       { backgroundColor: 'hsl(var(--muted) / 0.25)' },
          '.cm-selectionBackground, ::selection': { backgroundColor: 'hsl(var(--primary) / 0.25) !important' },
          '.cm-focused':          { outline: 'none' },
          '.cm-scroller':         { overflow: 'auto' },
        }),
        oneDark,
        EditorView.updateListener.of(upd => {
          if (upd.docChanged) {
            isDirty = upd.state.doc.toString() !== orig;
          }
        }),
      ],
      parent: editorEl,
    });

    return () => { view?.destroy(); view = null; };
  });

  // ── Save ───────────────────────────────────────────────────────────────────
  async function save() {
    if (!view || !isDirty || saving) return;
    saving = true;
    error  = '';
    try {
      const content = view.state.doc.toString();
      await writeProjectFile(projectPath, relPath, content);
      originalContent = content;
      isDirty = false;
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  // ── Discard ────────────────────────────────────────────────────────────────
  function discard() {
    if (!view) return;
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: originalContent },
    });
    isDirty = false;
  }

  // ── Keyboard shortcut (Ctrl/Cmd + S) ──────────────────────────────────────
  function onKeyDown(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      void save();
    }
  }

  // ── Language detection ─────────────────────────────────────────────────────
  function detectLang(path) {
    const ext = path?.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'yml': case 'yaml': return 'yaml';
      case 'md':  case 'mdx':  return 'markdown';
      default:                  return null;
    }
  }

  function getLangExt(lang) {
    switch (lang) {
      case 'yaml':     return yaml();
      case 'markdown': return markdown({ base: markdownLanguage, codeLanguages: languages });
      default:         return [];
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="h-full flex flex-col overflow-hidden" onkeydown={onKeyDown}>

  <!-- Header -->
  <div class="flex items-center gap-2 px-4 py-2 border-b shrink-0 h-10">
    <FileCode size={14} class="text-muted-foreground shrink-0" />
    <span class="text-sm font-medium flex-1 truncate font-mono">{relPath}</span>

    {#if isDirty}
      <span class="text-[10px] text-muted-foreground/60 shrink-0">unsaved changes</span>
    {/if}

    {#if isDirty}
      <!-- Discard -->
      <button
        type="button"
        onclick={discard}
        disabled={saving}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded text-xs text-muted-foreground
               hover:text-foreground hover:bg-muted transition-colors disabled:opacity-50"
      >
        <Undo2 size={12} />
        Discard
      </button>
      <!-- Save -->
      <button
        type="button"
        onclick={save}
        disabled={saving}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium
               bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50"
      >
        {#if saving}
          <Loader2 size={12} class="animate-spin" />
          Saving…
        {:else}
          <Save size={12} />
          Save
        {/if}
      </button>
    {/if}
  </div>

  <!-- States -->
  {#if loading}
    <div class="flex flex-1 items-center justify-center gap-2 text-muted-foreground">
      <Loader2 size={16} class="animate-spin" />
      <span class="text-sm">Loading…</span>
    </div>
  {:else if error}
    <div class="flex flex-1 items-center justify-center gap-2 text-destructive px-8">
      <AlertTriangle size={16} class="shrink-0" />
      <span class="text-sm break-all">{error}</span>
    </div>
  {:else}
    <!-- Editor fills remaining space -->
    <div bind:this={editorEl} class="flex-1 overflow-hidden [&_.cm-editor]:h-full [&_.cm-editor]:text-[13px]"></div>
  {/if}

</div>
