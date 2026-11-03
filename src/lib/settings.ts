export type Theme = "dark" | "light" | "system";

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
