<script lang="ts">
  import Icon from "./Icon.svelte";

  type Theme = "dark" | "light" | "system";

  let {
    theme,
    workspace,
    workspaceName,
    fileCount,
    skillCount,
    refreshing,
    onTheme,
    onRescan,
    onForget,
  }: {
    theme: Theme;
    workspace: string | null;
    workspaceName: string;
    fileCount: number;
    skillCount: number;
    refreshing: boolean;
    onTheme: (theme: Theme) => void;
    onRescan: () => void;
    onForget: () => void;
  } = $props();

  const themes: { value: Theme; label: string; icon: "moon" | "sun" | "computer" }[] = [
    { value: "dark", label: "Dark", icon: "moon" },
    { value: "light", label: "Light", icon: "sun" },
    { value: "system", label: "System", icon: "computer" },
  ];

  const stack = [
    { name: "Tauri", icon: "tauri" },
    { name: "Svelte", icon: "svelte" },
    { name: "Rust", icon: "rust" },
    { name: "TypeScript", icon: "typescript" },
  ];
</script>

<div class="h-full overflow-auto bg-editor">
  <div class="mx-auto w-full max-w-[760px] px-8 py-10">
    <h1 class="m-0 text-2xl font-semibold tracking-[-0.02em] text-ink">Settings</h1>
    <p class="mt-2 max-w-[52ch] text-sm leading-relaxed text-muted">
      Tune how CODE.MD looks and where it reads from. Everything stays on this machine.
    </p>

    <section class="mt-9 border-b border-line pb-8">
      <h2 class="m-0 text-sm font-semibold text-ink">Appearance</h2>
      <p class="mt-1.5 text-xs leading-relaxed text-faint">Choose the surface that matches your room.</p>
      <div class="mt-4 grid grid-cols-3 gap-2">
        {#each themes as option (option.value)}
          <button
            type="button"
            aria-pressed={theme === option.value}
            class="flex items-center gap-2 rounded-lg border px-3 py-2.5 text-left text-[13px] font-medium transition-colors duration-150 {theme ===
            option.value
              ? 'border-line-strong bg-elev text-ink'
              : 'border-line text-muted hover:bg-elev hover:text-ink'}"
            onclick={() => onTheme(option.value)}
          >
            <Icon name={option.icon} size={16} />
            <span>{option.label}</span>
            {#if theme === option.value}
              <span class="ml-auto size-1.5 rounded-full bg-ink"></span>
            {/if}
          </button>
        {/each}
      </div>
    </section>

    <section class="mt-8 border-b border-line pb-8">
      <h2 class="m-0 text-sm font-semibold text-ink">Workspace</h2>
      <p class="mt-1.5 text-xs leading-relaxed text-faint">The folder CODE.MD is reading right now.</p>
      {#if workspace}
        <div class="mt-4 rounded-lg bg-elev px-3 py-3">
          <div class="flex items-baseline justify-between gap-3">
            <span class="truncate text-[13px] font-medium text-ink">{workspaceName}</span>
            <span class="shrink-0 text-[11px] tabular-nums text-faint">{fileCount} files</span>
          </div>
          <p class="mt-1 truncate font-mono text-[11px] text-faint" title={workspace}>{workspace}</p>
          <div class="mt-3 flex items-center gap-2">
            <button
              type="button"
              class="inline-flex h-8 items-center gap-2 rounded-md border border-line-strong px-2.5 text-xs font-medium text-ink transition-colors duration-150 hover:bg-hover disabled:opacity-40"
              disabled={refreshing}
              onclick={onRescan}
            >
              <Icon name="refresh" size={14} />
              <span>{refreshing ? "Scanning" : "Rescan"}</span>
            </button>
            <button
              type="button"
              class="inline-flex h-8 items-center gap-2 rounded-md px-2.5 text-xs font-medium text-muted transition-colors duration-150 hover:bg-hover hover:text-ink"
              onclick={onForget}
            >
              <Icon name="close" size={14} />
              <span>Forget folder</span>
            </button>
            <span class="ml-auto text-[11px] tabular-nums text-faint">{skillCount} {skillCount === 1 ? "skill" : "skills"}</span>
          </div>
        </div>
      {:else}
        <p class="mt-4 text-[13px] text-muted">No folder open.</p>
      {/if}
    </section>

    <section class="mt-8">
      <h2 class="m-0 text-sm font-semibold text-ink">About</h2>
      <p class="mt-1.5 text-xs leading-relaxed text-faint">CODE.MD 0.1.0 · local-first, no account, no sync.</p>
      <div class="mt-4 flex flex-wrap items-center gap-x-5 gap-y-3">
        {#each stack as item (item.name)}
          <span class="flex items-center gap-2 text-xs text-muted">
            <img class="theme-icon size-4" src="/icons/{item.icon}.svg" alt="" />
            {item.name}
          </span>
        {/each}
      </div>
      <p class="mt-5 text-[11px] text-faint">UI system by LavUIX.</p>
    </section>
  </div>
</div>
