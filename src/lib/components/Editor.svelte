<script lang="ts">
  import { onMount } from "svelte";
  import { basicSetup } from "codemirror";
  import { EditorView, keymap } from "@codemirror/view";
  import { Compartment, EditorState, Prec } from "@codemirror/state";
  import { markdown } from "@codemirror/lang-markdown";
  import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
  import { tags } from "@lezer/highlight";
  import { isMarkdownPath } from "$lib/files";

  let {
    path,
    value,
    revealLine = 0,
    revealKey = 0,
    wordWrap = true,
    showLineNumbers = true,
    onChange,
    onSave,
  }: {
    path: string;
    value: string;
    revealLine?: number;
    revealKey?: number;
    wordWrap?: boolean;
    showLineNumbers?: boolean;
    onChange: (value: string) => void;
    onSave: () => void;
  } = $props();

  let host: HTMLDivElement;
  let view: EditorView | null = null;
  const wrapCompartment = new Compartment();

  const theme = EditorView.theme(
    {
      "&": {
        height: "100%",
        backgroundColor: "var(--bg-editor)",
        color: "var(--text)",
        fontSize: "var(--editor-font-size, 13px)",
      },
      "&.cm-focused": { outline: "none" },
      ".cm-scroller": {
        fontFamily: "var(--font-mono)",
        lineHeight: "var(--editor-line-height, 1.65)",
        tabSize: "var(--editor-tab-size, 2)",
        overflow: "auto",
      },
      ".cm-content": {
        padding: "20px 0 30vh",
        caretColor: "var(--accent-strong)",
      },
      ".cm-line": { padding: "0 24px" },
      ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--accent-strong)" },
      ".cm-gutters": {
        backgroundColor: "var(--bg-editor)",
        color: "var(--text-faint)",
        border: "none",
        borderRight: "1px solid var(--line)",
        paddingLeft: "4px",
      },
      ".cm-activeLineGutter": {
        backgroundColor: "var(--bg-elev)",
        color: "var(--text-dim)",
      },
      ".cm-activeLine": { backgroundColor: "var(--active-line)" },
      "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection":
        {
          backgroundColor: "var(--selection)",
        },
      ".cm-selectionMatch": { backgroundColor: "var(--bg-active)" },
      ".cm-searchMatch": { backgroundColor: "var(--bg-active)", outline: "1px solid var(--line-strong)" },
      ".cm-searchMatch.cm-searchMatch-selected": { backgroundColor: "var(--bg-hover)" },
      ".cm-panels": {
        backgroundColor: "var(--bg-elev)",
        color: "var(--text)",
        borderTop: "1px solid var(--line)",
      },
      ".cm-panels.cm-panels-top": { borderBottom: "1px solid var(--line)" },
      ".cm-textfield": {
        backgroundColor: "var(--bg-editor)",
        color: "var(--text)",
        border: "1px solid var(--line-strong)",
        borderRadius: "4px",
        padding: "2px 6px",
      },
      ".cm-button": {
        backgroundColor: "var(--bg-hover)",
        backgroundImage: "none",
        color: "var(--text)",
        border: "1px solid var(--line-strong)",
        borderRadius: "4px",
        padding: "2px 8px",
      },
      ".cm-tooltip": {
        backgroundColor: "var(--bg-elev)",
        border: "1px solid var(--line-strong)",
        borderRadius: "5px",
        color: "var(--text)",
      },
    },
    { dark: true },
  );

  const highlight = HighlightStyle.define([
    { tag: tags.heading1, color: "var(--syntax-strong)", fontWeight: "700" },
    { tag: tags.heading2, color: "var(--syntax-strong)", fontWeight: "600" },
    { tag: tags.heading, color: "var(--syntax-dim)", fontWeight: "600" },
    { tag: tags.strong, color: "var(--syntax-strong)", fontWeight: "700" },
    { tag: tags.emphasis, color: "var(--syntax-dim)", fontStyle: "italic" },
    { tag: tags.link, color: "var(--syntax-link)", textDecoration: "underline" },
    { tag: tags.url, color: "var(--syntax-faint)" },
    { tag: tags.monospace, color: "var(--syntax-dim)" },
    { tag: tags.quote, color: "var(--syntax-faint)", fontStyle: "italic" },
    { tag: tags.list, color: "var(--syntax-dim)" },
    { tag: tags.processingInstruction, color: "var(--syntax-faint)" },
    { tag: tags.meta, color: "var(--syntax-faint)" },
    { tag: tags.comment, color: "var(--syntax-faint)" },
  ]);

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: value,
        extensions: [
          basicSetup,
          ...(isMarkdownPath(path) ? [markdown()] : []),
          theme,
          syntaxHighlighting(highlight),
          wrapCompartment.of(wordWrap ? EditorView.lineWrapping : []),
          Prec.highest(
            keymap.of([
              {
                key: "Mod-s",
                run: () => {
                  onSave();
                  return true;
                },
              },
            ]),
          ),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) onChange(update.state.doc.toString());
          }),
        ],
      }),
    });

    if (revealLine > 0) scrollToLine(revealLine);

    return () => {
      view?.destroy();
      view = null;
    };
  });

  function scrollToLine(lineNumber: number) {
    if (!view) return;
    const line = Math.min(Math.max(lineNumber, 1), view.state.doc.lines);
    const position = view.state.doc.line(line).from;
    view.dispatch({
      selection: { anchor: position },
      effects: EditorView.scrollIntoView(position, { y: "center" }),
    });
  }

  $effect(() => {
    const next = wordWrap;
    if (view) {
      view.dispatch({
        effects: wrapCompartment.reconfigure(next ? EditorView.lineWrapping : []),
      });
    }
  });

  $effect(() => {
    const next = value;
    if (view && next !== view.state.doc.toString()) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: next },
      });
    }
  });

  $effect(() => {
    void revealKey;
    if (revealLine > 0) scrollToLine(revealLine);
  });
</script>

<div class="h-full overflow-hidden rounded-xl bg-editor [&_.cm-editor]:h-full {showLineNumbers ? '' : '[&_.cm-gutters]:hidden'}" bind:this={host}></div>
