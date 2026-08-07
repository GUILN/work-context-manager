- We are creating the cask of the project
- Should create a Makefile
	- All important: build / test commands should be described in the make file
- This project will be in rust
	- Will have the lib and cli separated
	- In future we will have a tauri app so the coding in this initial phase that will only use cli also needs to be extensible / reusable for a future Tauri app.
	- This cask just need to create the lib and a functional cli
- Update `README.md`


### Tests
- write tests

## Done
- Cargo workspace with `lib/` and `cli/` crates
- Makefile with build/run/test/lint/fmt/clean targets
- CLI prints `work-context-manager v0.1.0`
- Lib exposes `App` with name/version, unit tests passing
- README updated