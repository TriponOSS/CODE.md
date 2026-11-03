<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { isMarkdownPath } from "$lib/files";
  import { renderMarkdown } from "$lib/markdown";

  let { content, path }: { content: string; path: string } = $props();

  const markdownMode = $derived(isMarkdownPath(path));
  const html = $derived(markdownMode ? renderMarkdown(content) : "");
  let container: HTMLDivElement | undefined;

  onMount(() => {
    const handleClick = (event: MouseEvent) => {
      const anchor = (event.target as HTMLElement | null)?.closest("a");
      const href = anchor?.getAttribute("href");
      if (!href) return;
      event.preventDefault();
      if (/^https?:\/\//i.test(href) || href.startsWith("mailto:")) {
        void openUrl(href).catch(() => {
          // External links are best-effort; never navigate the app window.
        });
      }
    };

    container?.addEventListener("click", handleClick);
    return () => container?.removeEventListener("click", handleClick);
  });
</script>

<div class="h-full overflow-auto bg-editor" bind:this={container} role="document">
  {#if content.trim() === ""}
    <p class="mx-auto max-w-[720px] px-8 py-10 text-faint">This file is empty.</p>
  {:else if markdownMode}
    <article class="prose mx-auto max-w-[720px] px-8 pt-8 pb-20">{@html html}</article>
  {:else}
    <pre class="mx-auto max-w-[900px] px-6 pt-6 pb-20 font-mono text-[12.5px] leading-relaxed whitespace-pre-wrap break-words text-muted [tab-size:2]">{content}</pre>
  {/if}
</div>
