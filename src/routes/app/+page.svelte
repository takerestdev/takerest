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
        X as XIcon,
    } from "@lucide/svelte";
    import { marked } from "marked";
    import { openUrl } from "@tauri-apps/plugin-opener";

    import {
        EdraEditor,
        EdraToolBar,
        EdraDragHandleExtended,
    } from "$lib/components/edra/shadcn";

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
    let readmeView = $state("preview"); // 'preview' | 'raw' | 'edit'

    // Editor states
    let editorContent = $state(null);
    let editor = $state(null);
    let isSaving = $state(false);

    // ─── Marked setup (single call) ───────────────────────────────────────────────
    const renderer = new marked.Renderer();

    renderer.link = ({ href, title, text }) => {
        const titleAttr = title ? ` title="${title}"` : "";
        return `<a href="${href}"${titleAttr} data-external-link>${text}</a>`;
    };

    renderer.code = ({ text, lang, escaped }) => {
        const language = lang || "";
        const dataCode = btoa(unescape(encodeURIComponent(text)));
        const escapedText = escaped
            ? text
            : text
                  .replace(/&/g, "&amp;")
                  .replace(/</g, "&lt;")
                  .replace(/>/g, "&gt;");
        const langHtml = language
            ? `<span class="ml-2 text-[10px] font-mono tracking-widest uppercase">${language}</span>`
            : "";
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

    // Single marked.use — sets renderer + gfm + breaks all at once
    marked.use({ renderer, gfm: true, breaks: true });

    // ─── Fix markdown tables that are missing the separator row ──────────────────
    // marked requires "| --- | --- |" between header and data rows to parse a table.
    // This repairs pre-existing files that were written without it.
    function fixMarkdownTables(md) {
        const lines = md.split("\n");
        const result = [];
        let i = 0;

        while (i < lines.length) {
            const line = lines[i];

            if (!line.includes("|")) {
                result.push(line);
                i++;
                continue;
            }

            // Collect all consecutive pipe-containing lines as one table block
            const block = [];
            let j = i;
            while (j < lines.length && lines[j].includes("|")) {
                block.push(lines[j]);
                j++;
            }

            // A separator row contains only -, |, :, and spaces
            const isSep = (l) =>
                /^\|?[\s]*[\-:]+[\s]*(\|[\s]*[\-:]+[\s]*)*\|?$/.test(
                    l.trim(),
                ) && l.includes("-");
            const hasSep = block.some(isSep);

            if (!hasSep && block.length >= 2) {
                // Count columns from the first row
                const parts = block[0].split("|").slice(1, -1);
                const cols = parts.length || 1;
                const sep = "| " + Array(cols).fill("---").join(" | ") + " |";
                result.push(block[0], sep, ...block.slice(1));
            } else {
                result.push(...block);
            }

            i = j;
        }

        return result.join("\n");
    }

    // ─── Preview HTML (uses custom renderer + table fix) ─────────────────────────
    let renderedReadmeHtml = $derived(
        projectInfo?.rootReadme
            ? marked.parse(fixMarkdownTables(projectInfo.rootReadme))
            : "",
    );

    // ─── Copy button wiring ───────────────────────────────────────────────────────
    $effect(() => {
        if (readmeView !== "preview" || !renderedReadmeHtml) return;
        const timer = setTimeout(() => {
            const buttons = document.querySelectorAll(".copy-button");
            buttons.forEach((button) => {
                button.replaceWith(button.cloneNode(true));
                const freshButton = document
                    .querySelectorAll(".copy-button")
                    .item(Array.from(buttons).indexOf(button));
                freshButton?.addEventListener("click", async (e) => {
                    e.stopImmediatePropagation();
                    const dataCode = freshButton.getAttribute("data-code");
                    if (!dataCode) return;
                    try {
                        const codeText = decodeURIComponent(
                            escape(atob(dataCode)),
                        );
                        await navigator.clipboard.writeText(codeText);
                        const originalHTML = freshButton.innerHTML;
                        freshButton.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
                        setTimeout(() => {
                            freshButton.innerHTML = originalHTML;
                        }, 2000);
                    } catch (err) {
                        console.error("Failed to copy code:", err);
                    }
                });
            });
        }, 80);
        return () => clearTimeout(timer);
    });

    // ─── External link handler ────────────────────────────────────────────────────
    function handleReadmeClick(event) {
        const target = event.target;
        if (target.tagName === "A" && target.hasAttribute("data-external-link")) {
            event.preventDefault();
            const href = target.getAttribute("href");
            if (href) openUrl(href);
        }
    }

    // ─── Transform marked HTML → TipTap-compatible HTML ──────────────────────────
    function prepareHtmlForEditor(html) {
        const div = document.createElement("div");
        div.innerHTML = html;

        div.querySelectorAll("ul").forEach((ul) => {
            const hasCheckbox = ul.querySelector('li > input[type="checkbox"]');
            if (!hasCheckbox) return;
            ul.setAttribute("data-type", "taskList");
            ul.querySelectorAll("li").forEach((li) => {
                const input = li.querySelector('input[type="checkbox"]');
                if (!input) return;
                const checked = input.checked || input.hasAttribute("checked");
                li.setAttribute("data-type", "taskItem");
                li.setAttribute("data-checked", String(checked));
                input.remove();
                const text = li.innerHTML.trim();
                li.innerHTML = `<p>${text}</p>`;
            });
        });

        return div.innerHTML;
    }

    // ─── Edit mode ────────────────────────────────────────────────────────────────
    function enterEditMode() {
        readmeView = "edit";
        if (!projectInfo?.rootReadme) {
            editorContent = "";
            return;
        }
        // Use clean default renderer (no copy buttons) + table fix for TipTap
        const cleanHtml = marked.parse(
            fixMarkdownTables(projectInfo.rootReadme),
            { renderer: new marked.Renderer() },
        );
        editorContent = prepareHtmlForEditor(cleanHtml);
    }

    function cancelEdit() {
        readmeView = "preview";
        editorContent = null;
    }

    // ─── Save ─────────────────────────────────────────────────────────────────────
    async function handleSave() {
        if (!editor || !folderPath) return;
        isSaving = true;
        try {
            const tempDiv = document.createElement("div");
            tempDiv.innerHTML = editor.getHTML();
            const markdownContent = convertHtmlToMarkdown(tempDiv).trimEnd() + "\n";
            await saveReadme(folderPath, markdownContent);
            const info = await scanProject(folderPath);
            projectInfo = info;
            readmeView = "preview";
            editorContent = null;
            console.log("✅ README saved successfully");
        } catch (error) {
            console.error("Failed to save README:", error);
        } finally {
            isSaving = false;
        }
    }

    // ─── Inline content ───────────────────────────────────────────────────────────
    function getInlineContent(node) {
        let text = "";
        for (const child of node.childNodes) {
            if (child.nodeType === Node.TEXT_NODE) {
                text += child.textContent;
            } else if (child.nodeType === Node.ELEMENT_NODE) {
                const tag = child.tagName.toLowerCase();
                switch (tag) {
                    case "strong":
                    case "b":   text += `**${getInlineContent(child)}**`; break;
                    case "em":
                    case "i":   text += `*${getInlineContent(child)}*`;   break;
                    case "s":   text += `~~${getInlineContent(child)}~~`; break;
                    case "code": text += `\`${child.textContent}\``;       break;
                    case "a":   text += `[${getInlineContent(child)}](${child.getAttribute("href")})`; break;
                    case "br":  text += "\n"; break;
                    default:    text += getInlineContent(child);
                }
            }
        }
        return text;
    }

    // ─── List item content ────────────────────────────────────────────────────────
    function getListItemContent(li) {
        let content = "";
        for (const child of li.childNodes) {
            if (child.nodeType === Node.TEXT_NODE) {
                content += child.textContent;
            } else if (child.nodeType === Node.ELEMENT_NODE) {
                const tag = child.tagName.toLowerCase();
                if (tag === "label" || tag === "input") continue;
                content += getInlineContent(child);
            }
        }
        return content.trim();
    }

    // ─── Table → Markdown ─────────────────────────────────────────────────────────
    function convertTableToMarkdown(table) {
        let md = "";
        const rows = Array.from(table.querySelectorAll("tr"));
        rows.forEach((row, rowIndex) => {
            const cells = Array.from(row.querySelectorAll("th, td"));
            const texts = cells.map(
                (c) => getInlineContent(c).replace(/\|/g, "\\|").trim() || " ",
            );
            md += `| ${texts.join(" | ")} |\n`;
            // Always emit separator after first row — required for valid GFM
            if (rowIndex === 0) {
                md += `| ${texts.map(() => "---").join(" | ")} |\n`;
            }
        });
        return md + "\n";
    }

    // ─── Block HTML → Markdown ────────────────────────────────────────────────────
    function convertHtmlToMarkdown(element) {
        let markdown = "";
        for (const node of element.childNodes) {
            if (node.nodeType === Node.TEXT_NODE) {
                markdown += node.textContent;
                continue;
            }
            if (node.nodeType !== Node.ELEMENT_NODE) continue;
            const tag = node.tagName.toLowerCase();
            switch (tag) {
                case "h1": markdown += `# ${getInlineContent(node)}\n\n`; break;
                case "h2": markdown += `## ${getInlineContent(node)}\n\n`; break;
                case "h3": markdown += `### ${getInlineContent(node)}\n\n`; break;
                case "h4": markdown += `#### ${getInlineContent(node)}\n\n`; break;
                case "h5": markdown += `##### ${getInlineContent(node)}\n\n`; break;
                case "h6": markdown += `###### ${getInlineContent(node)}\n\n`; break;
                case "p": {
                    const inner = getInlineContent(node).trim();
                    if (inner) markdown += `${inner}\n\n`;
                    break;
                }
                case "strong":
                case "b":  markdown += `**${getInlineContent(node)}**`; break;
                case "em":
                case "i":  markdown += `*${getInlineContent(node)}*`;   break;
                case "s":  markdown += `~~${getInlineContent(node)}~~`; break;
                case "a":  markdown += `[${getInlineContent(node)}](${node.getAttribute("href")})`; break;
                case "code": markdown += `\`${node.textContent}\``; break;
                case "pre": {
                    const codeEl = node.querySelector("code");
                    const langMatch = codeEl?.className?.match(/language-(\S+)/);
                    const lang = langMatch ? langMatch[1] : "";
                    const codeText = (codeEl ?? node).textContent ?? "";
                    markdown += `\`\`\`${lang}\n${codeText}\n\`\`\`\n\n`;
                    break;
                }
                case "blockquote": {
                    const inner = convertHtmlToMarkdown(node).trimEnd();
                    markdown += inner.split("\n").map((l) => `> ${l}`).join("\n") + "\n\n";
                    break;
                }
                case "ul": {
                    const isTaskList = node.getAttribute("data-type") === "taskList";
                    for (const child of node.children) {
                        if (child.tagName.toLowerCase() !== "li") continue;
                        const text = getListItemContent(child);
                        if (isTaskList) {
                            const checked = child.getAttribute("data-checked") === "true";
                            markdown += `- [${checked ? "x" : " "}] ${text}\n`;
                        } else {
                            markdown += `- ${text}\n`;
                        }
                    }
                    markdown += "\n";
                    break;
                }
                case "ol": {
                    let index = 1;
                    for (const child of node.children) {
                        if (child.tagName.toLowerCase() !== "li") continue;
                        markdown += `${index}. ${getListItemContent(child)}\n`;
                        index++;
                    }
                    markdown += "\n";
                    break;
                }
                case "table": markdown += convertTableToMarkdown(node); break;
                case "hr":   markdown += "---\n\n"; break;
                case "br":   markdown += "\n"; break;
                default:     markdown += convertHtmlToMarkdown(node);
            }
        }
        return markdown;
    }

    function onEditorUpdate() {}

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
                    <h1 class="text-3xl font-bold tracking-tight">
                        {folderName}
                    </h1>
                    <p class="text-sm text-muted-foreground font-mono mt-0.5">
                        {folderPath}
                    </p>
                </div>
            </div>

            {#if isScanning}
                <div class="flex items-center gap-2 text-sm mr-8">
                    <div
                        class="w-2 h-2 bg-primary rounded-full animate-pulse"
                    ></div>
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
                        <h3 class="font-semibold text-destructive">
                            Scan failed
                        </h3>
                        <pre
                            class="mt-2 text-sm text-muted-foreground whitespace-pre-wrap">{scanError.message ||
                                scanError}</pre>
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
                            {projectInfo.takerestInitialized ? "Initialized" : "Not Active"}
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
                        <p class="text-2xl font-bold font-mono truncate">
                            {projectInfo.git?.repoName ?? "N/A"}
                        </p>
                        <p class="text-xs text-muted-foreground mt-1">
                            Branch: <span class="font-mono">{projectInfo.git?.branch ?? "N/A"}</span>
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
                            <p class="text-2xl font-bold font-mono">
                                {projectInfo.majorFiletype?.extension ?? "N/A"}
                            </p>
                            <p class="text-lg text-muted-foreground">
                                ×{projectInfo.majorFiletype?.count ?? 0}
                            </p>
                        </div>
                        <p class="text-xs text-muted-foreground mt-1">files in project</p>
                    </div>
                </div>

                <!-- README Section -->
                <div
                    class="border rounded-lg overflow-hidden"
                    in:fly={{ y: 30, duration: 500, delay: 500, easing: quintOut }}
                >
                    {#if projectInfo.rootReadme}
                        <div class="flex items-center justify-between border-b px-6 py-3 bg-muted/30">
                            <div class="flex items-center gap-2">
                                <FileText class="w-4 h-4" />
                                <h3 class="text-sm font-semibold">README.md</h3>
                            </div>
                            <div class="flex items-center gap-2">
                                {#if readmeView === "edit"}
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
                                        {isSaving ? "Saving..." : "Save"}
                                    </button>
                                {:else}
                                    <div class="inline-flex rounded-md border bg-background p-1">
                                        <button
                                            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded transition-colors
                                                {readmeView === 'preview' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}"
                                            onclick={() => (readmeView = "preview")}
                                        >
                                            <Eye class="w-3.5 h-3.5" />
                                            Preview
                                        </button>
                                        <button
                                            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded transition-colors
                                                {readmeView === 'raw' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}"
                                            onclick={() => (readmeView = "raw")}
                                        >
                                            <Code class="w-3.5 h-3.5" />
                                            Raw
                                        </button>
                                    </div>
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

                        <div class="p-6">
                            {#if readmeView === "preview"}
                                <!-- svelte-ignore a11y_click_events_have_key_events -->
                                <!-- svelte-ignore a11y_no_static_element_interactions -->
                                <div
                                    class="prose prose-sm dark:prose-invert max-w-none"
                                    onclick={handleReadmeClick}
                                >
                                    {@html renderedReadmeHtml}
                                </div>
                            {:else if readmeView === "raw"}
                                <pre class="bg-muted border border-border rounded-2xl p-6 text-sm font-mono overflow-x-auto whitespace-pre leading-relaxed max-w-full">{projectInfo.rootReadme}</pre>
                            {:else if readmeView === "edit"}
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