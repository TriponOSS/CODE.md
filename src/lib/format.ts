const parserByExtension: Record<string, string> = {
  md: "markdown",
  mdx: "mdx",
  markdown: "markdown",
  js: "babel",
  jsx: "babel",
  mjs: "babel",
  cjs: "babel",
  ts: "typescript",
  tsx: "typescript",
  mts: "typescript",
  cts: "typescript",
  json: "json",
  jsonc: "jsonc",
  json5: "json5",
  html: "html",
  htm: "html",
  vue: "vue",
  css: "css",
  scss: "scss",
  less: "less",
  yaml: "yaml",
  yml: "yaml",
  graphql: "graphql",
  gql: "graphql",
};

function extensionOf(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? path;
  const dot = name.lastIndexOf(".");
  return dot < 0 ? "" : name.slice(dot + 1).toLowerCase();
}

async function pluginsFor(parser: string) {
  switch (parser) {
    case "babel":
    case "json":
    case "jsonc":
    case "json5":
      return [
        await import("prettier/plugins/babel"),
        await import("prettier/plugins/estree"),
      ];
    case "typescript":
      return [
        await import("prettier/plugins/typescript"),
        await import("prettier/plugins/estree"),
      ];
    case "markdown":
    case "mdx":
      return [await import("prettier/plugins/markdown")];
    case "html":
    case "vue":
      return [await import("prettier/plugins/html")];
    case "css":
    case "scss":
    case "less":
      return [await import("prettier/plugins/postcss")];
    case "yaml":
      return [await import("prettier/plugins/yaml")];
    case "graphql":
      return [await import("prettier/plugins/graphql")];
    default:
      throw new Error(`No formatter is available for parser ${parser}.`);
  }
}

export function canFormatDocument(path: string): boolean {
  return extensionOf(path) in parserByExtension;
}

export async function formatDocument(path: string, source: string): Promise<string> {
  const parser = parserByExtension[extensionOf(path)];
  if (!parser) throw new Error(`No formatter is available for ${path}.`);
  const [prettier, plugins] = await Promise.all([
    import("prettier/standalone"),
    pluginsFor(parser),
  ]);
  return prettier.format(source, { parser, plugins });
}