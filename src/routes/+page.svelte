<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import {
    closeWorkspace,
    commitFiles,
    getGitDiff,
    getGitLog,
    getGitStatus,
    initGit,
    isTauri,
    openWorkspace,
    pickWorkspace,
    readFile,
    saveFile,
    scanSkills,
    scanTree,
    stageFiles,
    unstageFiles,
    workspaceRoot,
  } from "$lib/api";
  import { isMarkdownPath, isSkillPath } from "$lib/files";
  import { ancestorsOf } from "$lib/tree";
  import { documentStats, extractOutline } from "$lib/outline";
  import type { GitCommit, GitFile, GitStatus, SidebarView, SkillInfo, TreeEntry } from "$lib/types";
  import CommandPalette from "$lib/components/CommandPalette.svelte";
  import DiffView from "$lib/components/DiffView.svelte";
  import Editor from "$lib/components/Editor.svelte";
  import FileGlyph from "$lib/components/FileGlyph.svelte";
  import FileTree from "$lib/components/FileTree.svelte";
  import GitView from "$lib/components/GitView.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import OutlineView from "$lib/components/OutlineView.svelte";
  import Preview from "$lib/components/Preview.svelte";
  import SearchView from "$lib/components/SearchView.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import SkillsView from "$lib/components/SkillsView.svelte";
  import type { PaletteCommand, PaletteFile } from "$lib/command";

  type Theme = "dark" | "light" | "system";

  function storedTheme(): Theme {
    if (!browser) return "dark";
    const value = localStorage.getItem("codemd:theme");
    return value === "light" || value === "system" || value === "dark" ? value : "dark";
  }

  type Tab = {
    path: string;
    name: string;
    content: string;
    original: string;
    revealLine: number;
  };

  const PREVIEW_RATIO_KEY = "codemd:preview-ratio";
  const MIN_PREVIEW_RATIO = 0.28;
  const MAX_PREVIEW_RATIO = 0.72;

  function storedPreviewRatio(): number {
    if (!browser) return 0.5;
    const stored = Number(localStorage.getItem(PREVIEW_RATIO_KEY));
    if (!Number.isFinite(stored)) return 0.5;
    return Math.min(MAX_PREVIEW_RATIO, Math.max(MIN_PREVIEW_RATIO, stored));
  }

  function clampPreviewRatio(value: number): number {
    return Math.min(MAX_PREVIEW_RATIO, Math.max(MIN_PREVIEW_RATIO, value));
  }

  function persistPreviewRatio(): void {
    if (browser) localStorage.setItem(PREVIEW_RATIO_KEY, String(previewRatio));
  }

  function startPreviewResize(event: PointerEvent): void {
    if (!splitEl) return;
    event.preventDefault();
    resizingPreview = true;
    resizeStartX = event.clientX;
    resizeStartRatio = previewRatio;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function movePreviewResize(event: PointerEvent): void {
    if (!resizingPreview || !splitEl) return;
    const width = splitEl.getBoundingClientRect().width;
    if (width === 0) return;
    previewRatio = clampPreviewRatio(resizeStartRatio + (resizeStartX - event.clientX) / width);
  }

  function finishPreviewResize(event: PointerEvent): void {
    if (!resizingPreview) return;
    resizingPreview = false;
    persistPreviewRatio();
    const target = event.currentTarget as HTMLElement;
    if (target.hasPointerCapture(event.pointerId)) target.releasePointerCapture(event.pointerId);
  }

  function nudgePreviewResize(delta: number): void {
    previewRatio = clampPreviewRatio(previewRatio + delta);
    persistPreviewRatio();
  }

  function resetPreviewResize(): void {
    previewRatio = 0.5;
    persistPreviewRatio();
  }

  let workspace = $state<string | null>(null);
  let tree = $state<TreeEntry[]>([]);
  let skills = $state<SkillInfo[]>([]);
  let tabs = $state<Tab[]>([]);
  let activePath = $state<string | null>(null);
  let sidebarView = $state<SidebarView>("files");
  let theme = $state<Theme>(storedTheme());
  let menu = $state<"file" | "view" | "go" | null>(null);
  let paletteOpen = $state(false);
  let sidebarOpen = $state(true);
  let collapsed = $state(new Set<string>());
  let previewOn = $state(true);
  let previewRatio = $state(storedPreviewRatio());
  let resizingPreview = $state(false);
  let splitEl = $state<HTMLDivElement>();
  let resizeStartX = 0;
  let resizeStartRatio = 0.5;
  let revealKey = $state(0);
  let refreshing = $state(false);
  let notice = $state<{ kind: "ok" | "err"; text: string } | null>(null);
  let noticeTimer: ReturnType<typeof setTimeout> | undefined;
  let git = $state<GitStatus | null>(null);
  let gitCommits = $state<GitCommit[]>([]);
  let gitLoading = $state(false);
  let gitBusy = $state(false);
  let gitError = $state("");
  let gitActionError = $state("");
  let gitSelected = $state<GitFile | null>(null);
  let gitSelectedStaged = $state(false);
  let gitDiff = $state("");
  let gitDiffLoading = $state(false);
  let gitDiffError = $state("");

  const desktop = browser && isTauri();
  const activeTab = $derived(tabs.find((tab) => tab.path === activePath) ?? null);
  const dirty = $derived(activeTab ? activeTab.content !== activeTab.original : false);
  const invalidSkills = $derived(skills.filter((skill) => !skill.valid).length);
  const outline = $derived(activeTab ? extractOutline(activeTab.content) : []);
  const stats = $derived(activeTab ? documentStats(activeTab.content) : null);
  const workspaceName = $derived(
    workspace ? (workspace.split("/").filter(Boolean).pop() ?? workspace) : "",
  );
  const paletteFiles = $derived<PaletteFile[]>(
    tree
      .filter((entry) => !entry.isDir)
      .map((entry) => ({ path: entry.path, name: entry.name })),
  );
  const paletteCommands = $derived<PaletteCommand[]>([
    {
      id: "open-folder",
      label: "Open folder",
      hint: "Ctrl O",
      keywords: "workspace directory project",
      run: () => void openFolder(),
    },
    {
      id: "save-file",
      label: "Save file",
      hint: "Ctrl S",
      keywords: "write disk",
      disabled: !dirty,
      run: () => void save(),
    },
    {
      id: "toggle-preview",
      label: previewOn ? "Hide preview" : "Show preview",
      keywords: "markdown split pane",
      disabled: !activeTab,
      run: () => (previewOn = !previewOn),
    },
    {
      id: "document-list",
      label: "Show documents",
      keywords: "files tree project",
      disabled: !workspace,
      run: () => (sidebarView = "files"),
    },
    {
      id: "outline",
      label: "Show outline",
      keywords: "headings document structure",
      disabled: !activeTab,
      run: () => (sidebarView = "outline"),
    },
    {
      id: "search",
      label: "Search in folder",
      keywords: "find text grep",
      disabled: !workspace,
      run: () => (sidebarView = "search"),
    },
    {
      id: "source-control",
      label: "Show source control",
      keywords: "git changes commit stage diff branch",
      disabled: !workspace,
      run: () => {
        sidebarView = "git";
        sidebarOpen = true;
        void refreshGit();
      },
    },
    {
      id: "settings",
      label: "Open settings",
      hint: "Ctrl ,",
      keywords: "preferences theme appearance",
      run: () => (sidebarView = "settings"),
    },
    {
      id: "rescan",
      label: "Rescan folder",
      keywords: "refresh reload files",
      disabled: !workspace || refreshing,
      run: () => void refresh(),
    },
    {
      id: "toggle-theme",
      label: theme === "light" ? "Use dark theme" : "Use light theme",
      keywords: "appearance color mode",
      run: () => applyTheme(theme === "light" ? "dark" : "light"),
    },
    {
      id: "sidebar",
      label: sidebarOpen ? "Hide sidebar" : "Show sidebar",
      hint: "Ctrl B",
      keywords: "panel tree",
      run: () => (sidebarOpen = !sidebarOpen),
    },
    {
      id: "close-tab",
      label: "Close current tab",
      hint: "Ctrl W",
      keywords: "file",
      disabled: !activeTab,
      run: () => closeActiveTab(),
    },
  ]);

  function applyTheme(next: Theme) {
    theme = next;
    if (!browser) return;
    localStorage.setItem("codemd:theme", next);
    const dark =
      next === "dark" ||
      (next === "system" && !window.matchMedia("(prefers-color-scheme: light)").matches);
    document.documentElement.dataset.theme = dark ? "dark" : "light";
  }

  async function forgetWorkspace() {
    try {
      await closeWorkspace();
    } catch {
      // Browser preview has no backend state to clear.
    }
    workspace = null;
    tabs = [];
    activePath = null;
    tree = [];
    skills = [];
    git = null;
    gitCommits = [];
    closeGitDiff();
    sidebarView = "files";
    localStorage.removeItem("codemd:workspace");
    say("ok", "Folder closed");
  }

  async function refreshGit() {
    if (!workspace || !desktop) {
      git = null;
      gitCommits = [];
      return;
    }
    gitLoading = true;
    gitError = "";
    try {
      const status = await getGitStatus();
      git = status;
      gitCommits = status ? await getGitLog(8) : [];
      if (gitSelected && status) {
        const next = status.files.find((file) => file.path === gitSelected?.path);
        if (!next) closeGitDiff();
      }
    } catch (cause) {
      git = null;
      gitCommits = [];
      gitError = errorText(cause);
    } finally {
      gitLoading = false;
    }
  }

  function closeGitDiff() {
    gitSelected = null;
    gitDiff = "";
    gitDiffError = "";
    gitDiffLoading = false;
  }

  async function loadGitDiff(file: GitFile, staged: boolean) {
    gitDiffLoading = true;
    gitDiffError = "";
    gitDiff = "";
    try {
      gitDiff = await getGitDiff(file.path, staged);
    } catch (cause) {
      gitDiffError = errorText(cause);
    } finally {
      gitDiffLoading = false;
    }
  }

  async function selectGitFile(file: GitFile, staged: boolean) {
    gitSelected = file;
    gitSelectedStaged = staged;
    await loadGitDiff(file, staged);
  }

  async function toggleGitDiffStage() {
    if (!gitSelected) return;
    gitSelectedStaged = !gitSelectedStaged;
    await loadGitDiff(gitSelected, gitSelectedStaged);
  }

  async function refreshSelectedDiff() {
    if (!gitSelected) return;
    const next = git?.files.find((file) => file.path === gitSelected?.path);
    if (!next) {
      closeGitDiff();
      return;
    }
    gitSelected = next;
    await loadGitDiff(next, gitSelectedStaged);
  }

  async function stageSelected(paths: string[]) {
    gitBusy = true;
    gitActionError = "";
    try {
      await stageFiles(paths);
      await refreshGit();
      await refreshSelectedDiff();
    } catch (cause) {
      gitActionError = errorText(cause);
      say("err", gitActionError);
    } finally {
      gitBusy = false;
    }
  }

  async function unstageSelected(paths: string[]) {
    gitBusy = true;
    gitActionError = "";
    try {
      await unstageFiles(paths);
      await refreshGit();
      await refreshSelectedDiff();
    } catch (cause) {
      gitActionError = errorText(cause);
      say("err", gitActionError);
    } finally {
      gitBusy = false;
    }
  }

  async function commitStaged(message: string): Promise<boolean> {
    gitBusy = true;
    gitActionError = "";
    try {
      const hash = await commitFiles(message);
      await refreshGit();
      closeGitDiff();
      say("ok", `Committed ${hash}`);
      return true;
    } catch (cause) {
      gitActionError = errorText(cause);
      say("err", gitActionError);
      return false;
    } finally {
      gitBusy = false;
    }
  }

  async function initializeGit() {
    gitBusy = true;
    gitActionError = "";
    try {
      git = await initGit();
      gitCommits = await getGitLog(8);
      say("ok", "Initialized Git repository");
    } catch (cause) {
      gitActionError = errorText(cause);
      say("err", gitActionError);
    } finally {
      gitBusy = false;
    }
  }

  async function windowAction(action: "minimize" | "maximize" | "close") {
    if (!desktop) return;
    const win = getCurrentWindow();
    try {
      if (action === "minimize") await win.minimize();
      else if (action === "maximize") await win.toggleMaximize();
      else await win.close();
    } catch (cause) {
      say("err", errorText(cause));
    }
  }

  function basename(path: string): string {
    return path.split("/").pop() ?? path;
  }

  function errorText(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }

  function say(kind: "ok" | "err", text: string) {
    notice = { kind, text };
    clearTimeout(noticeTimer);
    noticeTimer = setTimeout(() => {
      notice = null;
    }, 4200);
  }

  async function refresh() {
    if (!workspace || refreshing) return;
    refreshing = true;
    try {
      const [nextTree, nextSkills] = await Promise.all([scanTree(), scanSkills()]);
      tree = nextTree;
      skills = nextSkills;

      const reloaded = await Promise.all(
        tabs.map(async (tab) => {
          if (tab.content !== tab.original) return tab;
          try {
            const text = await readFile(tab.path);
            if (text === tab.content) return tab;
            return { ...tab, content: text, original: text };
          } catch {
            return tab;
          }
        }),
      );
      tabs = reloaded;
      await refreshGit();
    } catch (cause) {
      say("err", errorText(cause));
    } finally {
      refreshing = false;
    }
  }

  async function useWorkspace(path: string) {
    const display = await openWorkspace(path);
    workspace = display;
    localStorage.setItem("codemd:workspace", display);
    localStorage.removeItem("trpsc:workspace");
    tabs = [];
    activePath = null;
    collapsed = new Set();
    await refresh();
  }

  async function openFolder() {
    try {
      const picked = await pickWorkspace();
      if (picked) await useWorkspace(picked);
    } catch (cause) {
      say("err", errorText(cause));
    }
  }

  function expandAncestors(path: string) {
    const ancestors = new Set(ancestorsOf(path));
    collapsed = new Set([...collapsed].filter((dir) => !ancestors.has(dir)));
  }

  async function openFile(path: string, line = 0) {
    const existing = tabs.find((tab) => tab.path === path);
    if (existing) {
      activePath = path;
      existing.revealLine = line;
      if (line > 0) revealKey += 1;
      return;
    }
    try {
      const content = await readFile(path);
      tabs = [...tabs, { path, name: basename(path), content, original: content, revealLine: line }];
      activePath = path;
      if (line > 0) revealKey += 1;
      expandAncestors(path);
    } catch (cause) {
      say("err", errorText(cause));
    }
  }

  function jumpToLine(line: number) {
    if (!activeTab) return;
    activeTab.revealLine = line;
    revealKey += 1;
  }

  function updateContent(path: string, value: string) {
    const tab = tabs.find((item) => item.path === path);
    if (tab) tab.content = value;
  }

  async function save() {
    const tab = activeTab;
    if (!tab || tab.content === tab.original) return;
    try {
      await saveFile(tab.path, tab.content);
      tab.original = tab.content;
      skills = await scanSkills();
      await refreshGit();
      say("ok", `Saved ${tab.name}`);
    } catch (cause) {
      say("err", errorText(cause));
    }
  }

  function closeTab(path: string) {
    const index = tabs.findIndex((tab) => tab.path === path);
    if (index === -1) return;
    const tab = tabs[index];
    if (tab.content !== tab.original && !confirm(`Discard unsaved changes to ${tab.name}?`)) {
      return;
    }
    const next = tabs.filter((item) => item.path !== path);
    tabs = next;
    if (activePath === path) {
      activePath = (next[index] ?? next[index - 1])?.path ?? null;
    }
  }

  function toggleDir(path: string) {
    const next = new Set(collapsed);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    collapsed = next;
  }

  function focusTab(path: string) {
    activePath = path;
    const tab = tabs.find((item) => item.path === path);
    if (tab) tab.revealLine = 0;
  }

  function handleWindowClick(event: MouseEvent) {
    const target = event.target;
    if (target instanceof Element && target.closest("[data-menu-root]")) return;
    menu = null;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      menu = null;
      paletteOpen = false;
      return;
    }
    const mod = event.metaKey || event.ctrlKey;
    if (!mod) return;
    const key = event.key.toLowerCase();
    if (key === "o") {
      event.preventDefault();
      void openFolder();
    } else if (key === "s") {
      event.preventDefault();
      void save();
    } else if (key === "k" || key === "p") {
      event.preventDefault();
      menu = null;
      paletteOpen = true;
    } else if (key === "b") {
      event.preventDefault();
      sidebarOpen = !sidebarOpen;
    } else if (key === "w") {
      event.preventDefault();
      closeActiveTab();
    } else if (event.key === ",") {
      event.preventDefault();
      sidebarView = "settings";
    }
  }

  function closeActiveTab() {
    if (activePath) closeTab(activePath);
  }

  let refreshTimer: ReturnType<typeof setTimeout> | undefined;
  function scheduleRefresh() {
    clearTimeout(refreshTimer);
    refreshTimer = setTimeout(() => void refresh(), 320);
  }

  onMount(() => {
    applyTheme(theme);
    let unlisten: UnlistenFn | undefined;
    const narrow = window.matchMedia("(max-width: 900px)");
    const syncSidebar = () => {
      if (narrow.matches) sidebarOpen = false;
    };
    syncSidebar();
    narrow.addEventListener("change", syncSidebar);

    if (!isTauri()) {
      say("err", "Browser preview: file access needs the desktop app. Run `npm run tauri dev`.");
      return () => {
        clearTimeout(refreshTimer);
        clearTimeout(noticeTimer);
      };
    }

    (async () => {
      try {
        const stored =
          localStorage.getItem("codemd:workspace") ?? localStorage.getItem("trpsc:workspace");
        const known = stored ?? (await workspaceRoot());
        if (known) {
          await useWorkspace(known);
        }
      } catch {
        localStorage.removeItem("codemd:workspace");
        localStorage.removeItem("trpsc:workspace");
      }

      try {
        unlisten = await listen("workspace-changed", scheduleRefresh);
      } catch (cause) {
        say("err", errorText(cause));
      }
    })();

    return () => {
      unlisten?.();
      narrow.removeEventListener("change", syncSidebar);
      clearTimeout(refreshTimer);
      clearTimeout(noticeTimer);
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} onclick={handleWindowClick} />

<div class="app flex h-screen flex-col bg-app" class:select-none={resizingPreview}>
  <header class="flex h-9 shrink-0 items-center border-b border-line bg-panel" data-tauri-drag-region>
    <div class="flex w-[148px] shrink-0 select-none items-center gap-2 pl-4" data-tauri-drag-region>
      <img class="size-[22px] rounded-[5px]" src="/code-md-mark.svg" alt="" aria-hidden="true" />
      <span class="text-[13px] font-semibold tracking-[-0.01em] text-ink">CODE<span class="text-faint">.MD</span></span>
    </div>

    <div class="flex items-center gap-1 px-1" data-menu-root>
      <div class="relative">
        <button class="h-6 rounded-md bg-elev px-2.5 text-xs text-muted transition-colors duration-100 hover:bg-hover hover:text-ink" class:bg-hover={menu === "file"} class:text-ink={menu === "file"} type="button" onclick={() => (menu = menu === "file" ? null : "file")}>File</button>
        {#if menu === "file"}
          <div class="absolute top-full left-0 z-30 mt-1 w-60 rounded-lg border border-line bg-elev p-1 shadow-[0_16px_40px_rgba(0,0,0,.55)]">
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink" type="button" onclick={() => { menu = null; void openFolder(); }}><Icon name="folder" size={14} />Open folder<span class="ml-auto font-mono text-[10px] text-faint">Ctrl O</span></button>
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink disabled:opacity-40" type="button" disabled={!dirty} onclick={() => { menu = null; void save(); }}><Icon name="save" size={14} />Save<span class="ml-auto font-mono text-[10px] text-faint">Ctrl S</span></button>
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink disabled:opacity-40" type="button" disabled={!activeTab} onclick={() => { menu = null; closeActiveTab(); }}><Icon name="close" size={14} />Close tab<span class="ml-auto font-mono text-[10px] text-faint">Ctrl W</span></button>
            <div class="my-1 h-px bg-line"></div>
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink disabled:opacity-40" type="button" disabled={!workspace || refreshing} onclick={() => { menu = null; void refresh(); }}><Icon name="refresh" size={14} />Rescan folder</button>
          </div>
        {/if}
      </div>

      <div class="relative">
        <button class="h-6 rounded-md bg-elev px-2.5 text-xs text-muted transition-colors duration-100 hover:bg-hover hover:text-ink" class:bg-hover={menu === "view"} class:text-ink={menu === "view"} type="button" onclick={() => (menu = menu === "view" ? null : "view")}>View</button>
        {#if menu === "view"}
          <div class="absolute top-full left-0 z-30 mt-1 w-60 rounded-lg border border-line bg-elev p-1 shadow-[0_16px_40px_rgba(0,0,0,.55)]">
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink disabled:opacity-40" type="button" disabled={!activeTab} onclick={() => { menu = null; previewOn = !previewOn; }}><Icon name={previewOn ? "eye-off" : "eye"} size={14} />{previewOn ? "Hide preview" : "Show preview"}</button>
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink" type="button" onclick={() => { menu = null; sidebarView = "outline"; }}><Icon name="outline" size={14} />Outline</button>
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink" type="button" onclick={() => { menu = null; sidebarView = "skills"; }}><Icon name="skill" size={14} />Skill audit</button>
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink" type="button" onclick={() => { menu = null; sidebarView = "search"; }}><Icon name="search" size={14} />Search</button>
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink disabled:opacity-40" type="button" disabled={!workspace} onclick={() => { menu = null; sidebarView = "git"; sidebarOpen = true; void refreshGit(); }}><Icon name="git" size={14} />Source control</button>
            <div class="my-1 h-px bg-line"></div>
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink" type="button" onclick={() => { menu = null; sidebarView = "settings"; }}><Icon name="settings" size={14} />Settings<span class="ml-auto font-mono text-[10px] text-faint">Ctrl ,</span></button>
          </div>
        {/if}
      </div>

      <div class="relative">
        <button class="h-6 rounded-md bg-elev px-2.5 text-xs text-muted transition-colors duration-100 hover:bg-hover hover:text-ink" class:bg-hover={menu === "go"} class:text-ink={menu === "go"} type="button" onclick={() => (menu = menu === "go" ? null : "go")}>Go</button>
        {#if menu === "go"}
          <div class="absolute top-full left-0 z-30 mt-1 w-60 rounded-lg border border-line bg-elev p-1 shadow-[0_16px_40px_rgba(0,0,0,.55)]">
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink" type="button" onclick={() => { menu = null; sidebarView = "files"; }}><Icon name="folder" size={14} />Documents</button>
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink" type="button" onclick={() => { menu = null; sidebarView = "outline"; }}><Icon name="outline" size={14} />Outline</button>
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink" type="button" onclick={() => { menu = null; sidebarView = "search"; }}><Icon name="search" size={14} />Search</button>
            <button class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-xs text-muted hover:bg-hover hover:text-ink disabled:opacity-40" type="button" disabled={!workspace} onclick={() => { menu = null; sidebarView = "git"; sidebarOpen = true; void refreshGit(); }}><Icon name="git" size={14} />Source control</button>
          </div>
        {/if}
      </div>
    </div>

    <div class="flex min-w-0 flex-1 items-center gap-2 px-4 text-xs max-[980px]:hidden" data-tauri-drag-region>
      {#if workspace}
        <span class="truncate font-medium text-muted">{workspaceName}</span>
        {#if sidebarView === "settings"}
          <span class="text-faint">/</span>
          <span class="shrink-0 text-ink">Settings</span>
        {:else if sidebarView === "git"}
          <span class="text-faint">/</span>
          <span class="shrink-0 text-ink">Source control</span>
        {:else if activeTab}
          <span class="text-faint">/</span>
          <span class="truncate text-ink">{activeTab.name}</span>
          {#if dirty}<span class="size-1.5 shrink-0 rounded-full bg-accent" title="Unsaved changes"></span>{/if}
        {/if}
      {:else}
        <span class="text-faint">No folder open</span>
      {/if}
    </div>

    <button
      class="mr-2 flex h-6 w-[250px] shrink-0 items-center gap-2 rounded-md border border-line bg-elev px-2.5 text-left text-xs text-faint transition-colors duration-100 hover:border-line-strong hover:bg-hover hover:text-muted max-[1080px]:w-auto max-[1080px]:border-0 max-[1080px]:bg-transparent"
      type="button"
      title="Search files or run a command (Ctrl K)"
      onclick={() => { menu = null; paletteOpen = true; }}
    >
      <Icon name="search" size={14} />
      <span class="truncate max-[1080px]:hidden">Search or jump to</span>
      <span class="ml-auto flex items-center gap-1 max-[1080px]:hidden">
        <kbd class="font-mono text-[10px]">Ctrl</kbd><kbd class="font-mono text-[10px]">K</kbd>
      </span>
    </button>

    <div class="flex shrink-0 items-center gap-0.5 px-1">
      <button class="grid size-7 place-items-center rounded-md text-muted transition-colors duration-100 hover:bg-elev hover:text-ink" type="button" title="Toggle sidebar (Ctrl B)" aria-label="Toggle sidebar" aria-pressed={sidebarOpen} onclick={() => (sidebarOpen = !sidebarOpen)}>
        <Icon name="sidebar" size={16} />
      </button>
      <button class="grid size-7 place-items-center rounded-md text-muted transition-colors duration-100 enabled:hover:bg-elev enabled:hover:text-ink disabled:opacity-35" type="button" title="Save (Ctrl S)" aria-label="Save" disabled={!dirty} onclick={save}>
        <Icon name="save" size={15} />
      </button>
      <button class="grid size-7 place-items-center rounded-md transition-colors duration-100 {previewOn ? 'bg-elev text-ink' : 'text-muted hover:bg-elev hover:text-ink'}" type="button" title={previewOn ? "Hide preview" : "Show preview"} aria-label={previewOn ? "Hide preview" : "Show preview"} aria-pressed={previewOn} disabled={!activeTab} onclick={() => (previewOn = !previewOn)}>
        <Icon name={previewOn ? "eye" : "eye-off"} size={15} />
      </button>
    </div>

    {#if desktop}
      <div class="ml-1 flex h-full shrink-0 items-center border-l border-line">
        <button class="grid h-full w-9 place-items-center text-muted transition-colors duration-100 hover:bg-elev hover:text-ink" type="button" title="Minimize" aria-label="Minimize" onclick={() => void windowAction("minimize")}>
          <Icon name="minus" size={15} />
        </button>
        <button class="grid h-full w-9 place-items-center text-muted transition-colors duration-100 hover:bg-elev hover:text-ink" type="button" title="Maximize" aria-label="Maximize" onclick={() => void windowAction("maximize")}>
          <Icon name="maximize" size={13} />
        </button>
        <button class="grid h-full w-9 place-items-center text-muted transition-colors duration-100 hover:bg-danger hover:text-white" type="button" title="Close" aria-label="Close" onclick={() => void windowAction("close")}>
          <Icon name="close" size={15} />
        </button>
      </div>
    {/if}
  </header>

  <div class="flex min-h-0 flex-1">
    {#if sidebarOpen && sidebarView !== "settings"}
      <div class="m-2 mr-0 flex w-[312px] shrink-0 flex-col">
        <nav class="mb-2 flex h-9 shrink-0 items-center gap-0.5 bg-transparent px-1" aria-label="Sidebar views">
          <button
            class="grid size-8 place-items-center rounded-md transition-colors duration-100 {sidebarView === 'files' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
            type="button"
            title="Project"
            aria-label="Project"
            aria-current={sidebarView === "files" ? "page" : undefined}
            onclick={() => { sidebarView = "files"; sidebarOpen = true; }}
          >
            <Icon name="folder" size={18} />
          </button>
          <button
            class="grid size-8 place-items-center rounded-md transition-colors duration-100 {sidebarView === 'outline' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
            type="button"
            title="Outline"
            aria-label="Outline"
            aria-current={sidebarView === "outline" ? "page" : undefined}
            onclick={() => { sidebarView = "outline"; sidebarOpen = true; }}
          >
            <Icon name="outline" size={18} />
          </button>
          <button
            class="grid size-8 place-items-center rounded-md transition-colors duration-100 {sidebarView === 'skills' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
            type="button"
            title="Skill audit"
            aria-label="Skill audit"
            aria-current={sidebarView === "skills" ? "page" : undefined}
            onclick={() => { sidebarView = "skills"; sidebarOpen = true; }}
          >
            <Icon name="skill" size={18} />
          </button>
          <button
            class="grid size-8 place-items-center rounded-md transition-colors duration-100 {sidebarView === 'search' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
            type="button"
            title="Search in folder"
            aria-label="Search in folder"
            aria-current={sidebarView === "search" ? "page" : undefined}
            onclick={() => { sidebarView = "search"; sidebarOpen = true; }}
          >
            <Icon name="search" size={18} />
          </button>
          <button
            class="relative grid size-8 place-items-center rounded-md transition-colors duration-100 {sidebarView === 'git' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
            type="button"
            title="Source control"
            aria-label="Source control"
            aria-current={sidebarView === "git" ? "page" : undefined}
            onclick={() => { sidebarView = "git"; sidebarOpen = true; void refreshGit(); }}
          >
            <Icon name="git" size={18} />
            {#if git && git.files.length > 0}
              <span class="absolute top-1 right-1 size-1.5 rounded-full bg-ink"></span>
            {/if}
          </button>

          <div class="ml-auto flex items-center gap-0.5">
            <button
              class="grid size-8 place-items-center rounded-md text-faint transition-colors duration-100 hover:bg-elev hover:text-ink"
              type="button"
              title="Quick open (Ctrl K)"
              aria-label="Quick open"
              onclick={() => { menu = null; paletteOpen = true; }}
            >
              <Icon name="command" size={18} />
            </button>
            <button
              class="grid size-8 place-items-center rounded-md text-faint transition-colors duration-100 hover:bg-elev hover:text-ink"
              type="button"
              title="Settings (Ctrl ,)"
              aria-label="Settings"
              onclick={() => (sidebarView = "settings")}
            >
              <Icon name="settings" size={18} />
            </button>
          </div>
        </nav>

        <div class="sidebar-surface group/sidebar relative flex min-h-0 flex-1 flex-col overflow-hidden rounded-2xl border-[1.3px] border-line bg-app shadow-[0_18px_50px_rgba(0,0,0,0.35),inset_0_1px_0_rgba(255,255,255,0.07)]">
          <div class="sidebar-aurora pointer-events-none absolute inset-x-0 top-0 h-40"></div>
          <div class="pointer-events-none absolute inset-x-4 top-0 h-px bg-gradient-to-r from-transparent via-ink/25 to-transparent"></div>

          <aside class="relative z-10 flex min-w-0 flex-1 flex-col bg-transparent">
          <div class="flex h-10 shrink-0 items-center gap-2 border-b border-line pr-2 pl-3">
            <span class="text-xs font-semibold text-ink">
              {#if sidebarView === "files"}Project{:else if sidebarView === "outline"}Outline{:else if sidebarView === "skills"}Skill audit{:else if sidebarView === "git"}Source control{:else}Search{/if}
            </span>
            {#if sidebarView === "files" && workspace}
              <span class="text-[11px] tabular-nums text-faint">{tree.filter((entry) => !entry.isDir).length}</span>
            {/if}
            {#if sidebarView === "git" && git}
              <span class="text-[11px] tabular-nums text-faint">{git.branch}</span>
            {/if}
            {#if sidebarView === "skills" && invalidSkills > 0}
              <span class="text-[11px] font-medium tabular-nums text-danger" title="{invalidSkills} skills need fixes">{invalidSkills}</span>
            {/if}
            {#if refreshing}
              <span class="text-[10px] text-faint">scanning</span>
            {/if}
            <div class="ml-auto flex items-center gap-0.5">
              <button class="grid size-6 place-items-center rounded-md text-faint transition-colors duration-100 enabled:hover:bg-hover enabled:hover:text-ink disabled:opacity-40" type="button" title="Rescan folder" aria-label="Rescan folder" disabled={!workspace || refreshing} onclick={() => void refresh()}>
                <Icon name="refresh" size={13} />
              </button>
              <button class="grid size-6 place-items-center rounded-md text-faint transition-colors duration-100 hover:bg-hover hover:text-ink" type="button" title="Hide sidebar (Ctrl B)" aria-label="Hide sidebar" onclick={() => (sidebarOpen = false)}>
                <Icon name="sidebar" size={14} />
              </button>
            </div>
          </div>

          <div class="min-h-0 flex-1 overflow-auto py-1">
            {#if !workspace}
              <div class="px-3.5 py-4">
                <p class="m-0 text-xs leading-relaxed text-faint">No folder open.</p>
                <button class="mt-3 inline-flex h-8 items-center gap-2 rounded-md bg-ink px-3 text-xs font-semibold text-app transition-opacity duration-100 hover:opacity-85" type="button" onclick={() => void openFolder()}>
                  <Icon name="folder" size={14} />
                  Open folder
                </button>
              </div>
            {:else if sidebarView === "files"}
              <FileTree
                entries={tree}
                {collapsed}
                activePath={activePath}
                onOpen={(path) => void openFile(path)}
                onToggleDir={toggleDir}
              />
            {:else if sidebarView === "outline"}
              <OutlineView headings={outline} hasDocument={activeTab !== null} onJump={jumpToLine} />
            {:else if sidebarView === "skills"}
              <SkillsView {skills} onOpen={(path) => void openFile(path)} />
            {:else if sidebarView === "git"}
              <GitView
                status={git}
                loading={gitLoading}
                error={gitError || gitActionError}
                busy={gitBusy}
                commits={gitCommits}
                selectedPath={gitSelected?.path ?? null}
                selectedStaged={gitSelectedStaged}
                onRefresh={() => void refreshGit()}
                onInit={() => void initializeGit()}
                onSelect={(file, staged) => void selectGitFile(file, staged)}
                onStage={(paths) => void stageSelected(paths)}
                onUnstage={(paths) => void unstageSelected(paths)}
                onCommit={commitStaged}
              />
            {:else}
              <SearchView onOpen={(path, line) => void openFile(path, line)} />
            {/if}
          </div>
          </aside>
        </div>
      </div>
    {:else}
      <nav class="flex w-12 shrink-0 flex-col items-center gap-0.5 bg-transparent py-2" aria-label="Sidebar views">
        <button
          class="grid size-8 place-items-center rounded-md transition-colors duration-100 {sidebarView === 'files' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
          type="button"
          title="Project"
          aria-label="Project"
          aria-current={sidebarView === "files" ? "page" : undefined}
          onclick={() => { sidebarView = "files"; sidebarOpen = true; }}
        >
          <Icon name="folder" size={18} />
        </button>
        <button
          class="grid size-8 place-items-center rounded-md transition-colors duration-100 {sidebarView === 'outline' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
          type="button"
          title="Outline"
          aria-label="Outline"
          aria-current={sidebarView === "outline" ? "page" : undefined}
          onclick={() => { sidebarView = "outline"; sidebarOpen = true; }}
        >
          <Icon name="outline" size={18} />
        </button>
        <button
          class="grid size-8 place-items-center rounded-md transition-colors duration-100 {sidebarView === 'skills' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
          type="button"
          title="Skill audit"
          aria-label="Skill audit"
          aria-current={sidebarView === "skills" ? "page" : undefined}
          onclick={() => { sidebarView = "skills"; sidebarOpen = true; }}
        >
          <Icon name="skill" size={18} />
        </button>
        <button
          class="grid size-8 place-items-center rounded-md transition-colors duration-100 {sidebarView === 'search' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
          type="button"
          title="Search in folder"
          aria-label="Search in folder"
          aria-current={sidebarView === "search" ? "page" : undefined}
          onclick={() => { sidebarView = "search"; sidebarOpen = true; }}
        >
          <Icon name="search" size={18} />
        </button>
        <button
          class="relative grid size-8 place-items-center rounded-md transition-colors duration-100 {sidebarView === 'git' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
          type="button"
          title="Source control"
          aria-label="Source control"
          aria-current={sidebarView === "git" ? "page" : undefined}
          onclick={() => { sidebarView = "git"; sidebarOpen = true; void refreshGit(); }}
        >
          <Icon name="git" size={18} />
          {#if git && git.files.length > 0}
            <span class="absolute top-1 right-1 size-1.5 rounded-full bg-ink"></span>
          {/if}
        </button>

        <div class="mt-auto flex flex-col items-center gap-0.5">
          <button
            class="grid size-8 place-items-center rounded-md text-faint transition-colors duration-100 hover:bg-elev hover:text-ink"
            type="button"
            title="Quick open (Ctrl K)"
            aria-label="Quick open"
            onclick={() => { menu = null; paletteOpen = true; }}
          >
            <Icon name="command" size={18} />
          </button>
          <button
            class="grid size-8 place-items-center rounded-md transition-colors duration-100 {sidebarView === 'settings' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
            type="button"
            title="Settings (Ctrl ,)"
            aria-label="Settings"
            aria-current={sidebarView === "settings" ? "page" : undefined}
            onclick={() => (sidebarView = "settings")}
          >
            <Icon name="settings" size={18} />
          </button>
        </div>
      </nav>
    {/if}

    <main class="ml-2 flex min-h-0 min-w-0 flex-1 flex-col bg-panel">
      {#if sidebarView === "git" && gitSelected}
        <DiffView
          file={gitSelected}
          diff={gitDiff}
          staged={gitSelectedStaged}
          loading={gitDiffLoading}
          error={gitDiffError}
          onToggleStaged={() => void toggleGitDiffStage()}
          onOpenFile={() => {
            const path = gitSelected?.path;
            closeGitDiff();
            if (path) void openFile(path);
          }}
          onClose={closeGitDiff}
        />
      {:else if sidebarView === "settings"}
        <div class="min-h-0 flex-1 overflow-hidden">
          <SettingsView
            {theme}
            {workspace}
            {workspaceName}
            fileCount={tree.filter((entry) => !entry.isDir).length}
            skillCount={skills.length}
            {refreshing}
            onTheme={applyTheme}
            onRescan={() => void refresh()}
            onForget={() => void forgetWorkspace()}
          />
        </div>
      {:else}
        {#if activeTab}
          <div class="flex h-[38px] shrink-0 items-stretch overflow-x-auto overflow-y-hidden border-b border-line bg-panel [scrollbar-width:none] [&::-webkit-scrollbar]:hidden" role="tablist" aria-label="Open files">
            {#each tabs as tab (tab.path)}
              <div class="group/tab relative flex max-w-[230px] shrink-0 items-center border-r border-line {tab.path === activePath ? 'bg-editor after:absolute after:inset-x-0 after:top-0 after:h-px after:bg-accent' : 'hover:bg-elev'}">
                <button
                  class="flex h-full min-w-0 items-center gap-2 py-0 pr-1 pl-3 text-xs {tab.path === activePath ? 'text-ink' : 'text-faint hover:text-muted'}"
                  type="button"
                  role="tab"
                  aria-selected={tab.path === activePath}
                  title={tab.path}
                  onclick={() => focusTab(tab.path)}
                >
                  {#if tab.content !== tab.original}
                    <span class="size-1.5 shrink-0 rounded-full bg-accent" aria-label="Unsaved changes"></span>
                  {/if}
                  <span class="flex size-[15px] shrink-0 items-center justify-center"><FileGlyph path={tab.path} size={15} /></span>
                  <span class="truncate">{tab.name}</span>
                </button>
                <button
                  class="mr-1.5 grid size-5 shrink-0 place-items-center rounded text-faint opacity-0 transition-opacity duration-100 hover:bg-hover hover:text-ink focus-visible:opacity-100 group-hover/tab:opacity-100 {tab.path === activePath ? 'opacity-100' : ''}"
                  type="button"
                  title="Close {tab.name}"
                  aria-label="Close {tab.name}"
                  onclick={() => closeTab(tab.path)}
                >
                  <Icon name="close" size={12} />
                </button>
              </div>
            {/each}
            <button class="ml-auto grid w-9 shrink-0 place-items-center border-l border-line text-faint transition-colors duration-100 hover:bg-elev hover:text-ink" type="button" title="Search files (Ctrl K)" aria-label="Search files" onclick={() => (paletteOpen = true)}>
              <Icon name="search" size={14} />
            </button>
          </div>
        {/if}

        <div class="flex min-h-0 flex-1" bind:this={splitEl}>
          {#if activeTab}
            <section class="min-h-0 min-w-0 flex-1 overflow-hidden" aria-label="Editor">
              {#key activeTab.path}
                <Editor
                  path={activeTab.path}
                  value={activeTab.content}
                  revealLine={activeTab.revealLine}
                  {revealKey}
                  onChange={(value) => updateContent(activeTab.path, value)}
                  onSave={save}
                />
              {/key}
            </section>
            {#if previewOn}
              <div
                class="preview-resizer group relative -ml-px z-10 w-[5px] shrink-0 cursor-col-resize bg-transparent max-[1100px]:hidden"
                role="slider"
                tabindex="0"
                aria-label="Preview width"
                aria-orientation="vertical"
                aria-valuemin={Math.round(MIN_PREVIEW_RATIO * 100)}
                aria-valuemax={Math.round(MAX_PREVIEW_RATIO * 100)}
                aria-valuenow={Math.round(previewRatio * 100)}
                title="Drag to resize preview, double-click to reset"
                onpointerdown={startPreviewResize}
                onpointermove={movePreviewResize}
                onpointerup={finishPreviewResize}
                onpointercancel={finishPreviewResize}
                ondblclick={resetPreviewResize}
                onkeydown={(event) => {
                  if (event.key === "ArrowLeft") {
                    event.preventDefault();
                    nudgePreviewResize(0.02);
                  } else if (event.key === "ArrowRight") {
                    event.preventDefault();
                    nudgePreviewResize(-0.02);
                  } else if (event.key === "Home") {
                    event.preventDefault();
                    resetPreviewResize();
                  }
                }}
              >
                <span class="absolute inset-y-0 left-1/2 w-px -translate-x-1/2 bg-line transition-colors duration-100 group-hover:bg-line-strong group-focus-visible:bg-ink {resizingPreview ? 'bg-ink' : ''}"></span>
              </div>
              <section class="min-h-0 min-w-[280px] shrink-0 overflow-hidden" style="width: {previewRatio * 100}%" aria-label="Preview">
                <Preview content={activeTab.content} path={activeTab.path} />
              </section>
            {/if}
          {:else}
            <div class="flex min-h-0 flex-1 items-center justify-center overflow-auto bg-editor px-6 py-10">
              <div class="w-full max-w-[520px]">
                <span class="mb-5 grid size-9 place-items-center rounded-lg border border-line bg-panel text-muted">
                  <Icon name={workspace ? "markdown" : "folder"} size={18} />
                </span>
                {#if workspace}
                  <h1 class="m-0 text-[26px] font-semibold leading-[1.2] tracking-[-0.025em] text-ink">Choose a document</h1>
                  <p class="mt-2.5 max-w-[48ch] text-[13.5px] leading-relaxed text-muted">
                    Pick a file in the sidebar, or search the project without leaving the keyboard.
                  </p>
                  <div class="mt-6 flex flex-wrap items-center gap-2">
                    <button class="inline-flex h-8 items-center gap-2 rounded-md bg-ink px-3 text-xs font-semibold text-app transition-opacity duration-100 hover:opacity-85" type="button" onclick={() => (paletteOpen = true)}>
                      <Icon name="search" size={14} />
                      Search files
                      <kbd class="ml-1 font-mono text-[10px] opacity-60">Ctrl K</kbd>
                    </button>
                    <button class="inline-flex h-8 items-center gap-2 rounded-md border border-line px-3 text-xs font-medium text-muted transition-colors duration-100 hover:border-line-strong hover:text-ink" type="button" onclick={() => void openFolder()}>
                      Open another folder
                    </button>
                  </div>
                {:else}
                  <h1 class="m-0 text-[26px] font-semibold leading-[1.2] tracking-[-0.025em] text-ink">Open a project</h1>
                  <p class="mt-2.5 max-w-[48ch] text-[13.5px] leading-relaxed text-muted">
                    Choose a folder to browse its Markdown, text, and code files. Markdown gets a live
                    preview and outline; skill files are checked in the same tree.
                  </p>
                  <div class="mt-6 flex flex-wrap items-center gap-2">
                    <button class="inline-flex h-8 items-center gap-2 rounded-md bg-ink px-3 text-xs font-semibold text-app transition-opacity duration-100 hover:opacity-85" type="button" onclick={() => void openFolder()}>
                      <Icon name="folder" size={14} />
                      Open folder
                      <kbd class="ml-1 font-mono text-[10px] opacity-60">Ctrl O</kbd>
                    </button>
                  </div>
                {/if}
                <dl class="mt-8 border-t border-line">
                  <div class="flex h-9 items-center justify-between border-b border-line text-xs"><dt class="text-faint">Quick search</dt><dd class="m-0 flex items-center gap-1 font-mono text-[10px] text-muted"><kbd>Ctrl</kbd><kbd>K</kbd></dd></div>
                  <div class="flex h-9 items-center justify-between border-b border-line text-xs"><dt class="text-faint">Toggle sidebar</dt><dd class="m-0 flex items-center gap-1 font-mono text-[10px] text-muted"><kbd>Ctrl</kbd><kbd>B</kbd></dd></div>
                  <div class="flex h-9 items-center justify-between border-b border-line text-xs"><dt class="text-faint">Save file</dt><dd class="m-0 flex items-center gap-1 font-mono text-[10px] text-muted"><kbd>Ctrl</kbd><kbd>S</kbd></dd></div>
                </dl>
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </main>
  </div>

  <footer class="flex h-7 shrink-0 items-center gap-3 border-t border-line bg-panel px-3 text-[11px] tabular-nums text-faint">
    <span class="flex min-w-0 items-center gap-1.5">
      <Icon name="folder" size={13} />
      <span class="truncate" title={workspace ?? "No folder open"}>{workspaceName || "No folder open"}</span>
    </span>
    {#if workspace}
      <span>{tree.filter((entry) => !entry.isDir).length} files</span>
      <span>{skills.length} {skills.length === 1 ? "skill" : "skills"}{#if invalidSkills > 0}<span class="text-danger"> · {invalidSkills} {invalidSkills === 1 ? "issue" : "issues"}</span>{/if}</span>
      {#if git}
        <button class="flex shrink-0 items-center gap-1 rounded px-1 text-faint transition-colors duration-100 hover:bg-elev hover:text-ink" type="button" title="Show source control" onclick={() => { sidebarView = "git"; sidebarOpen = true; }}>
          <Icon name="git" size={12} />{git.branch}
        </button>
        {#if !git.clean}
          <button class="shrink-0 rounded px-1 text-faint transition-colors duration-100 hover:bg-elev hover:text-ink" type="button" title="Show source control" onclick={() => { sidebarView = "git"; sidebarOpen = true; }}>
            {git.files.length} {git.files.length === 1 ? "change" : "changes"}
          </button>
        {/if}
      {:else if gitLoading}
        <span>checking git</span>
      {/if}
    {/if}
    {#if notice}
      <span class="min-w-0 truncate {notice.kind === 'ok' ? 'text-ok' : 'text-danger'}" aria-live="polite">{notice.text}</span>
    {/if}
    <span class="ml-auto flex shrink-0 items-center gap-3">
      {#if activeTab && stats}
        <span>{stats.words} {stats.words === 1 ? "word" : "words"} · {stats.minutes} min</span>
      {/if}
      {#if activeTab}
        <span class="max-[760px]:hidden">{isSkillPath(activeTab.path) ? "SKILL.md" : isMarkdownPath(activeTab.path) ? "Markdown" : "Plain text"}</span>
        <span>{dirty ? "Unsaved" : "Saved"}</span>
      {/if}
      <span>UTF-8</span>
    </span>
  </footer>
</div>

{#if paletteOpen}
  <CommandPalette
    files={paletteFiles}
    commands={paletteCommands}
    onClose={() => (paletteOpen = false)}
    onOpen={(path) => void openFile(path)}
  />
{/if}
