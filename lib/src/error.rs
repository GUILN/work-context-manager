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
    #[error("project name is empty")]
    EmptyProjectName,
    #[error("invalid project name `{0}`: {1}")]
    InvalidProjectName(String, String),
    #[error("template folder name is empty")]
    EmptyTemplateFolderName,
    #[error("invalid template folder name `{0}`: {1}")]
    InvalidTemplateFolderName(String, String),
    #[error("no projects found in {0}")]
    NoProjects(PathBuf),
    #[error("project `{0}` not found")]
    ProjectNotFound(String),
    #[error("editor command is empty")]
    EmptyEditor,
    #[error("failed to launch editor `{editor}`: {source}")]
    EditorLaunch {
        editor: String,
        #[source]
        source: std::io::Error,
    },
    #[error("editor `{editor}` exited with code {code:?}")]
    EditorExit { editor: String, code: Option<i32> },
}

pub type Result<T> = std::result::Result<T, Error>;
