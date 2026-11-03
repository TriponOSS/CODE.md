<script lang="ts">
  import Icon from "./Icon.svelte";

  let { path, size = 15 }: { path: string; size?: number } = $props();

  const languageIcons: Record<string, string> = {
    css: "css",
    html: "html5",
    htm: "html5",
    js: "javascript",
    jsx: "javascript",
    mjs: "javascript",
    cjs: "javascript",
    json: "json",
    jsonc: "json",
    md: "markdown",
    mdx: "markdown",
    markdown: "markdown",
    py: "python",
    rs: "rust",
    sh: "shell",
    bash: "shell",
    zsh: "shell",
    fish: "shell",
    svelte: "svelte",
    ts: "typescript",
    tsx: "typescript",
    yaml: "yaml",
    yml: "yaml",
  };

  const icon = $derived.by(() => {
    const name = path.split("/").pop()?.toLowerCase() ?? "";
    if (name === "tauri.conf.json") return "tauri";
    if (name === "vite.config.ts" || name === "vite.config.js") return "vite";
    const extension = name.includes(".") ? name.split(".").pop() ?? "" : "";
    return languageIcons[extension] ?? null;
  });
</script>

{#if icon}
  <img
    class="theme-icon shrink-0 object-contain"
    style="width: {size}px; height: {size}px"
    src="/icons/{icon}.svg"
    alt=""
    aria-hidden="true"
  />
{:else}
  <Icon name="file" {size} />
{/if}
