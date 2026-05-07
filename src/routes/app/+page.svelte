<script>
// @ts-nocheck

  import { useSearchParams, createSearchParamsSchema } from "runed/kit";
  import { scanProject } from "$lib/commands/project";
  import { fade, fly, scale } from "svelte/transition";
  import { quintOut } from "svelte/easing";
  import { 
    FileCode2, 
    GitBranch, 
    CheckCircle2, 
    XCircle, 
    Folder, 
    FileText,
    Settings,
    Container,
    Eye,
    Code
  } from "@lucide/svelte";
  import { marked } from 'marked';
  import { openUrl } from "@tauri-apps/plugin-opener";

  const schema = createSearchParamsSchema({
    path: { type: "string", default: "" },
  });

  const params = useSearchParams(schema, {
    pushHistory: false,
  });

  let folderPath = $derived(params.path);
  let folderName = $derived(
    folderPath.split(/[\\/]/).filter(Boolean).pop() ?? "Project",
  );

  let projectInfo = $state(null);
  let isScanning = $state(false);
  let scanError = $state(null);
  let readmeView = $state('preview'); // 'preview' | 'raw'

  // Configure marked for better rendering
  marked.setOptions({
    breaks: true,
    gfm: true,
  });

  // Custom renderer
  const renderer = new marked.Renderer();

  // Custom link handler (unchanged)
  renderer.link = ({ href, title, text }) => {
    const titleAttr = title ? ` title="${title}"` : '';
    return `<a href="${href}"${titleAttr} data-external-link>${text}</a>`;
  };

  // NEW: Custom code block renderer (GitHub-style + padding)
  renderer.code = ({ text, lang, escaped }) => {
    const language = lang || '';
    const dataCode = btoa(unescape(encodeURIComponent(text)));
    
    const escapedText = escaped 
      ? text 
      : text
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;');

    const langHtml = language 
      ? `<span class="ml-2 text-[10px] font-mono tracking-widest uppercase ">${language}</span>` 
      : '';

    return `
<div class="relative group/code my-6">
  <pre class="bg-muted border border-border rounded-2xl p-5 text-lg font-mono text-foreground overflow-x-auto leading-relaxed"><code>${escapedText}</code></pre>
  
  <!-- Copy button with language inside -->
  <button 
    class="copy-button cursor-pointer absolute top-3 right-3 flex items-center justify-center gap-1.5 rounded-2xl border bg-background shadow-sm px-3 py-1.5 opacity-0 transition-all group-hover/code:opacity-100 hover:bg-primary hover:text-primary-foreground hover:border-primary text-xs font-medium"
    data-code="${dataCode}"
    title="Copy code"
  >
    <!-- Copy icon -->
    <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.75" stroke-linecap="round" stroke-linejoin="round">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
      <path d="M5 15H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
    </svg>
    ${langHtml}
  </button>
</div>`;
  };

  marked.use({ renderer });

  // Handle clicks on external links (unchanged)
  function handleReadmeClick(event) {
    const target = event.target;
    if (target.tagName === 'A' && target.hasAttribute('data-external-link')) {
      event.preventDefault();
      const href = target.getAttribute('href');
      if (href) {
        openUrl(href);
      }
    }
  }

  // Derived HTML for README preview (so we can react to it)
  let renderedReadmeHtml = $derived(
    projectInfo?.rootReadme ? marked.parse(projectInfo.rootReadme) : ''
  );

  // Auto-attach copy functionality to all code blocks after render
  $effect(() => {
    if (readmeView !== 'preview' || !renderedReadmeHtml) return;

    const timer = setTimeout(() => {
      const buttons = document.querySelectorAll('.copy-button');
      
      buttons.forEach((button) => {
        // Remove any previous listeners (safe)
        button.replaceWith(button.cloneNode(true));
        const freshButton = document.querySelectorAll('.copy-button').item(Array.from(buttons).indexOf(button));
        
        freshButton?.addEventListener('click', async (e) => {
          e.stopImmediatePropagation();
          
          const dataCode = freshButton.getAttribute('data-code');
          if (!dataCode) return;

          try {
            // Decode base64 safely
            const codeText = decodeURIComponent(escape(atob(dataCode)));
            await navigator.clipboard.writeText(codeText);

            // Visual feedback: show checkmark
            const originalHTML = freshButton.innerHTML;
            freshButton.innerHTML = `
              <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
            `;

            setTimeout(() => {
              freshButton.innerHTML = originalHTML;
            }, 2000);
          } catch (err) {
            console.error('Failed to copy code:', err);
          }
        });
      });
    }, 80); // Tiny delay so DOM has the new HTML

    return () => clearTimeout(timer);
  });

  $effect(() => {
    if (!folderPath) {
      projectInfo = null;
      return;
    }

    void (async () => {
      isScanning = true;
      scanError = null;

      try {
        const info = await scanProject(folderPath);
        projectInfo = info;
        console.log("✅ Project scanned:", info);
      } catch (error) {
        console.error("Project scan failed", error);
        scanError = error;
      } finally {
        isScanning = false;
      }
    })();
  });
</script>

