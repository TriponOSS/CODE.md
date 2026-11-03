<script lang="ts">
  import type { OutlineHeading } from "$lib/outline";

  let {
    headings,
    hasDocument,
    onJump,
  }: {
    headings: OutlineHeading[];
    hasDocument: boolean;
    onJump: (line: number) => void;
  } = $props();
</script>

{#if !hasDocument}
  <div class="px-3.5 py-5">
    <p class="m-0 mb-1 text-[13px] font-medium text-ink">No document open</p>
    <p class="m-0 text-xs leading-relaxed text-faint">Open a Markdown file to see its headings here.</p>
  </div>
{:else if headings.length === 0}
  <div class="px-3.5 py-5">
    <p class="m-0 mb-1 text-[13px] font-medium text-ink">No headings</p>
    <p class="m-0 text-xs leading-relaxed text-faint">
      Add <code class="font-mono text-accent">#</code> headings to build an outline for this document.
    </p>
  </div>
{:else}
  <div class="py-1 pb-4">
    {#each headings as heading (`${heading.line}:${heading.text}`)}
      <button
        type="button"
        class="mx-2 my-0.5 flex min-h-[30px] w-[calc(100%-16px)] items-baseline gap-2 rounded-lg pr-2 text-left text-[12.5px] leading-snug text-muted transition-colors duration-150 hover:bg-hover hover:text-ink"
        style="padding-left: {12 + (heading.level - 1) * 12}px"
        title="Line {heading.line}"
        onclick={() => onJump(heading.line)}
      >
        <span class="min-w-0 flex-1 truncate">{heading.text}</span>
        <span class="shrink-0 font-mono text-[11px] tabular-nums text-faint">{heading.line}</span>
      </button>
    {/each}
  </div>
{/if}
