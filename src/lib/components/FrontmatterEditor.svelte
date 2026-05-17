<script>
  // @ts-nocheck
  /**
   * Recursive YAML frontmatter editor.
   * Handles scalars, nested objects, and arrays (of scalars or objects).
   * Props:
   *   value    : string  — raw YAML text (no --- delimiters)
   *   onchange : (yaml: string) => void
   */
  let { value = '', onchange } = $props();

  import { parse as yamlParse, stringify as yamlStringify } from 'yaml';
  import { Input } from '$lib/components/ui/input/index.js';
  import { ChevronDown, ChevronRight, Plus, X } from '@lucide/svelte';

  // ── Parse / serialize ────────────────────────────────────────────────────

  function parseYaml(text) {
    if (!text?.trim()) return {};
    try {
      const p = yamlParse(text);
      return (p && typeof p === 'object' && !Array.isArray(p)) ? p : {};
    } catch { return {}; }
  }

  function toYaml(obj) {
    if (!obj || Object.keys(obj).length === 0) return '';
    try { return yamlStringify(obj, { lineWidth: 0 }).trimEnd(); } catch { return ''; }
  }

  let data = $state({});
  let _lastYaml = '';

  $effect(() => {
    if (value === _lastYaml) return;
    data = parseYaml(value);
  });

  function emit() {
    const yaml = toYaml(data);
    _lastYaml = yaml;
    onchange?.(yaml);
  }

  // ── Helpers ──────────────────────────────────────────────────────────────

  // Coerce a string input back to the appropriate JS primitive
  function coerce(str) {
    if (str === 'true') return true;
    if (str === 'false') return false;
    if (str === 'null' || str === '') return '';
    const n = Number(str);
    return (!isNaN(n) && str.trim() !== '') ? n : str;
  }

  // Format a scalar for display inside an input
  function display(v) {
    if (v === null || v === undefined) return '';
    if (typeof v !== 'object') return String(v);
    return '';
  }

  // Category of a value
  function kind(v) {
    if (v === null || v === undefined) return 'scalar';
    if (Array.isArray(v)) return 'array';
    if (typeof v === 'object') return 'object';
    return 'scalar';
  }

  function isLong(v) {
    return typeof v === 'string' && (v.length > 55 || v.includes('\n'));
  }

  // ── Object mutations ─────────────────────────────────────────────────────

  // Rename a key in a plain object while preserving order
  function renameObjKey(obj, oldKey, newKey) {
    if (!newKey.trim() || oldKey === newKey) return obj;
    const result = {};
    for (const [k, v] of Object.entries(obj)) result[k === oldKey ? newKey : k] = v;
    return result;
  }

  // ── Collapse state ───────────────────────────────────────────────────────

  let open = $state({});   // path string → boolean (true = expanded)

  function toggleOpen(pathKey) {
    open[pathKey] = !(open[pathKey] ?? true);
  }

  function isOpen(pathKey) {
    return open[pathKey] ?? true; // expanded by default
  }

  // ── Textarea auto-resize ─────────────────────────────────────────────────

  function autoResize(el) {
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 120) + 'px';
  }

  function bindResize(el) {
    if (el) { setTimeout(() => autoResize(el), 0); }
    return { destroy() {} };
  }
</script>

