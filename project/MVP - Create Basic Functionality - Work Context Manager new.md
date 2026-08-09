## Usage
`context-manager new <work name>`
`context-manager new` – Prompts the user with the name

## Requirements

- Should be configurable: configuration should live inside `~/.work_context_manager/config.toml` (this is not bound to `new` subcommand is more of a overall requirement that happens to be a prereq for implementing `new` )
	- The config should be toml
	- Configuration should allow:
		- Template config — a template folder with templates in markdown
		- Work context repo — a folder to store the markdown created with the `<work name>`
	- The when `context-manager new` is invoked it prompts list of templates from template folder in config is prompted so the user can choose among the available 
- The prompts should be pretty, like modern clis do
- Should use most common used libraries

## Done

- `Config` in lib: load/save TOML at `~/.work_context_manager/config.toml`
- Template discovery (`.md` files) from the configured template folder
- `new_work_context` renders the chosen template (`{{ name }}`) into `<work-context-repo>/<work-name>.md` (kebab-cased)
- CLI: `new [name]`, `init`, `show-config` using clap + dialoguer prompts
- Used common libraries: clap, dialoguer, serde, toml, thiserror, colored
