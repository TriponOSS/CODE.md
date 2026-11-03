<script lang="ts">
  import type { GitCommit, GitFile, GitStatus } from "$lib/types";
  import Icon from "./Icon.svelte";

  let {
    status,
    loading,
    error,
    busy,
    commits,
    selectedPath,
    selectedStaged,
    onRefresh,
    onInit,
    onSelect,
    onStage,
    onUnstage,
    onCommit,
    onPush,
  }: {
    status: GitStatus | null;
    loading: boolean;
    error: string;
    busy: boolean;
    commits: GitCommit[];
    selectedPath: string | null;
    selectedStaged: boolean;
    onRefresh: () => void;
    onInit: () => void;
    onSelect: (file: GitFile, staged: boolean) => void;
    onStage: (paths: string[]) => void;
    onUnstage: (paths: string[]) => void;
    onCommit: (message: string) => Promise<boolean>;
    onPush: () => Promise<boolean>;
  } = $props();

  let message = $state("");
  let committing = $state(false);
  let pushing = $state(false);

  const stagedFiles = $derived(status?.files.filter((file) => file.staged) ?? []);
  const changedFiles = $derived(status?.files.filter((file) => file.unstaged || file.untracked) ?? []);
  const total = $derived(status?.files.length ?? 0);

  function badge(file: GitFile, staged: boolean): string {
    if (file.untracked) return "?";
    const code = staged ? file.index : file.worktree;
    return code.trim() === "" ? "M" : code;
  }

  function tone(code: string): string {
    if (code === "D" || code === "U") return "text-danger";
    if (code === "A") return "text-ok";
    return "text-muted";
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (committing || message.trim() === "" || stagedFiles.length === 0) return;
    committing = true;
    try {
      if (await onCommit(message.trim())) message = "";
    } finally {
      committing = false;
    }
  }

  async function push() {
    if (pushing || busy || !status?.hasRemote) return;
    pushing = true;
    try {
      await onPush();
    } finally {
      pushing = false;
    }
  }
</script>

