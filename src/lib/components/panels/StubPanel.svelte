<script>
  // @ts-nocheck
  let { tool } = $props();

  import { Database, Braces, Box, GitBranch, Container } from '@lucide/svelte';

  const icons = { db: Database, kv: Braces, s3: Box, git: GitBranch, docker: Container };
  const labels = { db: 'Database', kv: 'Cache', s3: 'Storage', git: 'Git', docker: 'Docker' };
</script>

<div class="h-full flex flex-col items-center justify-center gap-2 text-muted-foreground p-4">
  <svelte:component this={icons[tool]} class="w-8 h-8 opacity-30" />
  <p class="text-xs text-center">{labels[tool] ?? tool}<br />coming soon</p>
</div>
