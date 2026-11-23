# Scripting docs site — design

## Intent
A public documentation website on GitHub Pages for the rhai scripting engine, for two readers: server owners who
write content in rhai without touching Rust, and contributors working on the engine.

## Goals
- Site built from the markdown already in `docs/scripting/`, published on every push to `main`.
- Two parts: **Writing scripts** (getting started, worked examples, the existing API reference) and
  **Engine internals** (how a script hook / function reaches Rust, how to add one).
- Sidebar, search, dark mode.

## Non-goals
- rhai syntax highlighting (code blocks render plain; add a highlight.js grammar later if wanted).
- Docs for anything other than scripting; versioned docs; custom domain.

## Anti-goals
- A second copy of the reference that drifts from `docs/scripting/*.md`: the site renders those files, it does not
  duplicate them.

## Approaches
- **mdBook (picked):** Rust's own docs tool, one binary in CI, sidebar/search built in; existing files unchanged plus a
  `SUMMARY.md`. No rhai highlighting.
- MkDocs Material: nicer look, but Python in a Rust repo's CI.
- Pages serving `docs/` raw: no build, but no sidebar or search.

## Design
- `docs/book.toml`, `docs/SUMMARY.md` (book source = `docs/`; `docs/superpowers/` is not in the summary, so not on
  the site).
- New pages: `docs/scripting/getting-started.md` (where scripts live, `[ReloadScripts`, a first item, a first
  creature, a first command), `docs/scripting/examples.md` (a custom item with a hook, a vendor, a command + event),
  `docs/engine/scripting.md` (how `muo_content/src/scripting` registers functions, how `script_hooks.rs` hooks are
  called, adding a function or a hook).
- `.github/workflows/docs.yml`: on push to `main` touching `docs/**`, install the mdBook binary, `mdbook build docs`,
  deploy with `actions/deploy-pages`.
- One manual step for the owner: repo Settings → Pages → Source: "GitHub Actions".

## Left to the builder
Wording and example choice in the new pages, drawn from real classes/scripts in `Data/Scripts` (not the custom raids
scripts).
