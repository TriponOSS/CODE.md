import { describe, expect, it } from "vitest";
import { canFormatDocument, formatDocument } from "./format";

describe("formatDocument", () => {
  it("formats supported JavaScript documents", async () => {
    expect(await formatDocument("src/app.js", "const answer={value:42}"))
      .toBe("const answer = { value: 42 };\n");
  });

  it("formats Markdown documents", async () => {
    expect(await formatDocument("README.md", "# Title   \n\nText   "))
      .toBe("# Title\n\nText\n");
  });

  it("leaves unsupported text files unavailable", async () => {
    expect(canFormatDocument("notes.txt")).toBe(false);
    await expect(formatDocument("notes.txt", "keep as-is")).rejects.toThrow(
      "No formatter is available",
    );
  });
});