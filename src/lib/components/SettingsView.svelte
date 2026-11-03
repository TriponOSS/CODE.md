<script lang="ts">
  import type { AppSettings, Theme } from "$lib/settings";
  import Icon from "./Icon.svelte";
  import Toggle from "./Toggle.svelte";

  type CategoryId =
    | "general"
    | "appearance"
    | "editor"
    | "preview"
    | "git"
    | "workspace"
    | "keybindings"
    | "about";

  let {
    theme,
    settings,
    workspace,
    workspaceName,
    fileCount,
    skillCount,
    refreshing,
    onTheme,
    onSettings,
    onRescan,
    onOpenFolder,
    onForget,
    onResetPreview,
    onClose,
  }: {
    theme: Theme;
    settings: AppSettings;
    workspace: string | null;
    workspaceName: string;
    fileCount: number;
    skillCount: number;
    refreshing: boolean;
    onTheme: (theme: Theme) => void;
    onSettings: (settings: AppSettings) => void;
    onRescan: () => void;
    onOpenFolder: () => void;
    onForget: () => void;
    onResetPreview: () => void;
    onClose: () => void;
  } = $props();

  const categories: {
    id: CategoryId;
    label: string;
    icon: "settings" | "moon" | "text" | "eye" | "git" | "folder" | "command" | "library";
    keywords: string;
  }[] = [
    { id: "general", label: "General", icon: "settings", keywords: "startup confirm discard tabs safety" },
    { id: "appearance", label: "Appearance", icon: "moon", keywords: "theme dark light system motion animation color" },
    { id: "editor", label: "Editor", icon: "text", keywords: "font size line height tab wrap line numbers" },
    { id: "preview", label: "Preview", icon: "eye", keywords: "markdown font size split width pane" },
    { id: "git", label: "Source control", icon: "git", keywords: "git refresh status branch changes commit" },
    { id: "workspace", label: "Workspace", icon: "folder", keywords: "folder path rescan forget files" },
    { id: "keybindings", label: "Keybindings", icon: "command", keywords: "shortcuts keyboard hotkeys" },
    { id: "about", label: "About", icon: "library", keywords: "version stack license local" },
  ];

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

  const shortcuts = [
    { keys: ["Ctrl", "K"], label: "Quick open and commands" },
    { keys: ["Ctrl", "O"], label: "Open folder" },
    { keys: ["Ctrl", "S"], label: "Save file" },
    { keys: ["Ctrl", "B"], label: "Toggle sidebar" },
    { keys: ["Ctrl", "W"], label: "Close tab" },
    { keys: ["Ctrl", ","], label: "Open settings" },
  ];

  let active = $state<CategoryId>("general");
  let query = $state("");

  const normalized = $derived(query.trim().toLowerCase());
  const visibleCategories = $derived(
    normalized === ""
      ? categories
      : categories.filter((category) =>
          `${category.label} ${category.keywords}`.toLowerCase().includes(normalized),
        ),
  );
  const activeCategory = $derived(categories.find((category) => category.id === active) ?? categories[0]);

  $effect(() => {
    if (normalized !== "" && !visibleCategories.some((category) => category.id === active)) {
      active = visibleCategories[0]?.id ?? active;
    }
  });

  function updateGeneral<K extends keyof AppSettings["general"]>(
    key: K,
    value: AppSettings["general"][K],
  ) {
    onSettings({ ...settings, general: { ...settings.general, [key]: value } });
  }

  function updateAppearance<K extends keyof AppSettings["appearance"]>(
    key: K,
    value: AppSettings["appearance"][K],
  ) {
    onSettings({ ...settings, appearance: { ...settings.appearance, [key]: value } });
  }

  function updateEditor<K extends keyof AppSettings["editor"]>(
    key: K,
    value: AppSettings["editor"][K],
  ) {
    onSettings({ ...settings, editor: { ...settings.editor, [key]: value } });
  }

  function updatePreview<K extends keyof AppSettings["preview"]>(
    key: K,
    value: AppSettings["preview"][K],
  ) {
    onSettings({ ...settings, preview: { ...settings.preview, [key]: value } });
  }

  function updateGit<K extends keyof AppSettings["git"]>(
    key: K,
    value: AppSettings["git"][K],
  ) {
    onSettings({ ...settings, git: { ...settings.git, [key]: value } });
  }

  function numberFrom(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value);
  }
</script>

