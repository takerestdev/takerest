<script>
  // @ts-nocheck
  let { data, tabId } = $props();

  import { readProjectFile, writeProjectFile } from '$lib/commands/files.js';
  import { workspace } from '$lib/stores/workspace.svelte.js';
  import FileEditor from '$lib/components/FileEditor.svelte';

  let projectPath = $derived(data.projectPath);
  let relPath     = $derived(data.relPath);
  let language    = $derived(data.language);
  let title       = $derived(relPath ?? '');

  const load = () => readProjectFile(projectPath, relPath);
  const save = (content) => writeProjectFile(projectPath, relPath, content);
</script>

<FileEditor
  {title}
  {load}
  {save}
  language={language}
  onDirtyChange={(d) => workspace.setTabDirty(tabId, d)}
/>
