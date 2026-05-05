<script>
	import "./layout.css";
	import { ModeWatcher } from "mode-watcher";
	import { Toaster } from "$lib/components/ui/sonner/index.js";
	const { children } = $props();
</script>

<ModeWatcher />
<div class="h-full bg-transparent">
	<Toaster />
	{@render children()}
</div>
