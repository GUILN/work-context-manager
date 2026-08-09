use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{Error, Result};

/// A discoverable markdown template in the configured template folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub name: String,
    pub path: PathBuf,
}

impl Template {
    /// Reads the template contents as a string.
    ///
    /// # Example
    ///
    /// ```
    /// use context_manager::Template;
    ///
    /// let dir = std::env::temp_dir().join("wcm-doc-contents");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let path = dir.join("t.md");
    /// std::fs::write(&path, "# Hello").unwrap();
    ///
    /// let template = Template { name: "t.md".into(), path };
    /// assert_eq!(template.contents().unwrap(), "# Hello");
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// ```
    pub fn contents(&self) -> Result<String> {
        Ok(std::fs::read_to_string(&self.path)?)
    }

    /// Renders the template contents, replacing `{{ name }}` with `name`.
    ///
    /// # Example
    ///
    /// ```
    /// use context_manager::Template;
    ///
    /// let dir = std::env::temp_dir().join("wcm-doc-render");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// let path = dir.join("t.md");
    /// std::fs::write(&path, "# {{ name }}\n").unwrap();
    ///
    /// let template = Template { name: "t.md".into(), path };
    /// assert_eq!(template.render("my work").unwrap(), "# my work\n");
    ///
    /// std::fs::remove_dir_all(&dir).ok();
    /// ```
    pub fn render(&self, name: &str) -> Result<String> {
        Ok(self.contents()?.replace("{{ name }}", name))
    }
}

/// Lists markdown templates inside `folder`, descending into subdirectories.
///
/// Template names are relative paths from `folder` (e.g. `daily/standup.md`),
/// returned sorted by name. Hidden entries (dotfiles) and a missing folder are
/// ignored (a missing folder yields an empty list).
///
/// # Example
///
/// ```
/// use context_manager::template::list_templates;
///
/// let dir = std::env::temp_dir().join("wcm-doc-list");
/// std::fs::create_dir_all(dir.join("daily")).unwrap();
/// std::fs::create_dir_all(dir.join(".hidden")).unwrap();
/// std::fs::write(dir.join("a.md"), "").unwrap();
/// std::fs::write(dir.join("b.txt"), "").unwrap();
/// std::fs::write(dir.join("daily/standup.md"), "").unwrap();
/// std::fs::write(dir.join(".hidden/secret.md"), "").unwrap();
///
/// let templates = list_templates(&dir).unwrap();
/// let names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
/// assert_eq!(names, vec!["a.md", "daily/standup.md"]);
///
/// std::fs::remove_dir_all(&dir).ok();
/// ```
pub fn list_templates(folder: &Path) -> Result<Vec<Template>> {
    if !folder.exists() {
        return Ok(Vec::new());
    }
    let mut templates = Vec::new();
    collect_templates(folder, folder, &mut templates)?;
    templates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(templates)
}

fn collect_templates(folder: &Path, root: &Path, out: &mut Vec<Template>) -> Result<()> {
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_templates(&path, root, out)?;
        } else if name.ends_with(".md") {
            let rel = path
                .strip_prefix(root)
                .expect("path always under root")
                .to_string_lossy()
                .into_owned();
            out.push(Template { name: rel, path });
        }
    }
    Ok(())
}

/// Creates a sub-folder inside the template folder.
///
/// The folder name is sanitized to kebab-case before being used as the folder
/// name, and the template folder is created if it does not exist yet.
///
/// # Example
///
/// ```
/// use context_manager::{Config, template};
///
/// let dir = std::env::temp_dir().join("wcm-doc-create-template-folder");
/// let template_folder = dir.join("templates");
/// let cfg = Config {
///     template_folder: template_folder.clone(),
///     work_context_repo: dir.join("repo"),
///     editor: None,
/// };
///
/// let path = template::create_template_folder(&cfg, "Daily Notes").unwrap();
/// assert!(path.ends_with("daily-notes"));
/// assert!(path.is_dir());
///
/// std::fs::remove_dir_all(&dir).ok();
/// ```
pub fn create_template_folder(config: &Config, name: &str) -> Result<PathBuf> {
    let folder = sanitize_template_folder_name(name)?;
    let path = config.template_folder.join(folder);
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

/// Sanitizes a template folder name into a valid folder name (kebab-case).
///
/// # Example
///
/// ```
/// use context_manager::template::sanitize_template_folder_name;
///
/// assert_eq!(
///     sanitize_template_folder_name("Daily Notes").unwrap(),
///     "daily-notes"
/// );
/// assert!(sanitize_template_folder_name("   ").is_err());
/// ```
pub fn sanitize_template_folder_name(name: &str) -> Result<String> {
    crate::work_context::sanitize_kebab(name).map_err(|e| match e {
        crate::work_context::SanitizeError::Empty => Error::EmptyTemplateFolderName,
        crate::work_context::SanitizeError::Invalid(c) => {
            Error::InvalidTemplateFolderName(name.to_string(), c.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn lists_only_markdown_files_sorted_recursively() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        write(dir, "general.md", "a");
        write(dir, "notes.md", "b");
        write(dir, "ignored.txt", "x");
        write(dir, "no_extension", "y");
        let sub_dir = dir.join("a-subdir");
        std::fs::create_dir(&sub_dir).unwrap();
        write(&sub_dir, "nested.md", "z");
        std::fs::create_dir_all(dir.join(".hidden")).unwrap();
        write(&dir.join(".hidden"), "secret.md", "s");

        let templates = list_templates(dir).unwrap();
        let names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["a-subdir/nested.md", "general.md", "notes.md"]);
    }

    #[test]
    fn missing_folder_yields_empty_list() {
        let missing = std::env::temp_dir().join("wcm-no-templates-12345");
        assert!(list_templates(&missing).unwrap().is_empty());
    }

    #[test]
    fn render_replaces_placeholder() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write(tmp.path(), "t.md", "# {{ name }}\n\nhello");
        assert_eq!(
            Template {
                path,
                name: "t.md".into()
            }
            .render("my work")
            .unwrap(),
            "# my work\n\nhello"
        );
    }

    #[test]
    fn create_template_folder_sanitizes_and_creates() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = Config {
            template_folder: tmp.path().join("templates"),
            work_context_repo: tmp.path().join("repo"),
            editor: None,
        };
        let path = create_template_folder(&cfg, "Daily Notes").unwrap();
        assert_eq!(path, cfg.template_folder.join("daily-notes"));
        assert!(path.is_dir());
    }

    #[test]
    fn create_template_folder_rejects_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = Config {
            template_folder: tmp.path().join("templates"),
            work_context_repo: tmp.path().join("repo"),
            editor: None,
        };
        assert!(matches!(
            create_template_folder(&cfg, "  "),
            Err(Error::EmptyTemplateFolderName)
        ));
    }
}
