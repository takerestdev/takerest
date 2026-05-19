<script>
	import "./layout.css";
	import { ModeWatcher } from "mode-watcher";
	import { Toaster } from "$lib/components/ui/sonner/index.js";
	import { onMount } from "svelte";
	import { browser } from "$app/environment";
	import { checkForUpdates } from "$lib/updater";

	const { children } = $props();

	onMount(() => {
		if (!browser) return;
		// delay slightly so the app window is fully visible first
		setTimeout(checkForUpdates, 3000);
	});
</script>

<ModeWatcher />
<div class="h-full bg-transparent">
	<Toaster />
	{@render children()}
</div>
