import { describe, expect, it } from "vitest";
import { renderRaw, stripFrontmatter } from "./markdown";

const doc = "---\nname: demo\ndescription: A demo\n---\n\n# Heading\n\nBody text.";

describe("stripFrontmatter", () => {
  it("removes a leading yaml block", () => {
    expect(stripFrontmatter(doc).trim()).toBe("# Heading\n\nBody text.");
  });

  it("leaves documents without frontmatter untouched", () => {
    const plain = "# Heading\n\nBody.";
    expect(stripFrontmatter(plain)).toBe(plain);
  });

  it("handles CRLF line endings", () => {
    const crlf = "---\r\nname: demo\r\n---\r\n\r\n# Heading";
    expect(stripFrontmatter(crlf).trim()).toBe("# Heading");
  });

  it("handles a BOM before the delimiter", () => {
    expect(stripFrontmatter(`\uFEFF${doc}`).trim()).toBe("# Heading\n\nBody text.");
  });

  it("does not strip an unterminated block", () => {
    const unterminated = "---\nname: demo\n# Heading";
    expect(stripFrontmatter(unterminated)).toBe(unterminated);
  });
});

describe("renderRaw", () => {
  it("renders markdown headings and emphasis", () => {
    const html = renderRaw("# Title\n\n**bold** and *italic*");
    expect(html).toContain("<h1>Title</h1>");
    expect(html).toContain("<strong>bold</strong>");
    expect(html).toContain("<em>italic</em>");
  });

  it("does not render frontmatter as content", () => {
    const html = renderRaw(doc);
    expect(html).not.toContain("description: A demo");
    expect(html).toContain("<h1>Heading</h1>");
  });

  it("renders GFM tables", () => {
    const html = renderRaw("| a | b |\n| - | - |\n| 1 | 2 |");
    expect(html).toContain("<table>");
    expect(html).toContain("<td>1</td>");
  });
});
