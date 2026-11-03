import { describe, expect, it } from "vitest";
import { parseThemeDefinition } from "./settings";

const theme = {
  version: 1,
  id: "custom-theme",
  name: "Custom",
  appearance: "light",
  author: "Example",
  description: "Test theme",
  colors: {
    canvas: "#ccc8b1",
    text: "#211f1b",
    accent: "#211f1b",
  },
  variants: {
    dark: {
      canvas: "#2f2d26",
      text: "#d1cdb7",
      accent: "#d1cdb7",
    },
  },
};

describe("parseThemeDefinition", () => {
  it("accepts a version-1 theme with a dark variant", () => {
    expect(parseThemeDefinition(theme)).toEqual(theme);
  });

  it("rejects unsupported versions and malformed colors", () => {
    expect(() => parseThemeDefinition({ ...theme, version: 2 })).toThrow("Unsupported theme version");
    expect(() => parseThemeDefinition({ ...theme, colors: { ...theme.colors, accent: "url(#bad)" } })).toThrow("hex values");
  });
});