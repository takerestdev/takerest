<script>
  let { class: className = "" } = $props();
</script>

<img
  src="/icon.png"
  alt="Anide"
  draggable="false"
  class="object-contain select-none {className || 'size-7'}"
/>
