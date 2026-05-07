<script>
// @ts-nocheck

  import { useSearchParams, createSearchParamsSchema } from "runed/kit";
  import { scanProject, saveReadme } from "$lib/commands/project";
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
    Code,
    Edit3,
    Save,
    X as XIcon
  } from "@lucide/svelte";
  import { marked } from 'marked';
  import { openUrl } from "@tauri-apps/plugin-opener";
  
  import { EdraEditor, EdraToolBar, EdraDragHandleExtended } from '$lib/components/edra/shadcn';

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
  let readmeView = $state('preview'); // 'preview' | 'raw' | 'edit'
  
  // Editor states
  let editorContent = $state(null);
  let editor = $state(null);
  let isSaving = $state(false);

  // Configure marked for better rendering
  marked.setOptions({
    breaks: true,
    gfm: true,
  });

  // Custom renderer
  const renderer = new marked.Renderer();

  // Custom link handler
  renderer.link = ({ href, title, text }) => {
    const titleAttr = title ? ` title="${title}"` : '';
    return `<a href="${href}"${titleAttr} data-external-link>${text}</a>`;
  };

  // Custom code block renderer (GitHub-style + padding)
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

  // Handle clicks on external links
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

  // Derived HTML for README preview
  let renderedReadmeHtml = $derived(
    projectInfo?.rootReadme ? marked.parse(projectInfo.rootReadme) : ''
  );

  // Auto-attach copy functionality to all code blocks after render
  $effect(() => {
    if (readmeView !== 'preview' || !renderedReadmeHtml) return;

    const timer = setTimeout(() => {
      const buttons = document.querySelectorAll('.copy-button');
      
      buttons.forEach((button) => {
        button.replaceWith(button.cloneNode(true));
        const freshButton = document.querySelectorAll('.copy-button').item(Array.from(buttons).indexOf(button));
        
        freshButton?.addEventListener('click', async (e) => {
          e.stopImmediatePropagation();
          
          const dataCode = freshButton.getAttribute('data-code');
          if (!dataCode) return;

          try {
            const codeText = decodeURIComponent(escape(atob(dataCode)));
            await navigator.clipboard.writeText(codeText);

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
    }, 80);

    return () => clearTimeout(timer);
  });

  // Handle entering edit mode
  function enterEditMode() {
    readmeView = 'edit';
    // Set editor content from current README
    editorContent = renderedReadmeHtml || '';
  }

  // Handle canceling edit
  function cancelEdit() {
    readmeView = 'preview';
    editorContent = null;
  }

  // Handle saving edited content
  async function handleSave() {
    if (!editor || !folderPath) return;
    
    isSaving = true;
    
    try {
      // Get markdown content from editor
      const htmlContent = editor.getHTML();
      
      // Convert HTML back to markdown (basic conversion)
      // For better conversion, consider using a library like turndown
      const tempDiv = document.createElement('div');
      tempDiv.innerHTML = htmlContent;
      const markdownContent = convertHtmlToMarkdown(tempDiv);
      
      // Save to backend
      await saveReadme(folderPath, markdownContent);
      
      // Refresh project info to show updated content
      const info = await scanProject(folderPath);
      projectInfo = info;
      
      // Exit edit mode
      readmeView = 'preview';
      editorContent = null;
      
      console.log("✅ README saved successfully");
    } catch (error) {
      console.error("Failed to save README:", error);
      // You might want to show an error toast here
    } finally {
      isSaving = false;
    }
  }

  // Basic HTML to Markdown conversion
  function convertHtmlToMarkdown(element) {
    let markdown = '';
    
    for (const node of element.childNodes) {
      if (node.nodeType === Node.TEXT_NODE) {
        markdown += node.textContent;
      } else if (node.nodeType === Node.ELEMENT_NODE) {
        const tag = node.tagName.toLowerCase();
        
        switch (tag) {
          case 'h1':
            markdown += `# ${node.textContent}\n\n`;
            break;
          case 'h2':
            markdown += `## ${node.textContent}\n\n`;
            break;
          case 'h3':
            markdown += `### ${node.textContent}\n\n`;
            break;
          case 'h4':
            markdown += `#### ${node.textContent}\n\n`;
            break;
          case 'h5':
            markdown += `##### ${node.textContent}\n\n`;
            break;
          case 'h6':
            markdown += `###### ${node.textContent}\n\n`;
            break;
          case 'p':
            markdown += `${convertHtmlToMarkdown(node)}\n\n`;
            break;
          case 'strong':
          case 'b':
            markdown += `**${node.textContent}**`;
            break;
          case 'em':
          case 'i':
            markdown += `*${node.textContent}*`;
            break;
          case 'code':
            markdown += `\`${node.textContent}\``;
            break;
          case 'pre':
            const code = node.querySelector('code');
            markdown += `\`\`\`\n${code ? code.textContent : node.textContent}\n\`\`\`\n\n`;
            break;
          case 'a':
            markdown += `[${node.textContent}](${node.getAttribute('href')})`;
            break;
          case 'ul':
            for (const li of node.querySelectorAll('li')) {
              markdown += `- ${li.textContent}\n`;
            }
            markdown += '\n';
            break;
          case 'ol':
            let index = 1;
            for (const li of node.querySelectorAll('li')) {
              markdown += `${index}. ${li.textContent}\n`;
              index++;
            }
            markdown += '\n';
            break;
          case 'blockquote':
            markdown += `> ${convertHtmlToMarkdown(node)}\n\n`;
            break;
          case 'br':
            markdown += '\n';
            break;
          default:
            markdown += convertHtmlToMarkdown(node);
        }
      }
    }
    
    return markdown;
  }

  function onEditorUpdate() {
    // Optional: handle real-time updates
    console.log("Editor content updated");
  }

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
  <!-- Header -->
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
        
        <!-- Top Stats Row -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
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

        <!-- Config Files Row -->
        <!-- <div 
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
        </div> -->

        <!-- README Section -->
        <div 
          class="border rounded-lg overflow-hidden"
          in:fly={{ y: 30, duration: 500, delay: 500, easing: quintOut }}
        >
          {#if projectInfo.rootReadme}
            <!-- Header with Tabs -->
            <div class="flex items-center justify-between border-b px-6 py-3 bg-muted/30">
              <div class="flex items-center gap-2">
                <FileText class="w-4 h-4" />
                <h3 class="text-sm font-semibold">README.md</h3>
              </div>
              
              <!-- View Toggle & Actions -->
              <div class="flex items-center gap-2">
                {#if readmeView === 'edit'}
                  <!-- Save and Cancel buttons -->
                  <button
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded border bg-background transition-colors hover:bg-muted"
                    onclick={cancelEdit}
                    disabled={isSaving}
                  >
                    <XIcon class="w-3.5 h-3.5" />
                    Cancel
                  </button>
                  <button
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded bg-primary text-primary-foreground transition-colors hover:bg-primary/90"
                    onclick={handleSave}
                    disabled={isSaving}
                  >
                    <Save class="w-3.5 h-3.5" />
                    {isSaving ? 'Saving...' : 'Save'}
                  </button>
                {:else}
                  <!-- View mode buttons -->
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
                  
                  <!-- Edit button -->
                  <button
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded border bg-background transition-colors hover:bg-muted"
                    onclick={enterEditMode}
                  >
                    <Edit3 class="w-3.5 h-3.5" />
                    Edit
                  </button>
                {/if}
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
              {:else if readmeView === 'raw'}
                <pre class="bg-muted border border-border rounded-2xl p-6 text-sm font-mono overflow-x-auto whitespace-pre leading-relaxed max-w-full">{projectInfo.rootReadme}</pre>
              {:else if readmeView === 'edit'}
                <!-- Edra Editor -->
                <div class="bg-background rounded-md border min-h-[400px]">
                  {#if editor && !editor.isDestroyed}
                    <EdraToolBar
                      class="bg-secondary/50 flex w-full items-center overflow-x-auto border-b p-0.5"
                      {editor}
                    />
                    <EdraDragHandleExtended {editor} />
                  {/if}
                  <EdraEditor
                    bind:editor
                    content={editorContent}
                    class="h-full overflow-y-auto px-6 py-4"
                    onUpdate={onEditorUpdate}
                  />
                </div>
              {/if}
            </div>
          {:else}
            <!-- No README found -->
            <div class="p-6 text-center text-muted-foreground">
              <FileText class="w-12 h-12 mx-auto mb-3 opacity-50" />
              <p>No .takerest/README.md found</p>
              <button
                class="mt-4 inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded bg-primary text-primary-foreground transition-colors hover:bg-primary/90"
                onclick={enterEditMode}
              >
                <Edit3 class="w-3.5 h-3.5" />
                Create README
              </button>
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