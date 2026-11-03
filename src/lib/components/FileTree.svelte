<script lang="ts">
  import { buildTree, flattenTree } from "$lib/tree";
  import type { TreeEntry } from "$lib/types";
  import FileGlyph from "./FileGlyph.svelte";
  import Icon from "./Icon.svelte";

  let {
    entries,
    collapsed,
    activePath,
    selectedDirectory,
    createKind,
    createParent,
    createName,
    createBusy,
    renamePath,
    renameName,
    renameBusy,
    onOpen,
    onToggleDir,
    onCreateName,
    onCreate,
    onCancelCreate,
    onRenameName,
    onRename,
    onCancelRename,
  }: {
    entries: TreeEntry[];
    collapsed: Set<string>;
    activePath: string | null;
    selectedDirectory: string | null;
    createKind: "file" | "folder" | null;
    createParent: string | null;
    createName: string;
    createBusy: boolean;
    renamePath: string | null;
    renameName: string;
    renameBusy: boolean;
    onOpen: (path: string) => void;
    onToggleDir: (path: string) => void;
    onCreateName: (name: string) => void;
    onCreate: (kind: "file" | "folder", parent: string | null, name: string) => void;
    onCancelCreate: () => void;
    onRenameName: (name: string) => void;
    onRename: (path: string, name: string) => void;
    onCancelRename: () => void;
  } = $props();

  const rows = $derived(flattenTree(buildTree(entries), collapsed));

  function focusInput(node: HTMLInputElement) {
    const frame = requestAnimationFrame(() => {
      node.focus();
      node.select();
    });
    return { destroy: () => cancelAnimationFrame(frame) };
  }

</script>

