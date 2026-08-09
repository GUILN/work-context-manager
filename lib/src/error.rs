use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("could not determine the home directory")]
    NoHomeDir,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not parse toml: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("could not serialize toml: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("config file not found at {0}")]
    ConfigNotFound(PathBuf),
    #[error("no templates found in {0}")]
    NoTemplates(PathBuf),
    #[error("work name is empty")]
    EmptyWorkName,
    #[error("invalid work name `{0}`: {1}")]
    InvalidWorkName(String, String),
}

pub type Result<T> = std::result::Result<T, Error>;
