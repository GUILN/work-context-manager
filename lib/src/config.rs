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
}

impl Config {
    /// Returns `~/.work_context_manager`.
    pub fn config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or(Error::NoHomeDir)?;
        Ok(home.join(CONFIG_DIR_NAME))
    }

    /// Returns `~/.work_context_manager/config.toml`.
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(CONFIG_FILE_NAME))
    }

    /// Creates a default config based on the home directory.
    pub fn default_config() -> Result<Self> {
        let home = dirs::home_dir().ok_or(Error::NoHomeDir)?;
        Ok(Self {
            template_folder: home.join(CONFIG_DIR_NAME).join("templates"),
            work_context_repo: home.join("work_contexts"),
        })
    }

    /// Loads the config from `~/.work_context_manager/config.toml`.
    pub fn load() -> Result<Self> {
        Self::load_from(&Self::config_path()?)
    }

    /// Loads the config from an explicit path.
    pub fn load_from(path: &std::path::Path) -> Result<Self> {
        let raw =
            std::fs::read_to_string(path).map_err(|_| Error::ConfigNotFound(path.to_path_buf()))?;
        Ok(toml::from_str(&raw)?)
    }

    /// Writes the config to its default location, creating parent dirs.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        self.save_to(&path)
    }

    /// Writes the config to an explicit path, creating parent dirs.
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
