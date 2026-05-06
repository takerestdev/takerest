<script>
  import { useSearchParams, createSearchParamsSchema } from "runed/kit";
  import { scanProject, initProject } from "$lib/commands/project";

  const schema = createSearchParamsSchema({
    path: { type: "string", default: "" },
  });

  const params = useSearchParams(schema, {
    pushHistory: false,
  });

  let folderPath = $state(params.path);
  let folderName = $state(
    folderPath.split(/[\\/]/).filter(Boolean).pop() ?? "Project",
  );

  import { onMount } from "svelte";

  onMount(async () => {
    const info = await scanProject(folderPath);
    console.log(info);
    const status = await initProject(folderPath);
    console.log(status);
  });
</script>

<div class="h-screen w-screen flex justify-center items-center">
  <h1>{folderName}</h1>
  <p>{folderPath}</p>
</div>