<div class="flex h-full min-h-0 bg-editor">
  <aside class="flex w-[226px] shrink-0 flex-col border-r border-line bg-panel">
    <div class="px-4 pt-5 pb-3">
      <h1 class="m-0 text-[15px] font-semibold tracking-[-0.01em] text-ink">Settings</h1>
      <p class="mt-1 mb-0 text-[11px] leading-relaxed text-faint">Preferences stay on this machine.</p>
    </div>

    <div class="mx-3 mb-3 flex h-8 items-center gap-1.5 rounded-md border border-line bg-elev px-2 text-faint focus-within:border-line-strong">
      <Icon name="search" size={13} />
      <input
        type="search"
        placeholder="Search settings"
        aria-label="Search settings"
        bind:value={query}
        spellcheck="false"
        autocomplete="off"
        class="min-w-0 flex-1 border-0 bg-transparent text-[12px] text-ink outline-none placeholder:text-faint"
      />
    </div>

    <nav class="min-h-0 flex-1 overflow-auto px-2 pb-4" aria-label="Settings categories">
      {#each visibleCategories as category (category.id)}
        <button
          class="mb-0.5 flex h-8 w-full items-center gap-2.5 rounded-md px-2.5 text-left text-[12px] transition-colors duration-100 {active === category.id ? 'bg-elev text-ink' : 'text-muted hover:bg-hover hover:text-ink'}"
          type="button"
          aria-current={active === category.id ? "page" : undefined}
          onclick={() => (active = category.id)}
        >
          <Icon name={category.icon} size={15} />
          <span class="truncate">{category.label}</span>
        </button>
      {/each}
      {#if visibleCategories.length === 0}
        <p class="m-0 px-2.5 py-2 text-[11.5px] leading-relaxed text-faint">No settings match “{query.trim()}”.</p>
      {/if}
    </nav>

    <div class="border-t border-line px-3 py-2.5">
      <button class="mb-2 flex h-7 w-full items-center gap-2 rounded-md px-1.5 text-left text-[11px] text-muted transition-colors duration-100 hover:bg-hover hover:text-ink" type="button" onclick={onClose}>
        <Icon name="outline" size={13} />Back to workspace
      </button>
      <p class="m-0 px-1.5 text-[10px] text-faint">CODE.MD 0.1.0</p>
    </div>
  </aside>

  <div class="min-w-0 flex-1 overflow-auto">
    <div class="mx-auto w-full max-w-[720px] px-8 py-8">
      {#if visibleCategories.length === 0}
        <div class="py-12">
          <h2 class="m-0 text-lg font-semibold text-ink">No settings found</h2>
          <p class="mt-2 mb-0 text-xs leading-relaxed text-faint">Try “editor”, “theme”, “git”, or “keybindings”.</p>
        </div>
      {:else}
        <h2 class="m-0 text-lg font-semibold tracking-[-0.01em] text-ink">{activeCategory.label}</h2>
        <p class="mt-1.5 mb-0 text-xs leading-relaxed text-faint">
          {#if active === "general"}Startup and safety behavior.
          {:else if active === "appearance"}Theme and motion across the workbench.
          {:else if active === "editor"}Type, wrapping, and editing behavior.
          {:else if active === "preview"}How rendered Markdown and split panes read.
          {:else if active === "git"}Source control refresh and status visibility.
          {:else if active === "workspace"}The folder currently open in CODE.MD.
          {:else if active === "keybindings"}Keyboard-first commands available anywhere.
          {:else}Version, stack, and local-first notes.{/if}
        </p>

        <div class="mt-6 border-t border-line">
          {#if active === "general"}
            <div class="flex items-center justify-between gap-6 border-b border-line py-4">
              <div class="min-w-0">
                <p class="m-0 text-[13px] font-medium text-ink">Confirm before discarding changes</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Ask before closing a tab with unsaved edits.</p>
              </div>
              <Toggle checked={settings.general.confirmDiscard} label="Confirm before discarding changes" onChange={(value) => updateGeneral("confirmDiscard", value)} />
            </div>
          {:else if active === "appearance"}
            <div class="border-b border-line py-4">
              <p class="m-0 text-[13px] font-medium text-ink">Theme</p>
              <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Choose the surface that matches your room.</p>
              <div class="mt-3 grid grid-cols-3 gap-2">
                {#each themes as option (option.value)}
                  <button
                    type="button"
                    aria-pressed={theme === option.value}
                    class="flex items-center gap-2 rounded-lg border px-3 py-2.5 text-left text-[12.5px] font-medium transition-colors duration-150 {theme === option.value ? 'border-line-strong bg-elev text-ink' : 'border-line text-muted hover:bg-elev hover:text-ink'}"
                    onclick={() => onTheme(option.value)}
                  >
                    <Icon name={option.icon} size={15} />
                    <span>{option.label}</span>
                    {#if theme === option.value}
                      <span class="ml-auto size-1.5 rounded-full bg-ink"></span>
                    {/if}
                  </button>
                {/each}
              </div>
            </div>
            <div class="flex items-center justify-between gap-6 border-b border-line py-4">
              <div class="min-w-0">
                <p class="m-0 text-[13px] font-medium text-ink">Reduce motion</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Disable palette and surface transitions.</p>
              </div>
              <Toggle checked={settings.appearance.reduceMotion} label="Reduce motion" onChange={(value) => updateAppearance("reduceMotion", value)} />
            </div>
          {:else if active === "editor"}
            <div class="flex items-center justify-between gap-6 border-b border-line py-4">
              <div class="min-w-0">
                <p class="m-0 text-[13px] font-medium text-ink">Font size</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Editor text size in pixels.</p>
              </div>
              <div class="flex shrink-0 items-center gap-3">
                <input class="w-36 cursor-pointer" type="range" min="11" max="22" step="1" value={settings.editor.fontSize} aria-label="Editor font size" style="accent-color: var(--text)" oninput={(event) => updateEditor("fontSize", numberFrom(event))} />
                <span class="w-9 text-right font-mono text-[11px] tabular-nums text-muted">{settings.editor.fontSize}px</span>
              </div>
            </div>
            <div class="flex items-center justify-between gap-6 border-b border-line py-4">
              <div class="min-w-0">
                <p class="m-0 text-[13px] font-medium text-ink">Line height</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Spacing between editor lines.</p>
              </div>
              <div class="flex shrink-0 items-center gap-3">
                <input class="w-36 cursor-pointer" type="range" min="1.3" max="2.2" step="0.05" value={settings.editor.lineHeight} aria-label="Editor line height" style="accent-color: var(--text)" oninput={(event) => updateEditor("lineHeight", numberFrom(event))} />
                <span class="w-9 text-right font-mono text-[11px] tabular-nums text-muted">{settings.editor.lineHeight.toFixed(2)}</span>
              </div>
            </div>
            <div class="flex items-center justify-between gap-6 border-b border-line py-4">
              <div class="min-w-0">
                <p class="m-0 text-[13px] font-medium text-ink">Word wrap</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Wrap long lines instead of scrolling horizontally.</p>
              </div>
              <Toggle checked={settings.editor.wordWrap} label="Word wrap" onChange={(value) => updateEditor("wordWrap", value)} />
            </div>
            <div class="flex items-center justify-between gap-6 border-b border-line py-4">
              <div class="min-w-0">
                <p class="m-0 text-[13px] font-medium text-ink">Line numbers</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Show the gutter beside the editor.</p>
              </div>
              <Toggle checked={settings.editor.lineNumbers} label="Line numbers" onChange={(value) => updateEditor("lineNumbers", value)} />
            </div>
            <div class="flex items-center justify-between gap-6 border-b border-line py-4">
              <div class="min-w-0">
                <p class="m-0 text-[13px] font-medium text-ink">Tab size</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Columns used for a tab character.</p>
              </div>
              <select class="h-8 rounded-md border border-line bg-elev px-2 text-[12px] text-ink outline-none focus:border-line-strong" value={settings.editor.tabSize} aria-label="Tab size" onchange={(event) => updateEditor("tabSize", numberFrom(event))}>
                <option value="2">2 spaces</option>
                <option value="4">4 spaces</option>
                <option value="8">8 spaces</option>
              </select>
            </div>
          {:else if active === "preview"}
            <div class="flex items-center justify-between gap-6 border-b border-line py-4">
              <div class="min-w-0">
                <p class="m-0 text-[13px] font-medium text-ink">Preview font size</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Base size for rendered Markdown.</p>
              </div>
              <div class="flex shrink-0 items-center gap-3">
                <input class="w-36 cursor-pointer" type="range" min="12" max="22" step="1" value={settings.preview.fontSize} aria-label="Preview font size" style="accent-color: var(--text)" oninput={(event) => updatePreview("fontSize", numberFrom(event))} />
                <span class="w-9 text-right font-mono text-[11px] tabular-nums text-muted">{settings.preview.fontSize}px</span>
              </div>
            </div>
            <div class="flex items-center justify-between gap-6 border-b border-line py-4">
              <div class="min-w-0">
                <p class="m-0 text-[13px] font-medium text-ink">Preview width</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Reset the editor and preview split to 50/50.</p>
              </div>
              <button class="inline-flex h-8 shrink-0 items-center gap-2 rounded-md border border-line px-2.5 text-[11.5px] font-medium text-muted transition-colors duration-100 hover:border-line-strong hover:text-ink" type="button" onclick={onResetPreview}>
                <Icon name="refresh" size={13} />Reset split
              </button>
            </div>
          {:else if active === "git"}
            <div class="flex items-center justify-between gap-6 border-b border-line py-4">
              <div class="min-w-0">
                <p class="m-0 text-[13px] font-medium text-ink">Refresh after save</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Update source control immediately after writing a file.</p>
              </div>
              <Toggle checked={settings.git.autoRefresh} label="Refresh source control after save" onChange={(value) => updateGit("autoRefresh", value)} />
            </div>
            <div class="flex items-center justify-between gap-6 border-b border-line py-4">
              <div class="min-w-0">
                <p class="m-0 text-[13px] font-medium text-ink">Show branch in status bar</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Keep branch and change counts visible at the bottom.</p>
              </div>
              <Toggle checked={settings.git.showInStatusBar} label="Show git branch in status bar" onChange={(value) => updateGit("showInStatusBar", value)} />
            </div>
          {:else if active === "workspace"}
            {#if workspace}
              <div class="border-b border-line py-4">
                <div class="flex items-baseline justify-between gap-3">
                  <span class="truncate text-[13px] font-medium text-ink">{workspaceName}</span>
                  <span class="shrink-0 text-[11px] tabular-nums text-faint">{fileCount} files · {skillCount} {skillCount === 1 ? "skill" : "skills"}</span>
                </div>
                <p class="mt-1 mb-0 truncate font-mono text-[11px] text-faint" title={workspace}>{workspace}</p>
                <div class="mt-3 flex items-center gap-2">
                  <button class="inline-flex h-8 items-center gap-2 rounded-md border border-line-strong px-2.5 text-xs font-medium text-ink transition-colors duration-150 hover:bg-hover disabled:opacity-40" type="button" disabled={refreshing} onclick={onRescan}>
                    <Icon name="refresh" size={14} />{refreshing ? "Scanning" : "Rescan"}
                  </button>
                  <button class="inline-flex h-8 items-center gap-2 rounded-md px-2.5 text-xs font-medium text-muted transition-colors duration-150 hover:bg-hover hover:text-ink" type="button" onclick={onForget}>
                    <Icon name="close" size={14} />Forget folder
                  </button>
                </div>
              </div>
            {:else}
              <div class="border-b border-line py-4">
                <p class="m-0 text-[13px] font-medium text-ink">No folder open</p>
                <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Open a folder of Markdown, text, and code files.</p>
                <button class="mt-3 inline-flex h-8 items-center gap-2 rounded-md bg-ink px-3 text-xs font-semibold text-app transition-opacity duration-100 hover:opacity-85" type="button" onclick={onOpenFolder}>
                  <Icon name="folder" size={14} />Open folder
                </button>
              </div>
            {/if}
          {:else if active === "keybindings"}
            {#each shortcuts as shortcut (shortcut.label)}
              <div class="flex items-center justify-between gap-6 border-b border-line py-3.5">
                <span class="text-[12.5px] text-muted">{shortcut.label}</span>
                <span class="flex shrink-0 items-center gap-1 font-mono text-[10px] text-faint">
                  {#each shortcut.keys as key (key)}<kbd class="rounded border border-line bg-elev px-1.5 py-0.5">{key}</kbd>{/each}
                </span>
              </div>
            {/each}
          {:else}
            <div class="border-b border-line py-4">
              <p class="m-0 text-[13px] font-medium text-ink">CODE.MD 0.1.0</p>
              <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">A local-first desktop workbench for Markdown, text, and code. No account, no sync.</p>
            </div>
            <div class="flex flex-wrap items-center gap-x-5 gap-y-3 border-b border-line py-4">
              {#each stack as item (item.name)}
                <span class="flex items-center gap-2 text-xs text-muted">
                  <img class="theme-icon size-4" src="/icons/{item.icon}.svg" alt="" />
                  {item.name}
                </span>
              {/each}
            </div>
            <div class="py-4 text-[11px] text-faint">UI system by LavUIX.</div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>
