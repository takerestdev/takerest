<script>
  let { class: className = "" } = $props();
  import { mode } from "mode-watcher";
</script>

{#if mode.current == "dark"}
  <img
    src="/icon-dark.png"
    alt="Anide"
    draggable="false"
    class="object-contain select-none {className || 'size-7'}"
  />
{:else}
  <img
    src="/icon.png"
    alt="Anide"
    draggable="false"
    class="object-contain select-none {className || 'size-7'}"
  />
{/if}