<div class="h-screen w-screen flex flex-col overflow-hidden">
  <!-- Header (unchanged) -->
  <div class="border-b px-8 py-6 bg-background">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div>
          <h1 class="text-3xl font-bold tracking-tight">{folderName}</h1>
          <p class="text-sm text-muted-foreground font-mono mt-0.5">{folderPath}</p>
        </div>
      </div>

      {#if isScanning}
        <div class="flex items-center gap-2 text-sm mr-8">
          <div class="w-2 h-2 bg-primary rounded-full animate-pulse"></div>
          <span class="font-medium">Scanning...</span>
        </div>
      {/if}
    </div>
  </div>

  <!-- Content -->
  <div class="flex-1 overflow-auto p-8 pb-16">
    {#if scanError}
      <div 
        class="border border-destructive bg-destructive/10 rounded-lg p-6"
        in:fly={{ y: 20, duration: 400, easing: quintOut }}
      >
        <div class="flex items-start gap-3">
          <XCircle class="w-5 h-5 text-destructive mt-0.5" />
          <div>
            <h3 class="font-semibold text-destructive">Scan failed</h3>
            <pre class="mt-2 text-sm text-muted-foreground whitespace-pre-wrap">{scanError.message || scanError}</pre>
          </div>
        </div>
      </div>
    {:else if projectInfo}
      <div class="max-w-7xl mx-auto space-y-6 pb-8">
        
        <!-- Top Stats Row (unchanged) -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
          <!-- ... same as before ... -->
          <div 
            class="border rounded-lg p-6"
            in:scale={{ duration: 400, delay: 100, easing: quintOut }}
          >
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-sm font-semibold flex items-center gap-2">
                <Settings class="w-4 h-4" />
                TakeRest
              </h3>
              {#if projectInfo.takerestInitialized}
                <CheckCircle2 class="w-5 h-5 text-green-600" />
              {:else}
                <XCircle class="w-5 h-5 text-muted-foreground" />
              {/if}
            </div>
            <p class="text-2xl font-bold">
              {projectInfo.takerestInitialized ? 'Initialized' : 'Not Active'}
            </p>
            <p class="text-xs text-muted-foreground mt-1">Workspace Status</p>
          </div>

          <div 
            class="border rounded-lg p-6"
            in:scale={{ duration: 400, delay: 200, easing: quintOut }}
          >
            <div class="flex items-center gap-2 mb-4">
              <GitBranch class="w-4 h-4" />
              <h3 class="text-sm font-semibold">Git Repository</h3>
            </div>
            <p class="text-2xl font-bold font-mono truncate">{projectInfo.git?.repoName ?? 'N/A'}</p>
            <p class="text-xs text-muted-foreground mt-1">
              Branch: <span class="font-mono">{projectInfo.git?.branch ?? 'N/A'}</span>
            </p>
          </div>

          <div 
            class="border rounded-lg p-6"
            in:scale={{ duration: 400, delay: 300, easing: quintOut }}
          >
            <div class="flex items-center gap-2 mb-4">
              <FileCode2 class="w-4 h-4" />
              <h3 class="text-sm font-semibold">Dominant Filetype</h3>
            </div>
            <div class="flex items-baseline gap-2">
              <p class="text-2xl font-bold font-mono">{projectInfo.majorFiletype?.extension ?? 'N/A'}</p>
              <p class="text-lg text-muted-foreground">×{projectInfo.majorFiletype?.count ?? 0}</p>
            </div>
            <p class="text-xs text-muted-foreground mt-1">files in project</p>
          </div>
        </div>

        <!-- Config Files Row (unchanged) -->
        <div 
          class="grid grid-cols-1 md:grid-cols-2 gap-6"
          in:fly={{ y: 30, duration: 500, delay: 400, easing: quintOut }}
        >
          <div class="border rounded-lg p-6">
            <div class="flex items-center gap-2 mb-4">
              <FileText class="w-4 h-4" />
              <h3 class="text-sm font-semibold">Environment Files</h3>
            </div>
            <p class="text-4xl font-bold tabular-nums">{projectInfo.envFiles.length}</p>
            <p class="text-xs text-muted-foreground mt-1">.env files found</p>
          </div>

          <div class="border rounded-lg p-6">
            <div class="flex items-center gap-2 mb-4">
              <Container class="w-4 h-4" />
              <h3 class="text-sm font-semibold">Docker Compose</h3>
            </div>
            <p class="text-4xl font-bold tabular-nums">{projectInfo.composeFiles.length}</p>
            <p class="text-xs text-muted-foreground mt-1">compose files found</p>
          </div>
        </div>

        <!-- README Section -->
        <div 
          class="border rounded-lg overflow-hidden"
          in:fly={{ y: 30, duration: 500, delay: 500, easing: quintOut }}
        >
          {#if projectInfo.rootReadme}
            <!-- Header with Tabs (unchanged) -->
            <div class="flex items-center justify-between border-b px-6 py-3 bg-muted/30">
              <div class="flex items-center gap-2">
                <FileText class="w-4 h-4" />
                <h3 class="text-sm font-semibold">README.md</h3>
              </div>
              
              <!-- View Toggle -->
              <div class="inline-flex rounded-md border bg-background p-1">
                <button
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded transition-colors
                         {readmeView === 'preview' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}"
                  onclick={() => readmeView = 'preview'}
                >
                  <Eye class="w-3.5 h-3.5" />
                  Preview
                </button>
                <button
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded transition-colors
                         {readmeView === 'raw' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}"
                  onclick={() => readmeView = 'raw'}
                >
                  <Code class="w-3.5 h-3.5" />
                  Raw
                </button>
              </div>
            </div>

            <!-- Content -->
            <div class="p-6">
              {#if readmeView === 'preview'}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div 
                  class="prose prose-sm dark:prose-invert max-w-none"
                  onclick={handleReadmeClick}
                >
                  {@html renderedReadmeHtml}
                </div>
              {:else}
                <pre class="bg-muted border border-border rounded-2xl p-6 text-sm font-mono overflow-x-auto whitespace-pre leading-relaxed max-w-full">{projectInfo.rootReadme}</pre>
              {/if}
            </div>
          {/if}
        </div>

      </div>
    {:else}
      <div class="flex items-center justify-center h-full">
        <p class="text-muted-foreground">Waiting for a valid project path...</p>
      </div>
    {/if}
  </div>
</div>