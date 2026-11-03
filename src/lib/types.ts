export type TreeEntry = {
  path: string;
  name: string;
  isDir: boolean;
};

export type SkillInfo = {
  path: string;
  name: string;
  description: string;
  triggers: string[];
  title: string | null;
  valid: boolean;
  issues: string[];
};

export type SearchHit = {
  path: string;
  line: number;
  text: string;
};

export type GitFile = {
  path: string;
  index: string;
  worktree: string;
  staged: boolean;
  unstaged: boolean;
  untracked: boolean;
  originalPath: string | null;
};

export type GitStatus = {
  repoRoot: string;
  branch: string;
  detached: boolean;
  ahead: number;
  behind: number;
  hasRemote: boolean;
  clean: boolean;
  files: GitFile[];
};

export type GitCommit = {
  hash: string;
  author: string;
  date: string;
  subject: string;
};

export type SidebarView = "files" | "outline" | "skills" | "search" | "git" | "settings";
