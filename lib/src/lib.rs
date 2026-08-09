//! # Work Context Manager
//!
//! Core logic for the Work Context Manager app: configurable work-context
//! tracking based on markdown files. This library is shared by the CLI
//! and (in the future) a Tauri desktop app.
//!
//! The library is organised around three pieces:
//!
//! - [`Config`]: TOML configuration living at `~/.context_manager/config.toml`
//! - [`Template`]: markdown templates stored in the configured template folder
//! - [`work_context`]: creating new work contexts from those templates
//!
//! # Example
//!
//! ```
//! use context_manager::{Config, Template, work_context};
//!
//! let dir = std::env::temp_dir().join("wcm-doc-crate");
//! let template_path = dir.join("templates");
//! std::fs::create_dir_all(&template_path).unwrap();
//! std::fs::write(template_path.join("general.md"), "# {{ name }}\n").unwrap();
//!
//! let config = Config {
//!     template_folder: template_path.clone(),
//!     work_context_repo: dir.join("repo"),
//!     editor: None,
//! };
//! let template = Template {
//!     name: "general.md".into(),
//!     path: template_path.join("general.md"),
//! };
//!
//! let path = work_context::new_work_context(&config, "My Work", &template).unwrap();
//! assert!(path.ends_with("my-work.md"));
//!
//! std::fs::remove_dir_all(&dir).ok();
//! ```

pub mod app;
pub mod config;
pub mod editor;
pub mod error;
pub mod template;
pub mod work_context;

pub use app::App;
pub use config::Config;
pub use editor::open_with;
pub use error::{Error, Result};
pub use template::Template;