<div class="flex min-h-0 flex-1 flex-col">
  {#if loading && !status}
    <p class="m-0 px-3 py-3 text-xs text-faint">Checking repository...</p>
  {:else if error}
    <div class="px-3.5 py-4">
      <p class="m-0 text-xs leading-relaxed text-danger">{error}</p>
      <button class="mt-3 inline-flex h-7 items-center gap-1.5 rounded-md border border-line px-2.5 text-[11px] font-medium text-muted transition-colors duration-100 hover:border-line-strong hover:text-ink" type="button" onclick={onRefresh}>
        <Icon name="refresh" size={13} />Try again
      </button>
    </div>
  {:else if !status}
    <div class="min-h-0 flex-1 px-3.5 py-4">
      <p class="m-0 text-[13px] font-medium text-ink">No repository</p>
      <p class="mt-1.5 mb-0 text-xs leading-relaxed text-faint">This folder is not tracked by Git yet.</p>
    </div>
    <div class="shrink-0 border-t border-line bg-panel p-2">
      <button class="inline-flex h-8 w-full items-center justify-center gap-2 rounded-md bg-ink px-3 text-xs font-semibold text-app transition-opacity duration-100 hover:opacity-85" type="button" onclick={onInit}>
        <Icon name="git" size={14} />Initialize repository
      </button>
    </div>
  {:else}
    <div class="shrink-0 border-b border-line px-3 pt-1.5 pb-2.5">
      <div class="flex min-w-0 items-center gap-1.5 text-[12px]">
        <Icon name="git" size={14} />
        <span class="truncate font-medium text-ink">{status.branch}</span>
        {#if status.detached}<span class="shrink-0 text-[10px] text-faint">detached</span>{/if}
        <span class="ml-auto shrink-0 tabular-nums text-[11px] text-faint">
          {#if status.ahead > 0}<span title="{status.ahead} ahead">↑{status.ahead}</span>{/if}
          {#if status.behind > 0}<span class="ml-1" title="{status.behind} behind">↓{status.behind}</span>{/if}
          {#if status.ahead === 0 && status.behind === 0 && !status.clean}{total} {total === 1 ? "change" : "changes"}{/if}
          {#if status.clean}<span class="text-ok">clean</span>{/if}
        </span>
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-auto pb-4">
      {#if total === 0}
        <div class="px-3.5 py-5">
          <p class="m-0 text-xs leading-relaxed text-faint">No changes. Stage edits as you work, then commit them from here.</p>
        </div>
      {:else}
        {#if stagedFiles.length > 0}
          <div class="mt-2 flex items-center justify-between px-3 text-[10px] font-semibold uppercase tracking-[0.08em] text-faint">
            <span>Staged · {stagedFiles.length}</span>
            <button class="rounded px-1 text-[10px] font-medium normal-case tracking-normal text-faint hover:text-ink" type="button" disabled={busy} onclick={() => onUnstage(stagedFiles.map((file) => file.path))}>Unstage all</button>
          </div>
          <div class="mt-1 px-1.5">
            {#each stagedFiles as file (file.path)}
              {@const code = badge(file, true)}
              <div class="group flex items-center rounded-md {selectedPath === file.path && selectedStaged ? 'bg-active' : 'hover:bg-hover'}">
                <button class="flex min-w-0 flex-1 items-center gap-2 px-2 py-1.5 text-left" type="button" title={file.path} onclick={() => onSelect(file, true)}>
                  <span class="w-3 shrink-0 font-mono text-[10px] font-semibold {tone(code)}">{code}</span>
                  <span class="truncate text-[11.5px] text-muted group-hover:text-ink">{file.path}</span>
                </button>
                <button class="mr-1 grid size-5 shrink-0 place-items-center rounded text-faint opacity-0 transition-opacity duration-100 hover:bg-elev hover:text-ink focus-visible:opacity-100 group-hover:opacity-100" type="button" title="Unstage {file.path}" aria-label="Unstage {file.path}" disabled={busy} onclick={() => onUnstage([file.path])}>
                  <Icon name="unstage" size={12} />
                </button>
              </div>
            {/each}
          </div>
        {/if}

        {#if changedFiles.length > 0}
          <div class="mt-3 flex items-center justify-between px-3 text-[10px] font-semibold uppercase tracking-[0.08em] text-faint">
            <span>Changes · {changedFiles.length}</span>
            <button class="rounded px-1 text-[10px] font-medium normal-case tracking-normal text-faint hover:text-ink" type="button" disabled={busy} onclick={() => onStage(changedFiles.map((file) => file.path))}>Stage all</button>
          </div>
          <div class="mt-1 px-1.5">
            {#each changedFiles as file (file.path)}
              {@const code = badge(file, false)}
              <div class="group flex items-center rounded-md {selectedPath === file.path && !selectedStaged ? 'bg-active' : 'hover:bg-hover'}">
                <button class="flex min-w-0 flex-1 items-center gap-2 px-2 py-1.5 text-left" type="button" title={file.path} onclick={() => onSelect(file, false)}>
                  <span class="w-3 shrink-0 font-mono text-[10px] font-semibold {tone(code)}">{code}</span>
                  <span class="truncate text-[11.5px] text-muted group-hover:text-ink">{file.path}</span>
                </button>
                <button class="mr-1 grid size-5 shrink-0 place-items-center rounded text-faint opacity-0 transition-opacity duration-100 hover:bg-elev hover:text-ink focus-visible:opacity-100 group-hover:opacity-100" type="button" title="Stage {file.path}" aria-label="Stage {file.path}" disabled={busy} onclick={() => onStage([file.path])}>
                  <Icon name="add" size={12} />
                </button>
              </div>
            {/each}
          </div>
        {/if}
      {/if}

      {#if commits.length > 0}
        <div class="mt-4 flex items-center justify-between px-3 text-[10px] font-semibold uppercase tracking-[0.08em] text-faint">
          <span>Recent commits</span>
          <button class="rounded px-1 text-[10px] font-medium normal-case tracking-normal text-faint hover:text-ink" type="button" onclick={onRefresh}>Refresh</button>
        </div>
        <div class="mt-1 px-1.5">
          {#each commits as commit (commit.hash)}
            <div class="flex items-start gap-2 rounded-md px-2 py-1.5" title="{commit.author} · {commit.date}">
              <span class="mt-px shrink-0 font-mono text-[10px] text-faint">{commit.hash}</span>
              <span class="min-w-0 flex-1 truncate text-[11.5px] text-muted">{commit.subject}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <form class="shrink-0 border-t border-line bg-panel p-2" onsubmit={submit}>
      <input
        type="text"
        class="h-8 w-full rounded-md border border-line bg-elev px-2.5 text-[12px] text-ink outline-none placeholder:text-faint focus:border-line-strong"
        placeholder="Commit message"
        aria-label="Commit message"
        bind:value={message}
        spellcheck="false"
      />
      <div class="mt-1.5 flex items-center gap-1.5">
        <button class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-md bg-ink px-2 text-[11px] font-semibold text-app transition-opacity duration-100 hover:opacity-85 disabled:opacity-35" type="submit" disabled={busy || committing || message.trim() === "" || stagedFiles.length === 0}>
          <Icon name="commit" size={13} />{committing ? "Committing..." : "Commit"}
        </button>
        <button class="inline-flex h-8 items-center justify-center gap-1.5 rounded-md border border-line px-2.5 text-[11px] font-medium text-muted transition-colors duration-100 hover:border-line-strong hover:text-ink disabled:opacity-35" type="button" title={status.hasRemote ? "Push current branch" : "Add a remote before pushing"} disabled={busy || pushing || !status.hasRemote} onclick={() => void push()}>
          <Icon name="git" size={13} />{pushing ? "Pushing..." : "Push"}
        </button>
      </div>
      {#if !status.hasRemote}
        <p class="m-0 mt-1.5 text-[10px] leading-relaxed text-faint">No remote configured. Add one to push.</p>
      {/if}
    </form>
  {/if}
</div>
