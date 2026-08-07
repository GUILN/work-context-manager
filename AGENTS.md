# AGENTS.md

## Project Overview

Work Context Manager is a personal task-organization tool. It helps the author keep track of progress across multiple simultaneous work fronts. It is built around markdown files (LLM friendly) and is managed as an Obsidian vault.

## Repository Layout

- `project/` — Markdown Kanban board (Obsidian Kanban plugin format) tracking the work itself. This is the source of truth for planned work.
  - `MVP.md` — the MVP board (TODO/DOING/DONE lists).
  - `MVP - Create Project Cask.md` — the current task: scaffolding the project.
- `.obsidian/` — Obsidian vault configuration (including the Kanban plugin). Do not modify unless working on the vault setup.

## Tech Direction

The app is Rust:

- Scaffolded as a **lib** and a **cli** separated into distinct targets/crates.
- The core logic lives in the lib so it can be reused by a future **Tauri** app; the code written for the CLI phase must remain extensible/reusable for that GUI app (no logic embedded in the CLI).
- The current tracked task is "Create Project Cask" — the initial scaffolding (crate layout, toolchain, workspace).

## Workflow Conventions

- Work is tracked as markdown Kanban files under `project/` using the Obsidian Kanban plugin format. Add/update tasks there before or alongside code changes.
- Keep this repo self-contained: project management stays inside this repository.
- Work on feature branches named `your_github_user_name.card_name.description_of_the_work_being_done`, then open a PR.
- Commits should be small and focused on a single item from the board where possible.

## Commands

No build/tooling commands are set up yet — the Rust project has not been scaffolded. Once the workspace exists, note here the canonical `cargo build` / `cargo test` / lint commands and update this file.