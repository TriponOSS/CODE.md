<script lang="ts">
  import type { GitFile } from "$lib/types";
  import Icon from "./Icon.svelte";

  let {
    file,
    diff,
    staged,
    loading,
    error,
    onToggleStaged,
    onOpenFile,
    onClose,
  }: {
    file: GitFile;
    diff: string;
    staged: boolean;
    loading: boolean;
    error: string;
    onToggleStaged: () => void;
    onOpenFile: () => void;
    onClose: () => void;
  } = $props();

  const lines = $derived(diff === "" ? [] : diff.replace(/\n$/, "").split("\n"));
  const additions = $derived(
    lines.filter((line) => line.startsWith("+") && !line.startsWith("+++")).length,
  );
  const deletions = $derived(
    lines.filter((line) => line.startsWith("-") && !line.startsWith("---")).length,
  );

  function lineTone(line: string): string {
    if (line.startsWith("+++") || line.startsWith("---") || line.startsWith("diff --git")) {
      return "text-faint";
    }
    if (line.startsWith("@@")) return "bg-elev text-muted";
    if (line.startsWith("+")) return "bg-elev text-ink";
    if (line.startsWith("-")) return "bg-danger/10 text-danger";
    return "text-muted";
  }
</script>

<div class="flex h-[38px] shrink-0 items-center gap-2 border-b border-line bg-panel px-3 text-xs">
  <Icon name="git" size={14} />
  <span class="min-w-0 truncate font-mono text-[11px] text-muted" title={file.path}>{file.path}</span>
  <span class="shrink-0 rounded-full bg-elev px-1.5 py-0.5 text-[10px] font-medium text-faint">{staged ? "Staged" : "Working tree"}</span>
  <span class="ml-auto flex shrink-0 items-center gap-2 tabular-nums text-[10px]">
    {#if !loading && !error && diff}
      <span class="text-ink">+{additions}</span>
      <span class="text-danger">-{deletions}</span>
    {/if}
    <button class="inline-flex h-6 items-center gap-1 rounded-md border border-line px-2 text-[11px] font-medium text-muted transition-colors duration-100 hover:border-line-strong hover:text-ink" type="button" onclick={onToggleStaged}>
      {staged ? "Working tree" : "Staged"}
    </button>
    <button class="inline-flex h-6 items-center gap-1 rounded-md border border-line px-2 text-[11px] font-medium text-muted transition-colors duration-100 hover:border-line-strong hover:text-ink" type="button" onclick={onOpenFile}>
      <Icon name="file" size={12} />Open file
    </button>
    <button class="grid size-6 place-items-center rounded-md text-faint transition-colors duration-100 hover:bg-elev hover:text-ink" type="button" title="Close diff" aria-label="Close diff" onclick={onClose}>
      <Icon name="close" size={13} />
    </button>
  </span>
</div>

<div class="min-h-0 flex-1 overflow-auto bg-editor">
  {#if loading}
    <p class="m-0 px-4 py-4 text-xs text-faint">Loading diff...</p>
  {:else if error}
    <p class="m-0 px-4 py-4 text-xs leading-relaxed text-danger">{error}</p>
  {:else if lines.length === 0}
    <div class="px-5 py-8">
      <p class="m-0 text-[13px] font-medium text-ink">No textual diff</p>
      <p class="mt-1.5 mb-0 text-xs leading-relaxed text-faint">
        {file.untracked ? "This file is untracked. Open it to review the contents." : "The file matches the selected revision."}
      </p>
    </div>
  {:else}
    <div class="min-w-full w-max py-2 font-mono text-[11.5px] leading-[1.65]">
      {#each lines as line, index (index)}
        <div class="whitespace-pre px-4 {lineTone(line)}">{line || " "}</div>
      {/each}
    </div>
  {/if}
</div>
