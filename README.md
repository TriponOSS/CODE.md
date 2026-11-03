# CODE.MD

A local-first desktop workspace for Markdown, text, and code. Tauri 2 + SvelteKit, built around browsing, editing, and auditing a folder of documents and agent skills.

## What it does

- Opens a folder and lists Markdown, text, and code files, git-aware, skipping build junk
- CodeMirror 6 editor with live, sanitized preview
- Document outline with jump-to-heading, plus word count and read time
- Skill audit for `SKILL.md` YAML frontmatter
- Full-text search across the open folder
- Atomic saves, so a crash never leaves a half-written file

## Stack

Tauri 2, Rust (`ignore`, `notify`, `serde_yaml`), Svelte 5 + SvelteKit, CodeMirror 6, markdown-it + DOMPurify, Hugeicons.

UI conventions follow LavUIX and Impeccable: tight spacing, solid surfaces, one icon family, no decorative gradients or glass.

Design direction: LavUIX by lavenv0 (https://vspcoderz.tech).

## Development

```bash
npm install
npm run tauri dev
```

Checks:

```bash
npm test
npm run check
cargo check --manifest-path src-tauri/Cargo.toml
```

`PLAN.md` is local scratch and is gitignored.
