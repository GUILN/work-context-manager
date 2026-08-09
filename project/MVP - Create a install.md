Create a installer script
- Create a minimal installer script in rust

## Done

- `installer/` crate: builds the workspace in release and copies `context-manager` into `~/.local/bin` (override with `WCM_INSTALL_DIR`)
- `make install` target wiring the installer
- CLI binary renamed to `context-manager`