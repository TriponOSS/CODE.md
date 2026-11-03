export type Theme = "dark" | "light" | "system";
export type BuiltInColorTheme = "t3-code" | "t3-chat" | "grove" | "ocean" | "ember" | "iris";
export type ColorTheme = BuiltInColorTheme | `custom:${string}`;

export type ThemeDefinition = {
  version: 1;
  id: string;
  name: string;
  appearance: "light" | "dark";
  author?: string;
  description?: string;
  colors: Record<string, string>;
  variants?: { dark?: Record<string, string> };
};

export function parseThemeDefinition(value: unknown): ThemeDefinition {
  if (!value || typeof value !== "object") throw new Error("Theme file must contain a JSON object.");
  const theme = value as Partial<ThemeDefinition>;
  const validColors = (colors: unknown): colors is Record<string, string> =>
    !!colors && typeof colors === "object" && !Array.isArray(colors) &&
    Object.entries(colors).every(([key, color]) => /^[a-zA-Z][a-zA-Z0-9]*$/.test(key) && typeof color === "string" && /^#[\da-fA-F]{6}$/.test(color));

  if (theme.version !== 1) throw new Error("Unsupported theme version. Expected version 1.");
  if (typeof theme.id !== "string" || !/^[a-zA-Z0-9][a-zA-Z0-9-]*$/.test(theme.id)) {
    throw new Error("Theme id must contain only letters, numbers, and hyphens.");
  }
  if (typeof theme.name !== "string" || theme.name.trim() === "") throw new Error("Theme name is required.");
  if (theme.appearance !== "light" && theme.appearance !== "dark") throw new Error("Theme appearance must be light or dark.");
  if (!validColors(theme.colors) || !theme.colors.canvas || !theme.colors.text || !theme.colors.accent) {
    throw new Error("Theme colors must include canvas, text, and accent hex values.");
  }
  if (theme.variants?.dark !== undefined && !validColors(theme.variants.dark)) {
    throw new Error("Dark variant colors must be six-digit hex values.");
  }
  return theme as ThemeDefinition;
}

const CUSTOM_THEMES_KEY = "codemd:custom-themes";

export function loadCustomThemes(): ThemeDefinition[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const stored: unknown = JSON.parse(localStorage.getItem(CUSTOM_THEMES_KEY) ?? "[]");
    return Array.isArray(stored) ? stored.map(parseThemeDefinition) : [];
  } catch {
    return [];
  }
}

export function saveCustomThemes(themes: ThemeDefinition[]): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(CUSTOM_THEMES_KEY, JSON.stringify(themes));
}

export type AppSettings = {
  general: {
    confirmDiscard: boolean;
  };
  appearance: {
    reduceMotion: boolean;
  };
  editor: {
    fontSize: number;
    lineHeight: number;
    tabSize: number;
    wordWrap: boolean;
    lineNumbers: boolean;
  };
  preview: {
    fontSize: number;
  };
  git: {
    autoRefresh: boolean;
    showInStatusBar: boolean;
  };
};

export const DEFAULT_SETTINGS: AppSettings = {
  general: {
    confirmDiscard: true,
  },
  appearance: {
    reduceMotion: false,
  },
  editor: {
    fontSize: 13,
    lineHeight: 1.65,
    tabSize: 2,
    wordWrap: true,
    lineNumbers: true,
  },
  preview: {
    fontSize: 14,
  },
  git: {
    autoRefresh: true,
    showInStatusBar: true,
  },
};

const SETTINGS_KEY = "codemd:settings";

function bool(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}

function numberIn(value: unknown, min: number, max: number, fallback: number): number {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(max, Math.max(min, parsed));
}

function cloneDefaults(): AppSettings {
  return structuredClone(DEFAULT_SETTINGS);
}

export function loadSettings(): AppSettings {
  if (typeof localStorage === "undefined") return cloneDefaults();
  try {
    const parsed = JSON.parse(localStorage.getItem(SETTINGS_KEY) ?? "null") as
      | Partial<AppSettings>
      | null;
    if (!parsed) return cloneDefaults();
    const defaults = DEFAULT_SETTINGS;
    const general = parsed.general ?? defaults.general;
    const appearance = parsed.appearance ?? defaults.appearance;
    const editor = parsed.editor ?? defaults.editor;
    const preview = parsed.preview ?? defaults.preview;
    const git = parsed.git ?? defaults.git;
    return {
      general: {
        confirmDiscard: bool(general.confirmDiscard, defaults.general.confirmDiscard),
      },
      appearance: {
        reduceMotion: bool(appearance.reduceMotion, defaults.appearance.reduceMotion),
      },
      editor: {
        fontSize: Math.round(numberIn(editor.fontSize, 11, 22, defaults.editor.fontSize)),
        lineHeight: numberIn(editor.lineHeight, 1.3, 2.2, defaults.editor.lineHeight),
        tabSize: [2, 4, 8].includes(Number(editor.tabSize)) ? Number(editor.tabSize) : defaults.editor.tabSize,
        wordWrap: bool(editor.wordWrap, defaults.editor.wordWrap),
        lineNumbers: bool(editor.lineNumbers, defaults.editor.lineNumbers),
      },
      preview: {
        fontSize: Math.round(numberIn(preview.fontSize, 12, 22, defaults.preview.fontSize)),
      },
      git: {
        autoRefresh: bool(git.autoRefresh, defaults.git.autoRefresh),
        showInStatusBar: bool(git.showInStatusBar, defaults.git.showInStatusBar),
      },
    };
  } catch {
    return cloneDefaults();
  }
}

export function saveSettings(settings: AppSettings): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
}
