<script>
  import { useSearchParams, createSearchParamsSchema } from "runed/kit";

  const schema = createSearchParamsSchema({
    path: { type: "string", default: "" },
  });

  const params = useSearchParams(schema, {
    pushHistory: false,
  });

  $: folderPath = params.path;
  $: folderName = folderPath.split(/[\\/]/).filter(Boolean).pop() ?? "Project";
</script>

<div>
  <h1>{folderName}</h1>
  <p>{folderPath}</p>
</div>