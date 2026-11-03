export type PaletteCommand = {
  id: string;
  label: string;
  hint?: string;
  keywords?: string;
  disabled?: boolean;
  run: () => void;
};

export type PaletteFile = {
  path: string;
  name: string;
};
