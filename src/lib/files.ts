/** Markdown files get a rendered preview; other text files open as plain text. */
export function isMarkdownPath(path: string): boolean {
  return /\.(md|mdx|markdown)$/i.test(path);
}

/** True for the agent skill convention: a file named SKILL.md. */
export function isSkillPath(path: string): boolean {
  const name = path.split("/").pop()?.toLowerCase() ?? "";
  return name === "skill.md";
}
