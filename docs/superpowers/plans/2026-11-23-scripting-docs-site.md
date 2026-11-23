# Scripting Docs Site Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** An mdBook site for the rhai scripting engine, published to GitHub Pages from `main`.

**Architecture:** `docs/` is the book source (`book.toml` + `SUMMARY.md`); the existing `docs/scripting/*.md` are
chapters as-is; three new pages; one workflow builds and deploys.

**Tech Stack:** mdBook (latest release binary), GitHub Actions (`actions/upload-pages-artifact`, `actions/deploy-pages`).

**Spec:** `docs/superpowers/specs/2026-11-23-scripting-docs-site-design.md`

## Global Constraints
- Existing reference pages are not copied or rewritten; only linked from `SUMMARY.md`.
- `docs/superpowers/**` must not appear on the site.
- Examples come from `Data/Scripts/{classes,content,examples,vendors,events}` — never `Data/Scripts/raids/` or
  `Data/Scripts/events/green_acres_battle.rhai` (private, uncommitted).
- Every rhai snippet must match the real API (checked against `crates/muo_content/src/scripting/*.rs` and
  `docs/scripting/*.md`).

## Review Focus
- Links between existing pages (`world.md`, `#classes` anchors in `README.md`) still resolve inside the book —
  check with `mdbook build` warnings and by clicking the sidebar in `mdbook serve`.
- The site works under the project path `https://andreakarasho.github.io/muo-rs/` (no absolute `/` links).
- `docs/superpowers/` files absent from `docs/book/` output.
- The workflow does not run (or fail) on pushes that don't touch `docs/**`.
- A snippet in the new pages that uses a function that doesn't exist — every snippet's functions grep-found in
  `crates/muo_content/src/scripting`.

---

### Task 1: Book scaffold and deploy workflow

**Files:**
- Create: `docs/book.toml`, `docs/SUMMARY.md`, `.github/workflows/docs.yml`
- Modify: `.gitignore` (add `docs/book/`)

- [ ] **Step 1:** `docs/book.toml`: `[book] title = "muo-rs scripting"`, `src = "."`; `[build] build-dir = "book"`;
  `[output.html] git-repository-url = "https://github.com/andreakarasho/muo-rs"`, `site-url = "/muo-rs/"`.
- [ ] **Step 2:** `docs/SUMMARY.md`: `[Introduction](scripting/README.md)`; part `# Writing scripts`: Getting started
  (`scripting/getting-started.md`), Examples (`scripting/examples.md`), then the five reference pages in README's
  order; part `# Engine internals`: Scripting engine (`engine/scripting.md`). Create the three new files as a
  one-line heading each for now.
- [ ] **Step 3:** `.github/workflows/docs.yml`: `on: push: branches: [main], paths: ['docs/**', '.github/workflows/docs.yml']`
  plus `workflow_dispatch`; permissions `contents: read, pages: write, id-token: write`; concurrency group `pages`;
  job build: checkout, download the mdBook release tarball (x86_64-unknown-linux-gnu) for the latest version,
  `mdbook build docs`, `actions/upload-pages-artifact` with `path: docs/book`; job deploy: `needs: build`,
  environment `github-pages`, `actions/deploy-pages`.
- [ ] **Step 4:** Install mdBook locally (`cargo install mdbook` or the release binary) and run `mdbook build docs`.
  Expected: exits 0, no "not found" link warnings; `docs/book/index.html` exists; `docs/book/superpowers` does not.

### Task 2: Getting started and examples pages

**Files:**
- Modify: `docs/scripting/getting-started.md`, `docs/scripting/examples.md`

- [ ] **Step 1:** Read `Data/Scripts/examples/`, one file each from `classes/`, `content/`, `vendors/`, `events/`
  (not green_acres), and `docs/scripting/README.md`.
- [ ] **Step 2:** `getting-started.md`: where scripts live and how the folders are used; `[ReloadScripts` and what a
  script error does; a first new item (`item(..)` with `extends`), a first creature tweak (`mobile(..)`), a first
  `/// @command`. Each snippet adapted from a real file, named.
- [ ] **Step 3:** `examples.md`: three worked examples — an item with `on_use`/`on_move_over`, a vendor (`shop(..)`),
  a command plus an event handler — each with a short walkthrough and links to the reference pages it uses.
- [ ] **Step 4:** Grep every function used in the snippets in `crates/muo_content/src/scripting`; `mdbook build docs`
  exits 0.

### Task 3: Engine internals page

**Files:**
- Modify: `docs/engine/scripting.md`

- [ ] **Step 1:** Read `crates/muo_content/src/scripting/mod.rs`, `friendly.rs`, `classes.rs`, one `api_*.rs`, and
  `crates/muo_server/src/script_hooks.rs` plus a caller (`mobile/part1/combat.rs` `on_move_over`).
- [ ] **Step 2:** Write: how scripts load (files, headers, reload), how API modules register functions on the
  Engine, how a class hook gets from `c.on_x(..)` to `ScriptHook` and the Rust call site; recipes "add a script
  function" and "add a hook" listing every file to touch (the `on_move_over` change in commit `07fd0bb` as the
  worked example); where tests go (`scripting/tests_*.rs`).
- [ ] **Step 3:** `mdbook build docs` exits 0; commit all three tasks after the user OKs (no commit without asking).
