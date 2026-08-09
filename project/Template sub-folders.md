# Template sub-folders

## Problem

Templates could only live directly inside the template folder (`~/context_manager/contexts/@templates`). Subdirectories were ignored by `list_templates`, there was no way to create them from the CLI, and template picking used a flat dialoguer `Select` that could not navigate folders.

## Solution

- `lib/src/template.rs`
  - `list_templates` now recurses into subdirectories (skipping hidden entries); template names are relative paths like `daily/standup.md`, sorted
  - New `create_template_folder(config, name)` + `sanitize_template_folder_name` (kebab-case), with new errors `EmptyTemplateFolderName` / `InvalidTemplateFolderName`
- `lib/src/tree.rs`
  - New `build_tree_from(root)` so any folder (e.g. the template folder) can be browsed with the same tree logic
- `cli/src/main.rs`
  - `context-manager new template-folder <name>` (also offered in the `new` kind prompt)
  - `run_tree_browser` now takes a file-activation callback and returns the selected path; `tree`/`open` open the editor and keep browsing, template picking selects and quits
  - `new context` template picking now uses the interactive tree browser starting at the template folder (navigate into `daily-check-ins/`, pick `standup.md`)

## Acceptance

- [x] `list_templates` discovers templates in sub-folders with relative names
- [x] `new template-folder` creates a sanitized folder and is offered in the `new` prompt
- [x] `new context` template picking browses template folders with the tree UI
- [x] Rendered output uses the sub-folder template (verified `# subfolder-test — daily check-in`)
- [x] clippy `-D warnings` and fmt clean; 27 unit + 23 doctests pass
