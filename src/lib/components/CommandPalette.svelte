<script lang="ts">
  import { onMount } from "svelte";
  import type { PaletteCommand, PaletteFile } from "$lib/command";
  import FileGlyph from "./FileGlyph.svelte";
  import Icon from "./Icon.svelte";

  let {
    files,
    commands,
    onClose,
    onOpen,
  }: {
    files: PaletteFile[];
    commands: PaletteCommand[];
    onClose: () => void;
    onOpen: (path: string) => void;
  } = $props();

  type Row =
    | { kind: "command"; command: PaletteCommand }
    | { kind: "file"; file: PaletteFile };

  let query = $state("");
  let selected = $state(0);
  let input: HTMLInputElement | undefined;
  let results: HTMLDivElement | undefined;

  const needle = $derived(query.trim().toLowerCase());
  const matchedCommands = $derived(
    commands.filter((command) => {
      const text = `${command.label} ${command.keywords ?? ""}`.toLowerCase();
      return needle === "" || text.includes(needle);
    }),
  );
  const matchedFiles = $derived(
    files
      .filter((file) => needle === "" || file.path.toLowerCase().includes(needle))
      .slice(0, 12),
  );
  const rows = $derived<Row[]>([
    ...matchedCommands.map((command) => ({ kind: "command" as const, command })),
    ...matchedFiles.map((file) => ({ kind: "file" as const, file })),
  ]);

  $effect(() => {
    query;
    selected = 0;
  });

  onMount(() => {
    input?.focus();
    if (results) results.scrollTop = 0;
  });

  function move(delta: number) {
    if (rows.length === 0) return;
    let next = selected;
    for (let step = 0; step < rows.length; step += 1) {
      next = (next + delta + rows.length) % rows.length;
      const row = rows[next];
      if (row.kind === "file" || !row.command.disabled) {
        selected = next;
        return;
      }
    }
  }

  function choose(row: Row | undefined) {
    if (!row) return;
    if (row.kind === "command") {
      if (row.command.disabled) return;
      onClose();
      row.command.run();
      return;
    }
    onClose();
    onOpen(row.file.path);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      move(1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      move(-1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      choose(rows[selected]);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<button
  class="palette-backdrop fixed inset-0 z-50 cursor-default bg-black/55"
  type="button"
  aria-label="Close command palette"
  onclick={onClose}
></button>

<div class="pointer-events-none fixed inset-x-0 top-[12vh] z-50 flex justify-center px-4">
  <div
    class="palette-panel pointer-events-auto w-full max-w-[560px] overflow-hidden rounded-xl border border-line-strong bg-panel shadow-[0_24px_70px_rgba(0,0,0,0.55)]"
    role="dialog"
    aria-modal="true"
    aria-label="Command palette"
  >
    <div class="flex h-12 items-center gap-2.5 border-b border-line px-4">
      <span class="text-faint"><Icon name="search" size={17} /></span>
      <input
        bind:this={input}
        bind:value={query}
        class="palette-input min-w-0 flex-1 border-0 bg-transparent text-[13.5px] text-ink outline-none placeholder:text-faint"
        type="text"
        role="combobox"
        aria-label="Search files and commands"
        aria-expanded="true"
        aria-controls="palette-results"
        aria-autocomplete="list"
        aria-activedescendant={rows[selected] ? `palette-row-${selected}` : undefined}
        placeholder="Search files or run a command"
        spellcheck="false"
        autocomplete="off"
      />
      <kbd class="rounded border border-line bg-elev px-1.5 py-0.5 font-mono text-[10px] text-faint">Esc</kbd>
    </div>

    <div bind:this={results} id="palette-results" class="max-h-[min(420px,54vh)] overflow-y-auto p-1.5" role="listbox">
      {#if rows.length === 0}
        <div class="px-3 py-9 text-center">
          <p class="m-0 text-[13px] font-medium text-muted">No results for “{query}”</p>
          <p class="mt-1.5 text-xs text-faint">Try a file name, a folder, or a command.</p>
        </div>
      {:else}
        {#each rows as row, index (row.kind === "command" ? row.command.id : row.file.path)}
          {@const active = index === selected}
          {@const disabled = row.kind === "command" && row.command.disabled}
          {#if index === 0 || rows[index - 1].kind !== row.kind}
            <p class="m-0 px-2.5 pt-2 pb-1 text-[10px] font-semibold tracking-[0.02em] text-faint">
              {row.kind === "command" ? "Commands" : "Files"}
            </p>
          {/if}
          <button
            id="palette-row-{index}"
            class="flex min-h-9 w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left transition-colors duration-100 {active
              ? 'bg-active text-ink'
              : 'text-muted hover:bg-hover hover:text-ink'} {disabled ? 'cursor-not-allowed opacity-40' : ''}"
            type="button"
            role="option"
            aria-selected={active}
            aria-disabled={disabled}
            disabled={disabled}
            onclick={() => choose(row)}
          >
            {#if row.kind === "command"}
              <span class="flex size-5 shrink-0 items-center justify-center text-faint">
                <Icon name="command" size={14} />
              </span>
              <span class="min-w-0 flex-1 truncate text-[13px]">{row.command.label}</span>
              {#if row.command.hint}
                <span class="shrink-0 font-mono text-[10px] text-faint">{row.command.hint}</span>
              {/if}
            {:else}
              <span class="flex size-5 shrink-0 items-center justify-center text-faint">
                <FileGlyph path={row.file.path} size={15} />
              </span>
              <span class="min-w-0 flex-1 truncate text-[13px] text-ink">{row.file.name}</span>
              <span class="max-w-[45%] shrink-0 truncate font-mono text-[10px] text-faint">
                {row.file.path}
              </span>
            {/if}
          </button>
        {/each}
      {/if}
    </div>

    <div class="flex h-9 items-center gap-3 border-t border-line px-4 text-[10.5px] text-faint">
      <span><kbd class="font-mono">↑↓</kbd> navigate</span>
      <span><kbd class="font-mono">↵</kbd> open</span>
      <span class="ml-auto"><kbd class="font-mono">Ctrl K</kbd> commands</span>
    </div>
  </div>
</div>