{#snippet guides(depth: number)}
  {@const guideDepths = Array.from({ length: depth }, (_, index) => index)}
  {#each guideDepths as guideDepth (guideDepth)}
    <span
      class="pointer-events-none absolute inset-y-0 z-0 w-px bg-line/70"
      style:left="{14 + guideDepth * 14}px"
      aria-hidden="true"
    ></span>
  {/each}
  {#if depth > 0}
    <span
      class="pointer-events-none absolute top-1/2 z-0 h-px -translate-y-1/2 bg-line/70"
      style="left: {14 + (depth - 1) * 14}px; width: 8px"
      aria-hidden="true"
    ></span>
  {/if}
{/snippet}

{#snippet createRow(depth: number, parent: string | null)}
  <form
    class="relative mx-2 flex h-[31px] w-[calc(100%-16px)] items-center gap-1 rounded-md border border-accent bg-elev pr-1 text-[13px] text-ink"
    style="padding-left: {8 + depth * 14}px"
    aria-label={createKind === "folder" ? "Create folder" : "Create file"}
    onsubmit={(event) => {
      event.preventDefault();
      if (createKind) onCreate(createKind, parent, createName);
    }}
  >
    {@render guides(depth)}
    <span class="relative z-10 flex size-[15px] shrink-0 items-center justify-center text-faint">
      <Icon name={createKind === "folder" ? "folder" : "file"} size={14} />
    </span>
    <input
      class="relative z-10 h-6 min-w-0 flex-1 border-0 bg-transparent text-[12px] text-ink outline-none placeholder:text-faint"
      value={createName}
      aria-label={createKind === "folder" ? "New folder name" : "New file name"}
      placeholder="Name or relative path"
      use:focusInput
      spellcheck="false"
      autocomplete="off"
      oninput={(event) => onCreateName(event.currentTarget.value)}
      onkeydown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          onCancelCreate();
        }
      }}
    />
    <button
      class="relative z-10 grid size-6 shrink-0 place-items-center rounded text-muted hover:bg-hover hover:text-ink disabled:opacity-40"
      type="submit"
      title="Create"
      aria-label="Create"
      disabled={createBusy || createName.trim() === ""}
    ><Icon name="check" size={13} /></button>
    <button
      class="relative z-10 grid size-6 shrink-0 place-items-center rounded text-faint hover:bg-hover hover:text-ink"
      type="button"
      title="Cancel"
      aria-label="Cancel"
      disabled={createBusy}
      onclick={onCancelCreate}
    ><Icon name="close" size={12} /></button>
  </form>
{/snippet}

<div class="py-1 pb-4" role="tree" aria-label="Workspace files">
  {#if createKind && createParent === null}
    {@render createRow(0, null)}
  {/if}
  {#if rows.length === 0 && !createKind}
    <p class="m-0 p-4 text-xs leading-relaxed text-faint">No text files in this folder.</p>
  {/if}
  {#each rows as row (row.path)}
    {@const active = (!row.isDir && row.path === activePath) || (row.isDir && row.path === selectedDirectory)}
    {#if renamePath === row.path}
      <form
        class="relative mx-2 flex h-[31px] w-[calc(100%-16px)] items-center gap-2 rounded-md border border-accent bg-elev pr-1 text-[13px] text-ink"
        style="padding-left: {8 + row.depth * 14}px"
        aria-label="Rename {row.name}"
        onsubmit={(event) => {
          event.preventDefault();
          onRename(row.path, renameName);
        }}
      >
        {@render guides(row.depth)}
        <span class="relative z-10 w-3.5 shrink-0"></span>
        <span class="relative z-10 flex size-[15px] shrink-0 items-center justify-center text-faint">
          {#if row.isDir}<Icon name="folder" size={15} />{:else}<FileGlyph path={row.path} size={15} />{/if}
        </span>
        <input
          class="relative z-10 h-6 min-w-0 flex-1 border-0 bg-transparent text-[12px] text-ink outline-none"
          value={renameName}
          aria-label="Rename {row.name}"
          use:focusInput
          spellcheck="false"
          autocomplete="off"
          disabled={renameBusy}
          oninput={(event) => onRenameName(event.currentTarget.value)}
          onkeydown={(event) => {
            if (event.key === "Escape") {
              event.preventDefault();
              onCancelRename();
            }
            }}
        />
        <button
          class="relative z-10 grid size-6 shrink-0 place-items-center rounded text-muted hover:bg-hover hover:text-ink disabled:opacity-40"
          type="submit"
          title="Rename"
          aria-label="Rename"
          disabled={renameBusy || renameName.trim() === ""}
        ><Icon name="check" size={13} /></button>
        <button
          class="relative z-10 grid size-6 shrink-0 place-items-center rounded text-faint hover:bg-hover hover:text-ink"
          type="button"
          title="Cancel"
          aria-label="Cancel rename"
          disabled={renameBusy}
          onclick={onCancelRename}
        ><Icon name="close" size={12} /></button>
      </form>
    {:else}
      <button
        type="button"
        class="relative mx-2 flex h-[31px] w-[calc(100%-16px)] items-center gap-2 rounded-md pr-2 text-left text-[13px] transition-colors duration-100 {active
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
        {@render guides(row.depth)}
        {#if row.isDir}
          <span class="relative z-10 flex w-3.5 shrink-0 items-center justify-center text-faint">
            <span class={collapsed.has(row.path) ? "flex [&_svg]:rotate-0" : "flex [&_svg]:rotate-90"}>
              <Icon name="chevron" size={13} />
            </span>
          </span>
          <span class="relative z-10 flex shrink-0 items-center text-muted"><Icon name="folder" size={15} /></span>
        {:else}
          <span class="relative z-10 w-3.5 shrink-0"></span>
          <span class="relative z-10 flex size-[15px] shrink-0 items-center justify-center {active ? 'text-ink' : 'text-faint'}">
            <FileGlyph path={row.path} size={15} />
          </span>
        {/if}
        <span class="relative z-10 truncate">{row.name}</span>
      </button>
    {/if}
    {#if createKind && createParent === row.path}
      {@render createRow(row.depth + 1, row.path)}
    {/if}
  {/each}
</div>
