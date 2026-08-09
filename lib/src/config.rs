use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub const CONFIG_DIR_NAME: &str = ".work_context_manager";
pub const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Folder containing markdown templates.
    pub template_folder: PathBuf,
    /// Folder where created work contexts are stored.
    pub work_context_repo: PathBuf,
    /// Command used to open work context files. When `None`, the `VISUAL`
    /// and `EDITOR` environment variables are used, falling back to `nvim`.
    #[serde(default)]
    pub editor: Option<String>,
}

impl Config {
    /// Resolves the editor command for a work context file.
    ///
    /// Preference order: `editor` from the config, then `VISUAL`, then
    /// `EDITOR`, then `nvim`.
    ///
    /// # Example
    ///
    /// ```
    /// use work_context_manager::Config;
    ///
    /// let cfg = Config::default_config().unwrap();
    /// let editor = cfg.resolve_editor();
    /// assert!(!editor.is_empty());
    /// ```
    pub fn resolve_editor(&self) -> String {
        self.editor
            .clone()
            .filter(|e| !e.trim().is_empty())
            .or_else(|| {
                std::env::var("VISUAL")
                    .ok()
                    .filter(|e| !e.trim().is_empty())
            })
            .or_else(|| {
                std::env::var("EDITOR")
                    .ok()
                    .filter(|e| !e.trim().is_empty())
            })
            .unwrap_or_else(|| "nvim".to_string())
    }

    /// Returns `~/.work_context_manager`.
    ///
    /// # Example
    ///
    /// ```
    /// use work_context_manager::config::CONFIG_DIR_NAME;
    /// use work_context_manager::Config;
    ///
    /// let dir = Config::config_dir().expect("home dir available");
    /// assert!(dir.ends_with(CONFIG_DIR_NAME));
    /// ```
    pub fn config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or(Error::NoHomeDir)?;
        Ok(home.join(CONFIG_DIR_NAME))
    }

    /// Returns `~/.work_context_manager/config.toml`.
    ///
    /// # Example
    ///
    /// ```
    /// use work_context_manager::config::CONFIG_FILE_NAME;
    /// use work_context_manager::Config;
    ///
    /// let path = Config::config_path().expect("home dir available");
    /// assert!(path.ends_with(CONFIG_FILE_NAME));
    /// ```
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(CONFIG_FILE_NAME))
    }

    /// Creates a default config based on the home directory.
    ///
    /// The `editor` field is left unset so the environment variables
    /// (`VISUAL`/`EDITOR`) or the `nvim` fallback apply.
    ///
    /// # Example
    ///
    /// ```
    /// use work_context_manager::Config;
    ///
    /// let cfg = Config::default_config().expect("home dir available");
    /// assert!(cfg.template_folder.ends_with("templates"));
    /// assert!(cfg.work_context_repo.ends_with("work_contexts"));
    /// ```
    pub fn default_config() -> Result<Self> {
        let home = dirs::home_dir().ok_or(Error::NoHomeDir)?;
        Ok(Self {
            template_folder: home.join(CONFIG_DIR_NAME).join("templates"),
            work_context_repo: home.join("work_contexts"),
            editor: None,
        })
    }

    /// Loads the config from `~/.work_context_manager/config.toml`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use work_context_manager::Config;
    ///
    /// let cfg = Config::load().expect("config file must exist");
    /// ```
    pub fn load() -> Result<Self> {
        Self::load_from(&Self::config_path()?)
    }

    /// Loads the config from an explicit path.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::Path;
    /// use work_context_manager::Config;
    ///
    /// let dir = std::env::temp_dir().join("wcm-doc-load_from");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let path = dir.join("config.toml");
    ///
    /// let cfg = Config {
    ///     template_folder: Path::new("/tmp/templates").to_path_buf(),
    ///     work_context_repo: Path::new("/tmp/work").to_path_buf(),
    ///     editor: Some("nvim".to_string()),
    /// };
    /// cfg.save_to(&path).unwrap();
    ///
    /// let loaded = Config::load_from(&path).unwrap();
    /// assert_eq!(loaded, cfg);
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// ```
    pub fn load_from(path: &std::path::Path) -> Result<Self> {
        let raw =
            std::fs::read_to_string(path).map_err(|_| Error::ConfigNotFound(path.to_path_buf()))?;
        Ok(toml::from_str(&raw)?)
    }

    /// Writes the config to its default location, creating parent dirs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use work_context_manager::Config;
    ///
    /// let cfg = Config::default_config().unwrap();
    /// cfg.save().expect("config should be saved");
    /// ```
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        self.save_to(&path)
    }

    /// Writes the config to an explicit path, creating parent dirs.
    ///
    /// # Example
    ///
    /// ```
    /// use work_context_manager::Config;
    ///
    /// let dir = std::env::temp_dir().join("wcm-doc-save_to");
    /// let path = dir.join("config.toml");
    ///
    /// let cfg = Config::default_config().unwrap();
    /// cfg.save_to(&path).unwrap();
    /// assert!(path.exists());
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// ```
    pub fn save_to(&self, path: &std::path::Path) -> Result<()> {
        let raw = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, raw)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default_config().unwrap_or_else(|_| Self {
            template_folder: PathBuf::new(),
            work_context_repo: PathBuf::new(),
            editor: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn config_path_is_under_home() {
        let dir = Config::config_dir().unwrap();
        let path = Config::config_path().unwrap();
        assert!(path.starts_with(&dir));
        assert!(path.ends_with(CONFIG_FILE_NAME));
    }

    #[test]
    fn roundtrips_through_toml() {
        let dir = std::env::temp_dir().join("wcm-config-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");

        let cfg = Config {
            template_folder: Path::new("/tmp/templates").to_path_buf(),
            work_context_repo: Path::new("/tmp/work").to_path_buf(),
            editor: Some("nvim".to_string()),
        };
        cfg.save_to(&path).unwrap();

        let loaded = Config::load_from(&path).unwrap();
        assert_eq!(loaded, cfg);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_file_errors_with_config_not_found() {
        let path = std::env::temp_dir().join("wcm-no-such-config-12345.toml");
        match Config::load_from(&path) {
            Err(Error::ConfigNotFound(_)) => {}
            other => panic!("unexpected result: {other:?}"),
        }
    }
}
