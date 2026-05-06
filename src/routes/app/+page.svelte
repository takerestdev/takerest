<script>
  import { useSearchParams, createSearchParamsSchema } from "runed/kit";
  import { scanProject, initProject } from "$lib/commands/project";

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

  import { onMount } from "svelte";

  onMount(async () => {
    if (!folderPath) return;
    try {
      const info = await scanProject(folderPath);
      console.log(info);
      const status = await initProject(folderPath);
      console.log(status);
    } catch (error) {
      console.error("Project initialization/scan failed", error);
    }
  });
</script>

<div class="h-screen w-screen flex justify-center items-center">
  <h1>{folderName}</h1>
  <p>{folderPath}</p>
</div>
