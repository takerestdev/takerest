<script>
  // @ts-nocheck

  import SunIcon from "@lucide/svelte/icons/sun";
  import MoonIcon from "@lucide/svelte/icons/moon";

  import { resetMode, setMode } from "mode-watcher";
  import { toast } from "svelte-sonner";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { buttonVariants } from "$lib/components/ui/button/index.js";

  let { ghost = false } = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class={buttonVariants({
      variant: ghost ? "ghost" : "outline",
      size: "icon",
    })}
  >
    <SunIcon
      class="h-[1.2rem] w-[1.2rem] scale-100 rotate-0 transition-all! dark:scale-0 dark:-rotate-90"
    />
    <MoonIcon
      class="absolute h-[1.2rem] w-[1.2rem] scale-0 rotate-90 transition-all! dark:scale-100 dark:rotate-0"
    />
    <span class="sr-only">Toggle theme</span>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end">
    <DropdownMenu.Item
      onclick={() => {
        setMode("light");
        toast("☀️ Light mode", { description: "Switched to light theme." });
      }}
    >
      Light
    </DropdownMenu.Item>
    <DropdownMenu.Item
      onclick={() => {
        setMode("dark");
        toast("🌙 Dark mode", { description: "Switched to dark theme." });
      }}
    >
      Dark
    </DropdownMenu.Item>
    <DropdownMenu.Item
      onclick={() => {
        resetMode();
        toast("💻 System mode", {
          description: "Following your system preference.",
        });
      }}
    >
      System
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>
