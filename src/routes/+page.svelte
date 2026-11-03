<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import {
    closeWorkspace,
    isTauri,
    openWorkspace,
    pickWorkspace,
    readFile,
    saveFile,
    scanSkills,
    scanTree,
    workspaceRoot,
  } from "$lib/api";
  import { isMarkdownPath } from "$lib/files";
  import { ancestorsOf } from "$lib/tree";
  import { documentStats, extractOutline } from "$lib/outline";
  import type { SidebarView, SkillInfo, TreeEntry } from "$lib/types";
  import Editor from "$lib/components/Editor.svelte";
  import FileTree from "$lib/components/FileTree.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import OutlineView from "$lib/components/OutlineView.svelte";
  import Preview from "$lib/components/Preview.svelte";
  import SearchView from "$lib/components/SearchView.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import SkillsView from "$lib/components/SkillsView.svelte";

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

  let workspace = $state<string | null>(null);
  let tree = $state<TreeEntry[]>([]);
  let skills = $state<SkillInfo[]>([]);
  let tabs = $state<Tab[]>([]);
  let activePath = $state<string | null>(null);
  let sidebarView = $state<SidebarView>("files");
  let theme = $state<Theme>(storedTheme());
  let collapsed = $state(new Set<string>());
  let previewOn = $state(true);
  let revealKey = $state(0);
  let refreshing = $state(false);
  let notice = $state<{ kind: "ok" | "err"; text: string } | null>(null);
  let noticeTimer: ReturnType<typeof setTimeout> | undefined;

  const desktop = browser && isTauri();
  const activeTab = $derived(tabs.find((tab) => tab.path === activePath) ?? null);
  const dirty = $derived(activeTab ? activeTab.content !== activeTab.original : false);
  const invalidSkills = $derived(skills.filter((skill) => !skill.valid).length);
  const outline = $derived(activeTab ? extractOutline(activeTab.content) : []);
  const stats = $derived(activeTab ? documentStats(activeTab.content) : null);
  const workspaceName = $derived(
    workspace ? (workspace.split("/").filter(Boolean).pop() ?? workspace) : "",
  );

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
    sidebarView = "files";
    localStorage.removeItem("codemd:workspace");
    say("ok", "Folder closed");
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

  function handleKeydown(event: KeyboardEvent) {
    const mod = event.metaKey || event.ctrlKey;
    if (!mod) return;
    if (event.key.toLowerCase() === "o") {
      event.preventDefault();
      void openFolder();
    } else if (event.key.toLowerCase() === "s") {
      event.preventDefault();
      void save();
    }
  }

  let refreshTimer: ReturnType<typeof setTimeout> | undefined;
  function scheduleRefresh() {
    clearTimeout(refreshTimer);
    refreshTimer = setTimeout(() => void refresh(), 320);
  }

  onMount(() => {
    applyTheme(theme);
    let unlisten: UnlistenFn | undefined;

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
      clearTimeout(refreshTimer);
      clearTimeout(noticeTimer);
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app grid h-screen grid-rows-[var(--top-h)_44px_minmax(0,1fr)] bg-app px-3 pt-2 pb-3">
  <header class="flex h-11 items-center gap-3 border-b border-line bg-panel px-3" data-tauri-drag-region>
    <div class="flex w-rail shrink-0 select-none items-center justify-center text-sm font-semibold text-ink" aria-label="CODE.MD" data-tauri-drag-region>CODE<span class="text-accent">.MD</span></div>

    <div class="flex min-w-0 flex-1 items-baseline gap-2 pl-0.5 text-faint" data-tauri-drag-region>
      {#if workspace}
        <Icon name="folder" size={15} />
        <span class="whitespace-nowrap font-medium text-ink">{workspaceName}</span>
        {#if sidebarView === "settings"}
          <span class="text-faint">/</span>
          <span class="truncate font-medium text-muted">Settings</span>
        {:else if activeTab}
          <span class="text-faint">/</span>
          <span class="truncate font-medium text-muted">{activeTab.name}</span>
        {/if}
        <span class="truncate font-mono text-[11px] text-faint max-[1080px]:hidden">{workspace}</span>
      {:else}
        <span class="text-xs text-faint">No folder open</span>
      {/if}
    </div>

    {#if desktop}
      <div class="flex items-center">
        <button class="grid size-8 place-items-center rounded-md text-muted transition-colors duration-150 hover:bg-elev hover:text-ink" type="button" title="Minimize" aria-label="Minimize" onclick={() => void windowAction("minimize")}>
          <Icon name="minus" size={15} />
        </button>
        <button class="grid size-8 place-items-center rounded-md text-muted transition-colors duration-150 hover:bg-elev hover:text-ink" type="button" title="Maximize" aria-label="Maximize" onclick={() => void windowAction("maximize")}>
          <Icon name="maximize" size={13} />
        </button>
        <button class="grid size-8 place-items-center rounded-md text-muted transition-colors duration-150 hover:bg-danger hover:text-white" type="button" title="Close" aria-label="Close" onclick={() => void windowAction("close")}>
          <Icon name="close" size={15} />
        </button>
      </div>
    {/if}
  </header>

  <div class="flex h-11 items-center gap-1 border-b border-line bg-app px-3">
    <button class="inline-flex h-8 items-center gap-2 rounded-md bg-accent px-3 text-xs font-semibold text-accent-ink transition-colors duration-150 hover:bg-accent-strong" type="button" onclick={openFolder}>
      <Icon name="folder" size={15} />
      <span>Open folder</span>
    </button>
    <button class="inline-flex h-8 items-center gap-2 rounded-md px-2.5 text-xs font-medium text-muted transition-colors duration-150 enabled:hover:bg-elev enabled:hover:text-ink disabled:opacity-40" type="button" disabled={!dirty} onclick={save}>
      <Icon name="save" size={15} />
      <span>Save</span>
    </button>
    <span class="mx-1 h-5 w-px bg-line"></span>
    <button class="inline-flex h-8 items-center gap-2 rounded-md px-2.5 text-xs font-medium transition-colors duration-150 {previewOn ? 'bg-elev text-ink' : 'text-muted hover:bg-elev hover:text-ink'}" type="button" onclick={() => (previewOn = !previewOn)}>
      <Icon name={previewOn ? "eye" : "eye-off"} size={15} />
      <span>Preview</span>
    </button>
    <button class="inline-flex h-8 items-center gap-2 rounded-md px-2.5 text-xs font-medium text-muted transition-colors duration-150 enabled:hover:bg-elev enabled:hover:text-ink disabled:opacity-40" type="button" disabled={!workspace || refreshing} onclick={() => void refresh()}>
      <Icon name="refresh" size={15} />
      <span>Rescan</span>
    </button>

    <div class="ml-auto flex items-center gap-3 text-[11px] tabular-nums text-faint">
      {#if activeTab && stats}
        <span>{stats.words} {stats.words === 1 ? "word" : "words"}</span>
        <span>{stats.minutes} min read</span>
      {/if}
      {#if workspace}
        <span>{skills.length} {skills.length === 1 ? "skill" : "skills"}{#if invalidSkills > 0} · {invalidSkills} {invalidSkills === 1 ? "needs a fix" : "need fixes"}{/if}</span>
      {/if}
    </div>
  </div>

  <div class="grid min-h-0 gap-2 {sidebarView === 'settings' ? 'grid-cols-[var(--rail-w)_minmax(0,1fr)]' : 'grid-cols-[var(--rail-w)_var(--side-w)_minmax(0,1fr)] max-[1080px]:grid-cols-[var(--rail-w)_240px_minmax(0,1fr)]'}">
    <nav class="flex flex-col items-center gap-1 py-2" aria-label="Sidebar views">
      <button
        class="grid size-10 place-items-center rounded-[10px] transition-colors duration-150 {sidebarView === 'files' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
        type="button"
        title="Documents"
        aria-label="Documents"
        aria-current={sidebarView === "files" ? "page" : undefined}
        onclick={() => (sidebarView = "files")}
      >
        <Icon name="folder" size={18} />
      </button>
      <button
        class="grid size-10 place-items-center rounded-[10px] transition-colors duration-150 {sidebarView === 'outline' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
        type="button"
        title="Outline"
        aria-label="Outline"
        aria-current={sidebarView === "outline" ? "page" : undefined}
        onclick={() => (sidebarView = "outline")}
      >
        <Icon name="outline" size={18} />
      </button>
      <button
        class="grid size-10 place-items-center rounded-[10px] transition-colors duration-150 {sidebarView === 'skills' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
        type="button"
        title="Skill audit"
        aria-label="Skill audit"
        aria-current={sidebarView === "skills" ? "page" : undefined}
        onclick={() => (sidebarView = "skills")}
      >
        <Icon name="skill" size={18} />
      </button>
      <button
        class="grid size-10 place-items-center rounded-[10px] transition-colors duration-150 {sidebarView === 'search' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
        type="button"
        title="Search"
        aria-label="Search"
        aria-current={sidebarView === "search" ? "page" : undefined}
        onclick={() => (sidebarView = "search")}
      >
        <Icon name="search" size={18} />
      </button>
      <div class="mt-auto">
        <button
          class="grid size-10 place-items-center rounded-[10px] transition-colors duration-150 {sidebarView === 'settings' ? 'bg-active text-ink' : 'text-faint hover:bg-elev hover:text-ink'}"
          type="button"
          title="Settings"
          aria-label="Settings"
          aria-current={sidebarView === "settings" ? "page" : undefined}
          onclick={() => (sidebarView = "settings")}
        >
          <Icon name="settings" size={18} />
        </button>
      </div>
    </nav>

    {#if sidebarView === "settings"}
      <div class="col-span-2 min-h-0 overflow-hidden rounded-xl border border-line bg-panel">
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
    <aside class="grid min-w-0 grid-rows-[40px_minmax(0,1fr)_auto]">
      <div class="flex items-center justify-between gap-2 pr-1 pl-3">
        <span class="text-[13px] font-medium text-ink">
          {#if sidebarView === "files"}Documents{:else if sidebarView === "outline"}Outline{:else if sidebarView === "skills"}Skill audit{:else}Search{/if}
        </span>
        <div class="flex items-center gap-1.5 pr-2">
          {#if sidebarView === "skills" && invalidSkills > 0}
            <span class="text-[11px] font-medium tabular-nums text-danger" title="{invalidSkills} skills need fixes">{invalidSkills}</span>
          {/if}
          {#if refreshing}
            <span class="text-[11px] text-faint">scanning</span>
          {/if}
        </div>
      </div>

      <div class="min-h-0 overflow-auto pt-0.5">
        {#if !workspace}
          <div class="px-3 py-4 text-xs leading-relaxed text-faint">
            <p class="m-0">Open a folder to see its Markdown, text, and code files.</p>
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
        {:else}
          <SearchView onOpen={(path, line) => void openFile(path, line)} />
        {/if}
      </div>

      <div class="mx-1 mt-1.5 flex min-w-0 items-center gap-2 overflow-hidden rounded-lg bg-elev px-2.5 py-2">
        <span class="flex shrink-0 text-faint"><Icon name="folder" size={16} /></span>
        {#if workspace}
          <span class="flex min-w-0 flex-1 flex-col gap-px" title={workspace}>
            <span class="truncate text-xs font-medium text-ink">{workspaceName}</span>
            <span class="truncate font-mono text-[11px] text-faint">{workspace}</span>
          </span>
          <button
            class="grid size-7 shrink-0 place-items-center rounded-md text-muted transition-colors duration-150 enabled:hover:bg-hover enabled:hover:text-ink disabled:opacity-45"
            type="button"
            title="Rescan folder"
            aria-label="Rescan folder"
            onclick={() => void refresh()}
            disabled={refreshing}
          >
            <Icon name="refresh" size={14} />
          </button>
        {:else}
          <span class="flex min-w-0 flex-1 flex-col gap-px">
            <span class="truncate text-xs font-medium text-ink">No folder open</span>
            <span class="truncate font-mono text-[11px] text-faint">Open one to begin</span>
          </span>
        {/if}
      </div>
    </aside>

    <main class="grid min-h-0 min-w-0 overflow-hidden rounded-xl border border-line bg-panel {workspace && tabs.length > 0 ? 'grid-rows-[var(--tab-h)_minmax(0,1fr)_var(--status-h)]' : 'grid-rows-[minmax(0,1fr)_var(--status-h)]'}">
      {#if workspace && tabs.length > 0}
      <div class="flex items-stretch overflow-x-auto overflow-y-hidden border-b border-line bg-panel [scrollbar-width:none] [&::-webkit-scrollbar]:hidden" role="tablist" aria-label="Open files">
        {#if tabs.length === 0}
          <span class="flex items-center px-4 text-xs text-faint">No open files</span>
        {:else}
          {#each tabs as tab (tab.path)}
            <div class="group/tab relative flex max-w-[230px] shrink-0 items-center border-r border-line bg-transparent {tab.path === activePath ? 'bg-editor after:absolute after:inset-x-3 after:bottom-0 after:h-0.5 after:rounded-t-sm after:bg-accent' : ''}">
              <button
                class="flex h-full min-w-0 items-center gap-2 py-0 pr-1 pl-3 text-xs text-faint hover:text-ink [&_svg]:shrink-0 [&_svg]:text-faint {tab.path === activePath ? 'text-ink' : ''}"
                type="button"
                role="tab"
                aria-selected={tab.path === activePath}
                title={tab.path}
                onclick={() => focusTab(tab.path)}
              >
                {#if tab.content !== tab.original}
                  <span class="size-1.5 shrink-0 rounded-full bg-accent" aria-label="Unsaved changes"></span>
                {/if}
                <Icon
                  name={tab.name.toLowerCase() === "skill.md"
                    ? "skill"
                    : isMarkdownPath(tab.path)
                      ? "markdown"
                      : "file"}
                  size={14}
                />
                <span class="truncate">{tab.name}</span>
              </button>
              <button
                class="mr-1 grid size-6 shrink-0 place-items-center rounded-[5px] text-faint opacity-0 transition-opacity duration-150 hover:bg-hover hover:text-ink focus-visible:opacity-100 group-hover/tab:opacity-100 {tab.path === activePath ? 'opacity-100' : ''}"
                type="button"
                title="Close {tab.name}"
                aria-label="Close {tab.name}"
                onclick={() => closeTab(tab.path)}
              >
                <Icon name="close" size={12} />
              </button>
            </div>
          {/each}
        {/if}
      </div>
      {/if}

      <div class="grid min-h-0 grid-cols-1 {previewOn && activeTab !== null ? 'min-[1101px]:grid-cols-2' : ''}">
        {#if activeTab}
          <section class="min-h-0 min-w-0 overflow-hidden" aria-label="Editor">
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
            <section class="min-h-0 min-w-0 overflow-hidden border-l border-line max-[1100px]:hidden" aria-label="Preview">
              <Preview content={activeTab.content} path={activeTab.path} />
            </section>
          {/if}
        {:else}
          <div class="grid h-full place-items-center overflow-auto bg-editor p-12 max-[1080px]:p-8">
            <div class="grid w-full max-w-[1040px] items-center gap-12 {!workspace ? 'min-[1280px]:grid-cols-[minmax(0,1fr)_380px]' : ''}">
              <div class="max-w-[460px] justify-self-center">
                {#if workspace}
                  <h1 class="m-0 mb-4 text-balance text-[30px] font-semibold leading-[1.15] tracking-[-0.02em] text-ink">Pick a file</h1>
                  <p class="m-0 mb-6 max-w-[46ch] text-sm leading-[1.65] text-muted">
                    Choose a file from the sidebar. Markdown gets a live preview and outline; other
                    text and code files open as editable plain text.
                  </p>
                {:else}
                  <h1 class="m-0 mb-4 text-balance text-[30px] font-semibold leading-[1.15] tracking-[-0.02em] text-ink">One window for the whole project</h1>
                  <p class="m-0 mb-6 max-w-[46ch] text-sm leading-[1.65] text-muted">
                    Open a folder to browse its Markdown, text, and code files. Markdown gets a live
                    preview and outline; skill files are audited from the same tree.
                  </p>
                  <button class="inline-flex h-9 items-center gap-2 whitespace-nowrap rounded-lg border border-accent bg-accent px-4 text-[13px] font-semibold text-accent-ink transition-colors duration-150 hover:border-accent-strong hover:bg-accent-strong" type="button" onclick={openFolder}>
                    <Icon name="folder" size={16} />
                    <span>Open a folder</span>
                  </button>
                  <dl class="mt-8 border-t border-line">
                    <div class="flex items-center justify-between gap-4 border-b border-line py-3"><dt class="text-xs text-faint">Open folder</dt><dd class="m-0 flex items-center gap-1"><kbd class="inline-grid h-5 min-w-5 place-items-center rounded bg-elev px-[5px] font-mono text-[11px] tabular-nums text-muted">Ctrl</kbd><kbd class="inline-grid h-5 min-w-5 place-items-center rounded bg-elev px-[5px] font-mono text-[11px] tabular-nums text-muted">O</kbd></dd></div>
                    <div class="flex items-center justify-between gap-4 border-b border-line py-3"><dt class="text-xs text-faint">Save file</dt><dd class="m-0 flex items-center gap-1"><kbd class="inline-grid h-5 min-w-5 place-items-center rounded bg-elev px-[5px] font-mono text-[11px] tabular-nums text-muted">Ctrl</kbd><kbd class="inline-grid h-5 min-w-5 place-items-center rounded bg-elev px-[5px] font-mono text-[11px] tabular-nums text-muted">S</kbd></dd></div>
                    <div class="flex items-center justify-between gap-4 border-b border-line py-3"><dt class="text-xs text-faint">Toggle preview</dt><dd class="m-0 flex items-center gap-1"><kbd class="inline-grid h-5 min-w-5 place-items-center rounded bg-elev px-[5px] font-mono text-[11px] tabular-nums text-muted">Eye</kbd></dd></div>
                  </dl>
                {/if}
              </div>

              {#if !workspace}
                <div class="w-full max-[1279px]:hidden" aria-hidden="true">
                  <div class="rounded-xl border border-line bg-panel p-7">
                    <div class="flex items-baseline justify-between gap-4">
                      <span class="font-mono text-[11px] tabular-nums text-faint">docs/vision.md</span>
                      <span class="font-mono text-[11px] tabular-nums text-faint">482 words</span>
                    </div>
                    <span class="mt-5 mb-4 block h-0.5 w-8 bg-accent"></span>
                    <h2 class="m-0 mb-3 text-[22px] font-semibold tracking-[-0.01em] text-ink">Field notes</h2>
                    <p class="m-0 mb-5 text-[13px] leading-[1.65] text-muted">
                      Write in one pane. Read in the other. The outline follows the document, not
                      the other way around.
                    </p>
                    <h3 class="m-0 mb-2 text-sm font-semibold text-ink">Layout</h3>
                    <p class="m-0 mb-5 text-[13px] leading-[1.65] text-muted">A tree for the folder, an editor for the draft, a preview for the read.</p>
                    <div class="mt-6 flex flex-col gap-2 border-t border-line pt-4 text-xs text-faint">
                      <span class="flex items-center gap-2 text-ink before:size-1.5 before:rounded-full before:bg-accent">Field notes</span>
                      <span>Layout</span>
                      <span>Voice</span>
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      <footer class="flex min-w-0 items-center gap-3 border-t border-line bg-panel px-3 text-[11px] tabular-nums text-faint">
        <span class="max-w-[32%] truncate" title={workspace ?? "No folder open"}>{workspace ?? "No folder open"}</span>
        {#if notice}
          <span class="min-w-0 flex-[0_1_auto] truncate {notice.kind === 'ok' ? 'text-ok' : 'text-danger'}" aria-live="polite">
            {notice.text}
          </span>
        {/if}
        <span class="ml-auto flex shrink-0 items-center gap-3">
          {#if activeTab && stats}
            <span>{stats.words} {stats.words === 1 ? "word" : "words"} · {stats.minutes} min read</span>
          {/if}
          {#if activeTab}
            <span class="max-w-[300px] truncate font-mono text-muted max-[860px]:hidden" title={activeTab.path}>
              {activeTab.path}{#if dirty}<span class="ml-[7px] font-sans text-accent">Unsaved</span>{/if}
            </span>
          {/if}
          {#if workspace}
            <span>{skills.length} {skills.length === 1 ? "skill" : "skills"}{#if invalidSkills > 0} · {invalidSkills} {invalidSkills === 1 ? "needs a fix" : "need fixes"}{/if}</span>
          {/if}
        </span>
      </footer>
    </main>
    {/if}
  </div>
</div>
