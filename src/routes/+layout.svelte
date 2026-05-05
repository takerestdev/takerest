<script>
	import './layout.css';
	import { ModeWatcher } from "mode-watcher";
	const { children } = $props();
</script>

<ModeWatcher />
<div class="h-full">
	{@render children()}
</div>
