import { describe, expect, it } from "vitest";
import { ancestorsOf, buildTree, flattenTree } from "./tree";
import type { TreeEntry } from "./types";

const entries: TreeEntry[] = [
  { path: "skills", name: "skills", isDir: true },
  { path: "skills/code-review", name: "code-review", isDir: true },
  { path: "skills/code-review/SKILL.md", name: "SKILL.md", isDir: false },
  { path: "README.md", name: "README.md", isDir: false },
  { path: "docs", name: "docs", isDir: true },
  { path: "docs/guide.md", name: "guide.md", isDir: false },
];

describe("buildTree", () => {
  it("nests children under their parents", () => {
    const tree = buildTree(entries);
    const skills = tree.find((node) => node.path === "skills");
    expect(skills?.children[0]?.path).toBe("skills/code-review");
    expect(skills?.children[0]?.children[0]?.path).toBe("skills/code-review/SKILL.md");
  });

  it("sorts directories before files at each level", () => {
    const names = buildTree(entries).map((node) => node.name);
    expect(names).toEqual(["docs", "skills", "README.md"]);
  });

  it("survives entries whose parent is missing", () => {
    const tree = buildTree([{ path: "a/b/c.md", name: "c.md", isDir: false }]);
    expect(tree.map((node) => node.path)).toEqual(["a/b/c.md"]);
  });
});

describe("flattenTree", () => {
  it("hides descendants of collapsed directories", () => {
    const tree = buildTree(entries);
    const open = flattenTree(tree, new Set());
    const closed = flattenTree(tree, new Set(["skills"]));

    expect(open.some((row) => row.path === "skills/code-review/SKILL.md")).toBe(true);
    expect(closed.some((row) => row.path.startsWith("skills/") && row.path !== "skills")).toBe(
      false,
    );
  });

  it("tracks depth for indentation", () => {
    const rows = flattenTree(buildTree(entries), new Set());
    expect(rows.find((row) => row.path === "README.md")?.depth).toBe(0);
    expect(rows.find((row) => row.path === "skills/code-review/SKILL.md")?.depth).toBe(2);
  });
});

describe("ancestorsOf", () => {
  it("returns every parent directory", () => {
    expect(ancestorsOf("a/b/c.md")).toEqual(["a", "a/b"]);
    expect(ancestorsOf("c.md")).toEqual([]);
  });
});
