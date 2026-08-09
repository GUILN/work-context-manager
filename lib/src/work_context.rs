use std::path::PathBuf;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::template::Template;

/// Creates a new work context file inside `project` in the configured repo,
/// rendering `template`.
///
/// The project and work names are sanitized to kebab-case before being used
/// as the folder/file names, and the project directory is created if it does
/// not exist yet.
///
/// # Example
///
/// ```
/// use context_manager::{Config, Template, work_context};
///
/// let dir = std::env::temp_dir().join("wcm-doc-new");
/// let repo = dir.join("repo");
/// let template_path = dir.join("templates");
/// std::fs::create_dir_all(&template_path).unwrap();
/// std::fs::write(template_path.join("general.md"), "# {{ name }}\n").unwrap();
///
/// let config = Config {
///     template_folder: template_path.clone(),
///     work_context_repo: repo.clone(),
///     editor: None,
/// };
/// let template = Template {
///     name: "general.md".into(),
///     path: template_path.join("general.md"),
/// };
///
/// let path =
///     work_context::new_work_context(&config, "Client Work", "My Work", &template).unwrap();
/// assert!(path.ends_with("client-work/my-work.md"));
/// assert_eq!(std::fs::read_to_string(&path).unwrap(), "# my-work\n");
///
/// std::fs::remove_dir_all(&dir).ok();
/// ```
pub fn new_work_context(
    config: &Config,
    project: &str,
    name: &str,
    template: &Template,
) -> Result<PathBuf> {
    let project = crate::project::sanitize_project_name(project)?;
    let name = sanitize_name(name)?;
    let folder = config.work_context_repo.join(project);
    std::fs::create_dir_all(&folder)?;

    let file_name = format!("{name}.md");
    let path = folder.join(file_name);
    let contents = template.render(&name)?;
    std::fs::write(&path, contents)?;
    Ok(path)
}

/// Sanitizes a work name into a valid & tidy file name (kebab-case).
///
/// # Example
///
/// ```
/// use context_manager::work_context::sanitize_name;
///
/// assert_eq!(sanitize_name("My Work").unwrap(), "my-work");
/// assert_eq!(sanitize_name("  Frontend   Refresh ").unwrap(), "frontend-refresh");
/// assert!(sanitize_name("   ").is_err());
/// ```
pub fn sanitize_name(name: &str) -> Result<String> {
    sanitize_kebab(name).map_err(|e| match e {
        SanitizeError::Empty => Error::EmptyWorkName,
        SanitizeError::Invalid(c) => Error::InvalidWorkName(name.to_string(), c.to_string()),
    })
}

pub(crate) enum SanitizeError {
    Empty,
    Invalid(char),
}

/// Shared kebab-case sanitizer used for both work and project names.
pub(crate) fn sanitize_kebab(name: &str) -> std::result::Result<String, SanitizeError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(SanitizeError::Empty);
    }

    let mut out = String::new();
    let mut prev_dash = false;
    for c in trimmed.chars() {
        if c.is_ascii_alphanumeric() {
            out.extend(c.to_lowercase());
            prev_dash = false;
        } else if c == '.' || c == '_' || c == '-' || c.is_whitespace() {
            if !prev_dash && !out.is_empty() {
                out.push('-');
                prev_dash = true;
            }
        } else {
            return Err(SanitizeError::Invalid(c));
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        return Err(SanitizeError::Empty);
    }
    Ok(out)
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
    fn sanitize_normalizes_names() {
        assert_eq!(sanitize_name("My Work").unwrap(), "my-work");
        assert_eq!(
            sanitize_name("  Frontend   Refresh ").unwrap(),
            "frontend-refresh"
        );
        assert_eq!(sanitize_name("Api-v2").unwrap(), "api-v2");
        assert_eq!(sanitize_name("a_b").unwrap(), "a-b");
    }

    #[test]
    fn sanitize_rejects_empty() {
        assert!(matches!(sanitize_name("   "), Err(Error::EmptyWorkName)));
    }

    #[test]
    fn sanitize_rejects_invalid_chars() {
        assert!(matches!(
            sanitize_name("my*work"),
            Err(Error::InvalidWorkName(..))
        ));
    }

    #[test]
    fn new_work_context_writes_rendered_file_inside_project() {
        let (cfg, _dir) = tmp_config();
        std::fs::create_dir_all(&cfg.template_folder).unwrap();
        let template_path = cfg.template_folder.join("general.md");
        std::fs::write(&template_path, "# {{ name }}\n").unwrap();
        let template = Template {
            name: "general.md".into(),
            path: template_path,
        };

        let written = new_work_context(&cfg, "Client Work", "My Work", &template).unwrap();
        let expected = cfg.work_context_repo.join("client-work").join("my-work.md");
        assert_eq!(written, expected);
        assert_eq!(std::fs::read_to_string(&expected).unwrap(), "# my-work\n");
    }
}
