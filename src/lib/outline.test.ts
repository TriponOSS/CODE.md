import { describe, expect, it } from "vitest";
import { documentStats, extractOutline } from "./outline";

describe("extractOutline", () => {
  it("collects headings with absolute line numbers", () => {
    const doc = "---\nname: x\n---\n\n# Title\n\ntext\n\n## Part one\n";
    expect(extractOutline(doc)).toEqual([
      { level: 1, text: "Title", line: 5 },
      { level: 2, text: "Part one", line: 9 },
    ]);
  });

  it("skips fenced code blocks", () => {
    const doc = "# Real\n\n```md\n# Not a heading\n```\n\n## Also real\n";
    expect(extractOutline(doc).map((heading) => heading.text)).toEqual(["Real", "Also real"]);
  });

  it("skips indented code", () => {
    const doc = "# Real\n\n    # indented code\n";
    expect(extractOutline(doc).map((heading) => heading.text)).toEqual(["Real"]);
  });

  it("cleans inline markdown from heading text", () => {
    expect(extractOutline("## **Bold** `code` [link](https://x.dev)")[0]?.text).toBe(
      "Bold code link",
    );
  });

  it("returns nothing for plain prose", () => {
    expect(extractOutline("just words\nmore words")).toEqual([]);
  });
});

describe("documentStats", () => {
  it("counts words without frontmatter or fenced code", () => {
    const doc = "---\nname: x\n---\n\n# Title\n\none two three\n\n```\nignored code here\n```\n";
    const stats = documentStats(doc);
    expect(stats.words).toBe(4);
    expect(stats.minutes).toBe(1);
  });

  it("reports at least one minute for an empty document", () => {
    expect(documentStats("").minutes).toBe(1);
  });
});
