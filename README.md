# Work Context Manager

A personal project to help me with fine-grained task organization.

The goal of this app is to organize my daily workflow. Since I started working with AI agents, my workflow has become more dynamic — I tend to work on several fronts simultaneously. This app helps me keep track of the current progress of each activity.

## App Premises
- Cross-platform app
- Based on markdown files (LLM friendly)
- Assumes a local repository — personally, I will use [Obsidian](todo post obsidian link here) to manage the repository

## Project Premises
- This app is managed inside this repository
- Uses a Markdown Kanban board
- Project details are contained in the [project folder](./project)

## Repository Structure
- Cargo workspace with two crates:
  - `lib` — core logic, reusable by the CLI and a future Tauri app
  - `cli` — thin command line interface on top of the lib

## Development
Build, test, lint and format commands are wrapped in the `Makefile` (see `make help`, or just `make` for a build). Cargo is available under `~/.cargo` — add it to your `PATH` if rustup was installed with `--no-modify-path`.

## Working on this project
- Describe a task inside: [project folder](./project)
- Create a feature branch that should follow the naming convention: `your_github_user_name.card_name.description_of_the_work_being_done`
- Work on the feature branch
- Create a PR