<!-- ── Root entries ────────────────────────────────────────────────────── -->
<div class="space-y-1">

  {#each Object.entries(data) as [key, val], i (key + i)}
    {@const k = kind(val)}
    {@const pathKey = key}

    <div>
      <!-- Header row: key | value-or-toggle | remove -->
      <div class="grid items-start gap-2 mb-0.5" style="grid-template-columns: 140px 1fr auto">

        <!-- Key -->
        <Input
          value={key}
          onchange={(e) => {
            data = renameObjKey(data, key, e.target.value.trim() || key);
            emit();
          }}
          class="h-7 text-xs font-mono"
          placeholder="key"
        />

        <!-- Value -->
        {#if k === 'scalar'}
          {#if isLong(val)}
            <textarea
              value={display(val)}
              oninput={(e) => {
                autoResize(e.currentTarget);
                data = { ...data, [key]: e.currentTarget.value };
                emit();
              }}
              use:bindResize
              rows="1"
              spellcheck="false"
              class="w-full font-mono text-xs bg-transparent border border-input rounded-md px-3 py-1.5 outline-none resize-none leading-relaxed focus:ring-1 focus:ring-ring overflow-hidden"
            ></textarea>
          {:else}
            <Input
              value={display(val)}
              oninput={(e) => {
                data = { ...data, [key]: coerce(e.currentTarget.value) };
                emit();
              }}
              class="h-7 text-xs font-mono"
              placeholder="value"
            />
          {/if}

        {:else}
          <!-- Complex value: show toggle badge -->
          <button
            type="button"
            onclick={() => toggleOpen(pathKey)}
            class="flex items-center gap-1.5 h-7 px-2 rounded-md border border-input bg-muted/30 hover:bg-muted/60 transition-colors text-xs font-mono text-muted-foreground w-full text-left"
          >
            {#if isOpen(pathKey)}<ChevronDown size={11} class="shrink-0" />{:else}<ChevronRight size={11} class="shrink-0" />{/if}
            {#if k === 'array'}
              <span class="text-muted-foreground/70">[ {val.length} item{val.length !== 1 ? 's' : ''} ]</span>
            {:else}
              <span class="text-muted-foreground/70">{"{"} {Object.keys(val).length} key{Object.keys(val).length !== 1 ? 's' : ''} {"}"}</span>
            {/if}
          </button>
        {/if}

        <!-- Remove top-level key -->
        <button
          type="button"
          onclick={() => {
            const { [key]: _, ...rest } = data;
            data = rest;
            emit();
          }}
          class="w-7 h-7 flex items-center justify-center rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors shrink-0"
        >
          <X size={13} />
        </button>
      </div>

      <!-- Expanded content for objects -->
      {#if k === 'object' && isOpen(pathKey)}
        <div class="ml-4 pl-3 border-l border-border/50 mt-1 mb-1 space-y-1">
          {#each Object.entries(val) as [subKey, subVal], si (subKey + si)}
            <div class="grid items-start gap-2" style="grid-template-columns: 120px 1fr auto">
              <Input
                value={subKey}
                onchange={(e) => {
                  const newSub = e.target.value.trim() || subKey;
                  data = { ...data, [key]: renameObjKey(val, subKey, newSub) };
                  emit();
                }}
                class="h-7 text-xs font-mono"
              />
              {#if isLong(subVal)}
                <textarea
                  value={display(subVal)}
                  oninput={(e) => {
                    autoResize(e.currentTarget);
                    data = { ...data, [key]: { ...val, [subKey]: e.currentTarget.value } };
                    emit();
                  }}
                  use:bindResize
                  rows="1"
                  spellcheck="false"
                  class="w-full font-mono text-xs bg-transparent border border-input rounded-md px-3 py-1.5 outline-none resize-none leading-relaxed focus:ring-1 focus:ring-ring overflow-hidden"
                ></textarea>
              {:else}
                <Input
                  value={display(subVal)}
                  oninput={(e) => {
                    data = { ...data, [key]: { ...val, [subKey]: coerce(e.currentTarget.value) } };
                    emit();
                  }}
                  class="h-7 text-xs font-mono"
                />
              {/if}
              <button
                type="button"
                onclick={() => {
                  const { [subKey]: _, ...rest } = val;
                  data = { ...data, [key]: rest };
                  emit();
                }}
                class="w-7 h-7 flex items-center justify-center rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors shrink-0"
              ><X size={13} /></button>
            </div>
          {/each}
          <button
            type="button"
            onclick={() => {
              const cur = data[key] || {};
              let n = 1; while (`key${n}` in cur) n++;
              data = { ...data, [key]: { ...cur, [`key${n}`]: '' } };
              emit();
            }}
            class="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors px-1 mt-0.5"
          ><Plus size={11} />add field</button>
        </div>
      {/if}

      <!-- Expanded content for arrays -->
      {#if k === 'array' && isOpen(pathKey)}
        <div class="ml-4 pl-3 border-l border-border/50 mt-1 mb-1 space-y-1.5">
          {#each val as item, idx (idx)}
            {@const itemKind = kind(item)}

            {#if itemKind === 'object'}
              <!-- Array item is an object -->
              <div class="border border-border/40 rounded-md px-2 py-1.5 space-y-1 bg-muted/10">
                <div class="flex items-center justify-between mb-1">
                  <span class="text-[10px] text-muted-foreground/60 font-medium">item {idx}</span>
                  <button
                    type="button"
                    onclick={() => {
                      data = { ...data, [key]: val.filter((_, i) => i !== idx) };
                      emit();
                    }}
                    class="w-5 h-5 flex items-center justify-center rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
                  ><X size={11} /></button>
                </div>
                {#each Object.entries(item) as [subKey, subVal], si (subKey + si)}
                  <div class="grid items-start gap-2" style="grid-template-columns: 100px 1fr auto">
                    <Input
                      value={subKey}
                      onchange={(e) => {
                        const newArr = [...val];
                        newArr[idx] = renameObjKey(item, subKey, e.target.value.trim() || subKey);
                        data = { ...data, [key]: newArr };
                        emit();
                      }}
                      class="h-6 text-[11px] font-mono"
                    />
                    {#if isLong(subVal)}
                      <textarea
                        value={display(subVal)}
                        oninput={(e) => {
                          autoResize(e.currentTarget);
                          const newArr = [...val];
                          newArr[idx] = { ...item, [subKey]: e.currentTarget.value };
                          data = { ...data, [key]: newArr };
                          emit();
                        }}
                        use:bindResize
                        rows="1"
                        spellcheck="false"
                        class="w-full font-mono text-[11px] bg-transparent border border-input rounded-md px-2 py-1 outline-none resize-none leading-relaxed focus:ring-1 focus:ring-ring overflow-hidden"
                      ></textarea>
                    {:else}
                      <Input
                        value={display(subVal)}
                        oninput={(e) => {
                          const newArr = [...val];
                          newArr[idx] = { ...item, [subKey]: coerce(e.currentTarget.value) };
                          data = { ...data, [key]: newArr };
                          emit();
                        }}
                        class="h-6 text-[11px] font-mono"
                      />
                    {/if}
                    <button
                      type="button"
                      onclick={() => {
                        const newArr = [...val];
                        const { [subKey]: _, ...rest } = item;
                        newArr[idx] = rest;
                        data = { ...data, [key]: newArr };
                        emit();
                      }}
                      class="w-6 h-6 flex items-center justify-center rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors shrink-0"
                    ><X size={11} /></button>
                  </div>
                {/each}
                <button
                  type="button"
                  onclick={() => {
                    const newArr = [...val];
                    const cur = item || {};
                    let n = 1; while (`key${n}` in cur) n++;
                    newArr[idx] = { ...cur, [`key${n}`]: '' };
                    data = { ...data, [key]: newArr };
                    emit();
                  }}
                  class="flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground transition-colors px-1 mt-0.5"
                ><Plus size={10} />add field</button>
              </div>

            {:else}
              <!-- Array item is a scalar -->
              <div class="flex items-center gap-2">
                <span class="text-[10px] text-muted-foreground/50 w-6 text-right shrink-0">{idx}</span>
                <Input
                  value={display(item)}
                  oninput={(e) => {
                    const newArr = [...val];
                    newArr[idx] = coerce(e.currentTarget.value);
                    data = { ...data, [key]: newArr };
                    emit();
                  }}
                  class="h-7 text-xs font-mono flex-1"
                />
                <button
                  type="button"
                  onclick={() => {
                    data = { ...data, [key]: val.filter((_, i) => i !== idx) };
                    emit();
                  }}
                  class="w-7 h-7 flex items-center justify-center rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors shrink-0"
                ><X size={13} /></button>
              </div>
            {/if}
          {/each}

          <!-- Add array item -->
          <button
            type="button"
            onclick={() => {
              const newItem = val.length > 0 && kind(val[0]) === 'object' ? {} : '';
              data = { ...data, [key]: [...val, newItem] };
              emit();
            }}
            class="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors px-1 mt-0.5"
          ><Plus size={11} />add item</button>
        </div>
      {/if}

    </div>
  {/each}

  <!-- Add top-level field — choose type -->
  <div class="flex items-center gap-1.5 mt-2 pt-2 border-t border-border/40">
    <span class="text-[10px] text-muted-foreground/60 shrink-0">Add field:</span>
    <button
      type="button"
      onclick={() => {
        let n = 1; while (`key${n}` in data) n++;
        data = { ...data, [`key${n}`]: '' };
        emit();
      }}
      class="flex items-center gap-1 px-2 py-0.5 rounded border border-border/60 text-[11px] text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors"
    ><Plus size={10} />string</button>
    <button
      type="button"
      onclick={() => {
        let n = 1; while (`key${n}` in data) n++;
        data = { ...data, [`key${n}`]: {} };
        emit();
      }}
      class="flex items-center gap-1 px-2 py-0.5 rounded border border-border/60 text-[11px] text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors font-mono"
    ><Plus size={10} />{"{ }"}</button>
    <button
      type="button"
      onclick={() => {
        let n = 1; while (`key${n}` in data) n++;
        data = { ...data, [`key${n}`]: [] };
        emit();
      }}
      class="flex items-center gap-1 px-2 py-0.5 rounded border border-border/60 text-[11px] text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors font-mono"
    ><Plus size={10} />{"[ ]"}</button>
  </div>
</div>
