# `context-manager open` — same prompt/visual pattern as `tree`

## Problem

`context-manager open` used two sequential dialoguer `Select` prompts (pick project, pick context), while `tree` uses a full-screen interactive browser with breadcrumbs, arrow keys, colors, and help text. The two commands had inconsistent UX.

## Solution

Extracted the interactive browser loop from `cmd_tree` into a shared `run_tree_browser` function in `cli/src/main.rs`. Now:

- `context-manager tree` — starts the browser at the work context repo root
- `context-manager open [project]` — starts the browser inside the given project (when provided), otherwise at the root, exactly like `tree`
- Opening a context with `↵`/`→` still prints `➜ opening with <editor> ...` and launches the configured editor

Also removed the now-unused `pick_context` helper (dialoguer `Select`), keeping `pick_project` for `new context`/`new work_context`.

## Acceptance

- [x] `open` without arguments shows the interactive tree from the repo root
- [x] `open <project>` starts the browser already inside that project
- [x] `open <missing-project>` errors with `project `X` not found`
- [x] `↵` on a context opens it with the configured editor
- [x] `pick_context` removed; clippy `-D warnings` and fmt clean
