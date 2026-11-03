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

export type SidebarView = "files" | "outline" | "skills" | "search" | "settings";
