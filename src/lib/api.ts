import { invoke } from "@tauri-apps/api/core";
import type { GitCommit, GitStatus, SearchHit, SkillInfo, TreeEntry } from "./types";

/**
 * True only inside the Tauri webview. The UI can still render in a plain
 * browser, but every file operation needs the IPC bridge.
 */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    return Promise.reject(
      new Error("File access needs the desktop app. Run `npm run tauri dev` instead of opening the Vite URL."),
    );
  }
  return invoke<T>(command, args);
}

/** Opens the native folder picker. Returns null when the user cancels. */
export function pickWorkspace(): Promise<string | null> {
  return call<string | null>("pick_workspace");
}

export function openWorkspace(path: string): Promise<string> {
  return call<string>("open_workspace", { path });
}

export function closeWorkspace(): Promise<void> {
  return call<void>("close_workspace");
}

export function workspaceRoot(): Promise<string | null> {
  return call<string | null>("workspace_root");
}

export function scanTree(): Promise<TreeEntry[]> {
  return call<TreeEntry[]>("scan_tree");
}

export function scanSkills(): Promise<SkillInfo[]> {
  return call<SkillInfo[]>("scan_skills");
}

export function readFile(path: string): Promise<string> {
  return call<string>("read_file", { path });
}

export function saveFile(path: string, content: string): Promise<void> {
  return call<void>("save_file", { path, content });
}

export function searchWorkspace(query: string): Promise<SearchHit[]> {
  return call<SearchHit[]>("search_workspace", { query });
}

export function getGitStatus(): Promise<GitStatus | null> {
  return call<GitStatus | null>("git_status");
}

export function initGit(): Promise<GitStatus> {
  return call<GitStatus>("git_init");
}

export function getGitDiff(path: string, staged: boolean): Promise<string> {
  return call<string>("git_diff", { path, staged });
}

export function stageFiles(paths: string[]): Promise<void> {
  return call<void>("git_stage", { paths });
}

export function unstageFiles(paths: string[]): Promise<void> {
  return call<void>("git_unstage", { paths });
}

export function commitFiles(message: string): Promise<string> {
  return call<string>("git_commit", { message });
}

export function getGitLog(limit = 20): Promise<GitCommit[]> {
  return call<GitCommit[]>("git_log", { limit });
}
