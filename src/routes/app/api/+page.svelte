<script>
  // @ts-nocheck
  import { useSearchParams, createSearchParamsSchema } from "runed/kit";

  const schema = createSearchParamsSchema({
    path: { type: "string", default: "" },
  });

  const params = useSearchParams(schema, { pushHistory: false });
  let folderPath = $state(params.path);
</script>

<div class="h-full flex flex-col items-center justify-center gap-2 text-muted-foreground">
  <p class="text-lg font-semibold text-foreground">API</p>
  <p class="text-sm">{folderPath}</p>
</div>
