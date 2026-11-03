# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Tauri 2 (Rust core) + SvelteKit/Svelte 5 + TypeScript, static adapter, CodeMirror 6 editor. UI follows LavUIX conventions with Hugeicons and Impeccable's craft floor. Chosen by the user ("tauri based app").

## Users

*Inferred from the brief, not yet confirmed by the user.* Developers and agent-tooling users who maintain a personal or team library of Markdown documents and AI-agent skill folders (for example `skills/<name>/SKILL.md` with YAML frontmatter). They work locally on Linux/desktop, keep files in git, and want a focused tool that opens straight into that library instead of a general-purpose IDE.

## Product Purpose

A Markdown-first desktop workspace. Success means: open a folder, see every Markdown document immediately, edit with live preview and an outline, and catch malformed skill metadata in the same tree before it breaks an agent.

## Positioning

*Inferred.* Not a general code editor: the document tree, outline, search, and stats are shaped around Markdown projects. Skill auditing is built into the same tree rather than bolted on as a separate tool.

## Operating Context

*Inferred.* Local-only desktop app. Users keep content on disk, version it with git, and often keep skills in per-agent directories (`.opencode/skills`, `.claude/skills`, `.agents/skills`). No cloud account required for the MVP.

## Capabilities and Constraints

MVP confirmed by the user: open a folder workspace, Markdown document tree, edit/save with atomic writes, sanitized live preview, document outline, word and read-time stats, read-only library search, and a skill audit parsing `SKILL.md` frontmatter. Constraints: file access only through typed Tauri commands; frontend never touches raw filesystem paths outside them; single window; no sync, no plugin system, no terminal. Undecided: skill format beyond `SKILL.md` + YAML frontmatter, whether Tripon-specific skill conventions exist.

## Brand Commitments

Name: CODE.MD. Dark, solid-surface tool UI. No translucent surfaces, no gradient backgrounds, no em dashes in copy, no AI attribution in commits.

## Evidence on Hand

None. No customer claims, benchmarks, or third-party assets were supplied. Do not fabricate them.

## Product Principles

1. The library is the product: opening a folder should reveal Markdown and skills, not an empty canvas.
2. Malformed metadata is surfaced, never silently accepted.
3. Plain files on disk, byte-faithful saves; the app is a guest in the user's git repo.
4. Local-first and private by default.
