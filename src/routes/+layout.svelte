<script>
	import './layout.css';

	const { children } = $props();
</script>

<div class="h-full">
	{@render children()}
</div>
