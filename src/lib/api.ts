import { invoke } from "@tauri-apps/api/core";
import type { SearchHit, SkillInfo, TreeEntry } from "./types";

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
