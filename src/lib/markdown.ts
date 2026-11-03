import MarkdownIt from "markdown-it";
import DOMPurify from "dompurify";

const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: false,
});

/**
 * Mirrors the Rust frontmatter splitter: a leading `---` block is metadata,
 * not preview content.
 */
export function stripFrontmatter(input: string): string {
  const text = input.startsWith("\uFEFF") ? input.slice(1) : input;
  const lines = text.split("\n");
  if (lines[0]?.trim() !== "---") return text;

  for (let i = 1; i < lines.length; i += 1) {
    const line = lines[i].trim();
    if (line === "---" || line === "...") {
      return lines.slice(i + 1).join("\n");
    }
  }
  return text;
}

/** Raw HTML from markdown-it. Never inject this without sanitizing. */
export function renderRaw(source: string): string {
  return md.render(stripFrontmatter(source));
}

export function renderMarkdown(source: string): string {
  return DOMPurify.sanitize(renderRaw(source), {
    USE_PROFILES: { html: true },
    FORBID_TAGS: ["style", "iframe", "form", "input", "button", "object", "embed"],
    FORBID_ATTR: ["style", "srcdoc"],
  });
}
