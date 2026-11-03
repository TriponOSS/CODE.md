import type { TreeEntry } from "./types";

export type TreeNode = TreeEntry & { children: TreeNode[] };
export type TreeRow = TreeNode & { depth: number };

function parentPath(path: string): string {
  const index = path.lastIndexOf("/");
  return index === -1 ? "" : path.slice(0, index);
}

export function buildTree(entries: TreeEntry[]): TreeNode[] {
  const nodes = new Map<string, TreeNode>();
  const roots: TreeNode[] = [];

  for (const entry of [...entries].sort((a, b) => a.path.localeCompare(b.path))) {
    nodes.set(entry.path, { ...entry, children: [] });
  }

  for (const entry of entries) {
    const node = nodes.get(entry.path);
    if (!node) continue;
    const parent = nodes.get(parentPath(entry.path));
    if (parent) {
      parent.children.push(node);
    } else {
      roots.push(node);
    }
  }

  const sortNodes = (list: TreeNode[]) => {
    list.sort((a, b) => {
      if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
      return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
    });
    for (const node of list) sortNodes(node.children);
  };
  sortNodes(roots);

  return roots;
}

export function flattenTree(
  nodes: TreeNode[],
  collapsed: Set<string>,
  depth = 0,
): TreeRow[] {
  const rows: TreeRow[] = [];
  for (const node of nodes) {
    rows.push({ ...node, depth });
    if (node.isDir && !collapsed.has(node.path)) {
      rows.push(...flattenTree(node.children, collapsed, depth + 1));
    }
  }
  return rows;
}

/** All ancestor directory paths of a file, used to auto-expand a file's parent. */
export function ancestorsOf(path: string): string[] {
  const parts = path.split("/");
  const out: string[] = [];
  for (let i = 1; i < parts.length; i += 1) {
    out.push(parts.slice(0, i).join("/"));
  }
  return out;
}
