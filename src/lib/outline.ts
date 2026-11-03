import { stripFrontmatter } from "./markdown";

export type OutlineHeading = {
  level: number;
  text: string;
  line: number;
};

function cleanHeading(text: string): string {
  return text
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/[*_`~]/g, "")
    .trim();
}

/**
 * ATX headings with absolute line numbers. Frontmatter, fenced code blocks and
 * headings indented enough to be code are skipped.
 */
export function extractOutline(source: string): OutlineHeading[] {
  const text = source.startsWith("\uFEFF") ? source.slice(1) : source;
  const lines = text.split("\n");
  const outline: OutlineHeading[] = [];

  let inFrontmatter = lines[0]?.trim() === "---";
  let fence: string | null = null;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const trimmed = line.trim();

    if (inFrontmatter) {
      if (index > 0 && (trimmed === "---" || trimmed === "...")) inFrontmatter = false;
      continue;
    }

    if (trimmed.startsWith("```") || trimmed.startsWith("~~~")) {
      const marker = trimmed.slice(0, 3);
      if (fence === null) fence = marker;
      else if (fence === marker) fence = null;
      continue;
    }
    if (fence !== null) continue;
    if (/^( {4,}|\t)/.test(line)) continue;

    const match = /^(#{1,6})\s+(.+?)\s*#*\s*$/.exec(line);
    if (!match) continue;

    const text = cleanHeading(match[2]);
    if (text !== "") {
      outline.push({ level: match[1].length, text, line: index + 1 });
    }
  }

  return outline;
}

export function documentStats(source: string): { words: number; minutes: number } {
  const body = stripFrontmatter(source);
  const words = body
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/[#>*_`~[\]()-]/g, " ")
    .trim()
    .split(/\s+/)
    .filter(Boolean).length;

  return { words, minutes: Math.max(1, Math.ceil(words / 220)) };
}
