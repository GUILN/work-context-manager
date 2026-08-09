use std::path::PathBuf;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::work_context::sanitize_kebab;

/// Lists the project folders inside the work context repo, sorted.
///
/// A missing repo and hidden folders (dotfiles) are ignored.
///
/// # Example
///
/// ```
/// use context_manager::{Config, project};
///
/// let dir = std::env::temp_dir().join("wcm-doc-list-projects");
/// let repo = dir.join("repo");
/// std::fs::create_dir_all(repo.join("alpha")).unwrap();
/// std::fs::create_dir_all(repo.join("beta")).unwrap();
/// std::fs::create_dir_all(repo.join(".hidden")).unwrap();
///
/// let cfg = Config {
///     template_folder: dir.join("templates"),
///     work_context_repo: repo,
///     editor: None,
/// };
///
/// let projects = project::list_projects(&cfg).unwrap();
/// assert_eq!(projects, vec!["alpha", "beta"]);
///
/// std::fs::remove_dir_all(&dir).ok();
/// ```
pub fn list_projects(config: &Config) -> Result<Vec<String>> {
    let repo = &config.work_context_repo;
    if !repo.exists() {
        return Ok(Vec::new());
    }
    let mut projects = Vec::new();
    for entry in std::fs::read_dir(repo)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        projects.push(name);
    }
    projects.sort();
    Ok(projects)
}

/// Creates a project folder inside the work context repo.
///
/// The project name is sanitized to kebab-case before being used as the
/// folder name, and the repo directory is created if it does not exist yet.
///
/// # Example
///
/// ```
/// use context_manager::{Config, project};
///
/// let dir = std::env::temp_dir().join("wcm-doc-create-project");
/// let repo = dir.join("repo");
/// let cfg = Config {
///     template_folder: dir.join("templates"),
///     work_context_repo: repo.clone(),
///     editor: None,
/// };
///
/// let path = project::create_project(&cfg, "Client Work").unwrap();
/// assert!(path.ends_with("client-work"));
/// assert!(path.is_dir());
///
/// std::fs::remove_dir_all(&dir).ok();
/// ```
pub fn create_project(config: &Config, name: &str) -> Result<PathBuf> {
    let project = sanitize_project_name(name)?;
    let path = config.work_context_repo.join(project);
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

/// Lists the markdown contexts inside a project folder, sorted.
///
/// Returns a [`Error::ProjectNotFound`] if the project folder does not exist.
///
/// # Example
///
/// ```
/// use context_manager::{Config, project};
///
/// let dir = std::env::temp_dir().join("wcm-doc-list-contexts");
/// let repo = dir.join("repo");
/// let project_dir = repo.join("alpha");
/// std::fs::create_dir_all(&project_dir).unwrap();
/// std::fs::write(project_dir.join("a.md"), "").unwrap();
/// std::fs::write(project_dir.join("b.md"), "").unwrap();
/// std::fs::write(project_dir.join("notes.txt"), "").unwrap();
///
/// let cfg = Config {
///     template_folder: dir.join("templates"),
///     work_context_repo: repo,
///     editor: None,
/// };
///
/// let contexts = project::list_contexts(&cfg, "alpha").unwrap();
/// let names: Vec<String> = contexts
///     .iter()
///     .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
///     .collect();
/// assert_eq!(names, vec!["a.md", "b.md"]);
///
/// std::fs::remove_dir_all(&dir).ok();
/// ```
pub fn list_contexts(config: &Config, project: &str) -> Result<Vec<PathBuf>> {
    let folder = config.work_context_repo.join(project);
    if !folder.exists() {
        return Err(Error::ProjectNotFound(project.to_string()));
    }
    let mut contexts = Vec::new();
    for entry in std::fs::read_dir(&folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            contexts.push(path);
        }
    }
    contexts.sort();
    Ok(contexts)
}

/// Sanitizes a project name into a valid folder name (kebab-case).
///
/// # Example
///
/// ```
/// use context_manager::project::sanitize_project_name;
///
/// assert_eq!(sanitize_project_name("Client Work").unwrap(), "client-work");
/// assert!(sanitize_project_name("   ").is_err());
/// ```
pub fn sanitize_project_name(name: &str) -> Result<String> {
    sanitize_kebab(name).map_err(|e| match e {
        crate::work_context::SanitizeError::Empty => Error::EmptyProjectName,
        crate::work_context::SanitizeError::Invalid(c) => {
            Error::InvalidProjectName(name.to_string(), c.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_config() -> (Config, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let cfg = Config {
            template_folder: dir.path().join("templates"),
            work_context_repo: dir.path().join("repo"),
            editor: None,
        };
        (cfg, dir)
    }

    #[test]
    fn lists_only_folders_sorted() {
        let (cfg, _dir) = tmp_config();
        let repo = &cfg.work_context_repo;
        std::fs::create_dir_all(repo.join("beta")).unwrap();
        std::fs::create_dir_all(repo.join("alpha")).unwrap();
        std::fs::create_dir_all(repo.join(".hidden")).unwrap();
        std::fs::write(repo.join("stray.md"), "").unwrap();

        assert_eq!(list_projects(&cfg).unwrap(), vec!["alpha", "beta"]);
    }

    #[test]
    fn missing_repo_yields_empty_list() {
        let (cfg, _dir) = tmp_config();
        assert!(list_projects(&cfg).unwrap().is_empty());
    }

    #[test]
    fn create_project_creates_sanitized_folder() {
        let (cfg, _dir) = tmp_config();
        let path = create_project(&cfg, "Client Work").unwrap();
        assert_eq!(path, cfg.work_context_repo.join("client-work"));
        assert!(path.is_dir());
    }

    #[test]
    fn create_project_rejects_empty() {
        let (cfg, _dir) = tmp_config();
        assert!(matches!(
            create_project(&cfg, "  "),
            Err(Error::EmptyProjectName)
        ));
    }

    #[test]
    fn lists_only_markdown_contexts_sorted() {
        let (cfg, _dir) = tmp_config();
        let project_dir = cfg.work_context_repo.join("alpha");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::write(project_dir.join("b.md"), "").unwrap();
        std::fs::write(project_dir.join("a.md"), "").unwrap();
        std::fs::write(project_dir.join("notes.txt"), "").unwrap();

        let contexts = list_contexts(&cfg, "alpha").unwrap();
        let names: Vec<String> = contexts
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["a.md", "b.md"]);
    }

    #[test]
    fn list_contexts_missing_project_is_an_error() {
        let (cfg, _dir) = tmp_config();
        assert!(matches!(
            list_contexts(&cfg, "ghost"),
            Err(Error::ProjectNotFound(project)) if project == "ghost"
        ));
    }
}
