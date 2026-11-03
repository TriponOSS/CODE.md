<script lang="ts">
  import { parseThemeDefinition, type AppSettings, type ColorTheme, type Theme, type ThemeDefinition } from "$lib/settings";
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
    colorTheme,
    customThemes,
    settings,
    workspace,
    workspaceName,
    fileCount,
    skillCount,
    onTheme,
    onColorTheme,
    onCustomThemes,
    onSettings,
    onRefresh,
    onOpenFolder,
    onForget,
    onResetPreview,
    onClose,
  }: {
    theme: Theme;
    colorTheme: ColorTheme;
    customThemes: ThemeDefinition[];
    settings: AppSettings;
    workspace: string | null;
    workspaceName: string;
    fileCount: number;
    skillCount: number;
    onTheme: (theme: Theme) => void;
    onColorTheme: (theme: ColorTheme) => void;
    onCustomThemes: (themes: ThemeDefinition[]) => void;
    onSettings: (settings: AppSettings) => void;
    onRefresh: () => void;
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
    { id: "workspace", label: "Workspace", icon: "folder", keywords: "folder path refresh forget files" },
    { id: "keybindings", label: "Keybindings", icon: "command", keywords: "shortcuts keyboard hotkeys" },
    { id: "about", label: "About", icon: "library", keywords: "version stack license local" },
  ];

  const themes: { value: Theme; label: string; icon: "moon" | "sun" | "computer" }[] = [
    { value: "system", label: "System", icon: "computer" },
    { value: "light", label: "Light", icon: "sun" },
    { value: "dark", label: "Dark", icon: "moon" },
  ];

  const colorThemes: { value: ColorTheme; label: string; primary: string; secondary: string }[] = [
    { value: "t3-code", label: "T3 Code", primary: "#dfd2ff", secondary: "#7453d1" },
    { value: "t3-chat", label: "T3 Chat", primary: "#ff91ba", secondary: "#b52f68" },
    { value: "grove", label: "Grove", primary: "#9adbb0", secondary: "#277d50" },
    { value: "ocean", label: "Ocean", primary: "#91cef2", secondary: "#2479b3" },
    { value: "ember", label: "Ember", primary: "#f6a087", secondary: "#c84c31" },
    { value: "iris", label: "Iris", primary: "#c9adff", secondary: "#7047cf" },
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
  let importError = $state("");
  let copiedTheme = $state<ColorTheme | null>(null);
  let importInput = $state<HTMLInputElement>();

  const allColorThemes = $derived([
    ...colorThemes,
    ...customThemes.map((item) => ({
      value: `custom:${item.id}` as ColorTheme,
      label: item.name,
      primary: item.colors.secondary ?? item.colors.accentForeground ?? item.colors.accent,
      secondary: item.colors.accent,
    })),
  ]);

  async function importTheme(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    try {
      const imported = parseThemeDefinition(JSON.parse(await file.text()));
      const id = `custom:${imported.id}` as ColorTheme;
      const next = [...customThemes.filter((item) => item.id !== imported.id), imported];
      onCustomThemes(next);
      onColorTheme(id);
      importError = "";
    } catch (error) {
      importError = error instanceof Error ? error.message : "Unable to import this theme.";
    } finally {
      input.value = "";
    }
  }

  async function copyTheme(option: (typeof allColorThemes)[number]) {
    const storedTheme = option.value.startsWith("custom:")
      ? customThemes.find((item) => `custom:${item.id}` === option.value)
      : undefined;
    const accent = option.secondary;
    const lightColors: Record<string, string> = {
      canvas: "#f3f3f3", chrome: "#f3f3f3", toolbar: "#f3f3f3", toolbarForeground: "#111111",
      toolbarBorder: "#e2e2e2", toolbarControl: "#efefef", toolbarControlForeground: "#111111",
      toolbarControlHover: "#e7e7e7", surface: "#fafafa", surfaceRaised: "#efefef",
      surfaceOverlay: "#e7e7e7", text: "#111111", textMuted: "#525252", border: "#e2e2e2",
      input: "#c9c9c9", focus: accent, accent, accentForeground: "#ffffff", secondary: "#efefef",
      secondaryForeground: "#111111", muted: "#efefef", mutedForeground: "#525252",
      placeholder: "#707070", secondaryLabel: "#707070", iconMuted: "#707070", error: "#b42318",
      errorForeground: "#b42318", errorSurface: "#fce8e6", warning: "#a07800",
      warningForeground: "#735700", warningSurface: "#fff4cc", update: accent,
      updateForeground: "#525252", updateSurface: "#efefef", accentSurface: "#efefef",
      accentSurfaceForeground: "#111111", messageSurface: "#fafafa", messageForeground: "#111111",
      messageAction: accent, messageActionForeground: "#ffffff", messageActionHover: accent,
      codeBackground: "#fafafa", codeForeground: "#111111", sidebar: "#ffffff",
      sidebarForeground: "#111111", sidebarMutedForeground: "#525252",
      sidebarControlSurface: "#efefef", sidebarRowHover: "#e7e7e7", sidebarRowActive: "#dedede",
      sidebarRowSelected: "#dedede", sidebarBorder: "#e2e2e2", terminalBackground: "#fafafa",
      terminalForeground: "#111111", terminalCursor: accent, terminalSelection: "#c9c9c9",
      terminalScrollbar: "#c9c9c9", terminalScrollbarHover: "#707070",
    };
    const darkColors: Record<string, string> = {
      canvas: "#0a0a0a", chrome: "#0a0a0a", toolbar: "#0a0a0a", toolbarForeground: "#f4f4f4",
      toolbarBorder: "#242424", toolbarControl: "#191919", toolbarControlForeground: "#f4f4f4",
      toolbarControlHover: "#222222", surface: "#0d0d0d", surfaceRaised: "#191919",
      surfaceOverlay: "#222222", text: "#f4f4f4", textMuted: "#a8a8a8", border: "#242424",
      input: "#363636", focus: option.primary, accent: option.primary, accentForeground: "#171717",
      secondary: "#191919", secondaryForeground: "#f4f4f4", muted: "#191919",
      mutedForeground: "#a8a8a8", placeholder: "#757575", secondaryLabel: "#757575",
      iconMuted: "#a8a8a8", error: "#e06c6c", errorForeground: "#e06c6c",
      errorSurface: "#3a2020", warning: "#c7ae68", warningForeground: "#d8c78b",
      warningSurface: "#38311d", update: option.primary, updateForeground: "#a8a8a8",
      updateSurface: "#191919", accentSurface: "#191919", accentSurfaceForeground: "#f4f4f4",
      messageSurface: "#111111", messageForeground: "#f4f4f4", messageAction: option.primary,
      messageActionForeground: "#171717", messageActionHover: option.secondary,
      codeBackground: "#0d0d0d", codeForeground: "#f4f4f4", sidebar: "#111111",
      sidebarForeground: "#f4f4f4", sidebarMutedForeground: "#a8a8a8",
      sidebarControlSurface: "#191919", sidebarRowHover: "#222222", sidebarRowActive: "#2a2a2a",
      sidebarRowSelected: "#2a2a2a", sidebarBorder: "#242424", terminalBackground: "#0d0d0d",
      terminalForeground: "#f4f4f4", terminalCursor: option.primary, terminalSelection: "#303030",
      terminalScrollbar: "#363636", terminalScrollbarHover: "#757575",
    };
    const definition = storedTheme ?? {
      version: 1 as const,
      id: option.value,
      name: option.label,
      appearance: "light" as const,
      description: `${option.label} color theme for CODE.MD.`,
      colors: lightColors,
      variants: { dark: darkColors },
    };
    const contents = JSON.stringify(definition, null, 2);
    try {
      await navigator.clipboard.writeText(contents);
    } catch {
      const field = document.createElement("textarea");
      field.value = contents;
      field.style.position = "fixed";
      field.style.opacity = "0";
      document.body.append(field);
      field.select();
      document.execCommand("copy");
      field.remove();
    }
    copiedTheme = option.value;
    setTimeout(() => {
      if (copiedTheme === option.value) copiedTheme = null;
    }, 1400);
  }

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
  <aside class="flex w-[226px] shrink-0 flex-col rounded-lg m-2 bg-panel">
    <div class="px-4 pt-5 pb-3">
      <h1 class="m-0 text-[15px] font-semibold tracking-[-0.01em] text-ink">Settings</h1>
      <p class="mt-1 mb-0 text-[11px] leading-relaxed text-faint">Preferences stay on this machine.</p>
    </div>

    <div class="mx-3 mb-3 flex h-8 items-center gap-1.5 rounded-md  bg-elev px-2 text-faint focus-within:border-line-strong">
      <Icon name="search" size={13} />
      <input
        type="search"
        placeholder="Search settings"
        aria-label="Search settings"
        bind:value={query}
        spellcheck="false"
        autocomplete="off"
        class="min-w-0 flex-1 bg-transparent text-[12px] text-ink placeholder:text-faint"
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
            <section class="border-b border-line py-4" aria-labelledby="color-scheme-heading">
              <p id="color-scheme-heading" class="m-0 text-[13px] font-medium text-ink">Color scheme</p>
              <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Set the workbench to follow your system or stay light or dark.</p>
              <div class="mt-3 grid grid-cols-3 gap-2.5">
                {#each themes as option (option.value)}
                  <button
                    type="button"
                    aria-pressed={theme === option.value}
                    aria-label="Use {option.label} color scheme"
                    class="group min-w-0 overflow-hidden rounded-lg border text-left transition-colors duration-150 {theme === option.value ? 'border-accent ring-1 ring-accent/40' : 'border-line hover:border-line-strong'}"
                    onclick={() => onTheme(option.value)}
                  >
                    <span class="flex h-[62px] gap-1.5 p-1.5 {option.value === 'dark' ? 'bg-[#111]' : 'bg-[#e9e9e9]'}" aria-hidden="true">
                      <span class="w-[27%] rounded-[3px] {option.value === 'dark' ? 'bg-[#292929]' : 'bg-white'}"></span>
                      <span class="flex min-w-0 flex-1 flex-col gap-1 rounded-[3px] p-1.5 {option.value === 'dark' ? 'bg-[#1b1b1b]' : 'bg-white'}">
                        <span class="h-1 w-3/4 rounded-full {option.value === 'dark' ? 'bg-[#777]' : 'bg-[#aaa]'}"></span>
                        <span class="h-1 w-1/2 rounded-full {option.value === 'dark' ? 'bg-[#444]' : 'bg-[#d0d0d0]'}"></span>
                        <span class="mt-auto h-1 w-full rounded-full {option.value === 'dark' ? 'bg-[#333]' : 'bg-[#e0e0e0]'}"></span>
                      </span>
                    </span>
                    <span class="flex h-8 items-center justify-between gap-1 px-2 text-[11px] font-medium {theme === option.value ? 'text-ink' : 'text-muted'}">
                      <span class="flex min-w-0 items-center gap-1.5 truncate"><Icon name={option.icon} size={13} />{option.label}</span>
                      {#if theme === option.value}<span class="size-1.5 shrink-0 rounded-full bg-accent"></span>{/if}
                    </span>
                  </button>
                {/each}
              </div>
            </section>
            <section class="border-b border-line py-4" aria-labelledby="themes-heading">
              <div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
                <div>
                  <p id="themes-heading" class="m-0 text-[13px] font-medium text-ink">Themes</p>
                  <p class="mt-1 mb-0 text-[11.5px] leading-relaxed text-faint">Choose an accent palette for the workbench.</p>
                </div>
                <div class="flex items-center gap-2">
                  <button class="rounded-md border border-line px-2 py-1 text-[10.5px] text-muted transition-colors hover:bg-elev hover:text-ink" type="button" onclick={() => importInput?.click()}>Import theme</button>
                  <input bind:this={importInput} class="visually-hidden" type="file" accept="application/json,.json" aria-label="Import theme JSON" onchange={importTheme} />
                </div>
              </div>
              {#if importError}<p class="mt-2 text-[11px] text-danger" role="alert">{importError}</p>{/if}
              <div class="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-3">
                {#each allColorThemes as option (option.value)}
                  <div class="relative min-w-0">
                    <button
                      type="button"
                      aria-pressed={colorTheme === option.value}
                      aria-label="Use {option.label} theme"
                      class="group flex min-h-[92px] w-full items-center justify-between gap-2 rounded-lg border px-3 py-2.5 pr-9 text-left transition-colors duration-150 {colorTheme === option.value ? 'border-accent bg-elev' : 'border-line hover:border-line-strong hover:bg-elev'}"
                      onclick={() => onColorTheme(option.value)}
                    >
                      <span class="truncate text-[12px] font-medium {colorTheme === option.value ? 'text-ink' : 'text-muted'}">{option.label}</span>
                      <span class="flex shrink-0 items-center -space-x-2" aria-hidden="true">
                        <span class="size-8 rounded-full border-2 border-panel shadow-sm" style="background: radial-gradient(circle at 32% 26%, {option.primary}, color-mix(in srgb, {option.primary} 70%, #000))"></span>
                        <span class="size-8 rounded-full border-2 border-panel shadow-sm" style="background: radial-gradient(circle at 32% 26%, {option.secondary}, color-mix(in srgb, {option.secondary} 68%, #000))"></span>
                      </span>
                    </button>
                    <button
                      class="absolute right-1.5 top-1/2 grid size-6 -translate-y-1/2 place-items-center rounded text-faint transition-colors hover:bg-hover hover:text-ink focus-visible:text-ink"
                      type="button"
                      aria-label="{copiedTheme === option.value ? 'Copied' : `Copy ${option.label} theme JSON`}"
                      title={copiedTheme === option.value ? "Copied" : "Copy theme JSON"}
                      onclick={() => void copyTheme(option)}
                    ><Icon name={copiedTheme === option.value ? "check" : "copy"} size={13} /></button>
                  </div>
                {/each}
              </div>
            </section>
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
                  <button class="inline-flex h-8 items-center gap-2 rounded-md border border-line-strong px-2.5 text-xs font-medium text-ink transition-colors duration-150 hover:bg-hover disabled:opacity-40" type="button" onclick={onRefresh}>
                    <Icon name="refresh" size={14} />Refresh
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
