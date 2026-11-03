<script lang="ts">
  import { searchWorkspace } from "$lib/api";
  import type { SearchHit } from "$lib/types";
  import Icon from "./Icon.svelte";

  let {
    onOpen,
  }: {
    onOpen: (path: string, line: number) => void;
  } = $props();

  let query = $state("");
  let results = $state<SearchHit[]>([]);
  let busy = $state(false);
  let error = $state("");
  let sequence = 0;

  $effect(() => {
    const value = query;
    const current = ++sequence;

    if (value.trim() === "") {
      results = [];
      busy = false;
      error = "";
      return;
    }

    busy = true;
    const timer = setTimeout(async () => {
      try {
        const hits = await searchWorkspace(value);
        if (current !== sequence) return;
        results = hits;
        error = "";
      } catch (cause) {
        if (current !== sequence) return;
        error = cause instanceof Error ? cause.message : String(cause);
        results = [];
      } finally {
        if (current === sequence) busy = false;
      }
    }, 180);

    return () => clearTimeout(timer);
  });

  function highlight(text: string, needle: string): { pre: string; hit: string; post: string } {
    const index = text.toLowerCase().indexOf(needle.toLowerCase());
    if (index === -1 || needle === "") return { pre: text, hit: "", post: "" };
    return {
      pre: text.slice(0, index),
      hit: text.slice(index, index + needle.length),
      post: text.slice(index + needle.length),
    };
  }
</script>

<div class="pb-4.5">
  <div class="mx-3 my-2 flex h-9 items-center gap-1.5 rounded-lg border border-line bg-elev px-2.5 text-faint focus-within:border-accent">
    <Icon name="search" size={14} />
    <input
      type="search"
      placeholder="Search text files"
      aria-label="Search text files"
      bind:value={query}
      spellcheck="false"
      autocomplete="off"
      class="min-w-0 flex-1 border-0 bg-transparent text-[12.5px] text-ink outline-none placeholder:text-faint"
    />
    {#if busy}
      <span class="shrink-0 text-[11px] text-faint">searching</span>
    {/if}
  </div>

  {#if error}
    <p class="m-0 px-3 py-2 text-xs leading-relaxed text-danger">{error}</p>
  {:else if query.trim() === ""}
    <p class="m-0 px-3 py-2 text-xs leading-relaxed text-faint">Type to search Markdown and code files. Results stay in this folder.</p>
  {:else if !busy && results.length === 0}
    <p class="m-0 px-3 py-2 text-xs leading-relaxed text-faint">No matches for “{query}”.</p>
  {:else}
    <p class="m-0 px-3 pt-1 pb-2 text-[11px] tabular-nums text-faint">{results.length}{results.length === 250 ? "+" : ""} {results.length === 1 ? "match" : "matches"}</p>
    <div class="px-2">
      {#each results as hit (`${hit.path}:${hit.line}`)}
        {@const parts = highlight(hit.text, query.trim())}
        <button
          type="button"
          class="my-0.5 flex w-full flex-col gap-0.5 rounded-lg px-2.5 py-2 text-left transition-colors duration-150 hover:bg-hover"
          title={hit.path}
          onclick={() => onOpen(hit.path, hit.line)}
        >
          <span class="flex min-w-0 items-baseline gap-1.5">
            <span class="truncate font-mono text-[11px] text-muted">{hit.path}</span>
            <span class="ml-auto shrink-0 font-mono text-[10px] text-faint">{hit.line}</span>
          </span>
          <span class="truncate text-[11.5px] leading-snug text-faint">
            {parts.pre}<mark class="rounded-[2px] bg-active px-px text-ink">{parts.hit}</mark>{parts.post}
          </span>
        </button>
      {/each}
    </div>
  {/if}
</div>
