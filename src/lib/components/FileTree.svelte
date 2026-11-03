<script lang="ts">
  import { buildTree, flattenTree } from "$lib/tree";
  import type { TreeEntry } from "$lib/types";
  import FileGlyph from "./FileGlyph.svelte";
  import Icon from "./Icon.svelte";

  let {
    entries,
    collapsed,
    activePath,
    onOpen,
    onToggleDir,
  }: {
    entries: TreeEntry[];
    collapsed: Set<string>;
    activePath: string | null;
    onOpen: (path: string) => void;
    onToggleDir: (path: string) => void;
  } = $props();

  const rows = $derived(flattenTree(buildTree(entries), collapsed));

</script>

{#if rows.length === 0}
  <p class="m-0 p-4 text-xs leading-relaxed text-faint">No text files in this folder.</p>
{:else}
  <div class="py-1 pb-4" role="tree" aria-label="Workspace files">
    {#each rows as row (row.path)}
      {@const active = !row.isDir && row.path === activePath}
      <button
        type="button"
        class="mx-2 flex h-[31px] w-[calc(100%-16px)] items-center gap-2 rounded-md pr-2 text-left text-[13px] transition-colors duration-100 {active
          ? 'bg-active text-ink'
          : row.isDir
            ? 'font-medium text-ink hover:bg-hover'
            : 'text-muted hover:bg-hover hover:text-ink'}"
        style="padding-left: {8 + row.depth * 14}px"
        role="treeitem"
        aria-level={row.depth + 1}
        aria-expanded={row.isDir ? !collapsed.has(row.path) : undefined}
        aria-selected={active}
        title={row.path}
        data-context-path={row.path}
        data-context-kind={row.isDir ? "directory" : "file"}
        onclick={() => (row.isDir ? onToggleDir(row.path) : onOpen(row.path))}
      >
        {#if row.isDir}
          <span class="flex w-3.5 shrink-0 items-center justify-center text-faint">
            <span class={collapsed.has(row.path) ? "flex [&_svg]:rotate-0" : "flex [&_svg]:rotate-90"}>
              <Icon name="chevron" size={13} />
            </span>
          </span>
          <span class="flex shrink-0 items-center text-muted"><Icon name="folder" size={15} /></span>
        {:else}
          <span class="w-3.5 shrink-0"></span>
          <span class="flex size-[15px] shrink-0 items-center justify-center {active ? 'text-ink' : 'text-faint'}">
            <FileGlyph path={row.path} size={15} />
          </span>
        {/if}
        <span class="truncate">{row.name}</span>
      </button>
    {/each}
  </div>
{/if}
