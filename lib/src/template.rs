use std::path::{Path, PathBuf};

use crate::error::Result;

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
    /// use work_context_manager::Template;
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
    /// use work_context_manager::Template;
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

/// Lists markdown templates inside `folder`.
///
/// Returns the templates sorted by name. Non-markdown files, subdirectories,
/// and a missing folder are ignored (a missing folder yields an empty list).
///
/// # Example
///
/// ```
/// use work_context_manager::template::list_templates;
///
/// let dir = std::env::temp_dir().join("wcm-doc-list");
/// std::fs::create_dir_all(&dir).unwrap();
/// std::fs::write(dir.join("a.md"), "").unwrap();
/// std::fs::write(dir.join("b.txt"), "").unwrap();
///
/// let templates = list_templates(&dir).unwrap();
/// let names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
/// assert_eq!(names, vec!["a.md"]);
///
/// std::fs::remove_dir_all(&dir).ok();
/// ```
pub fn list_templates(folder: &Path) -> Result<Vec<Template>> {
    if !folder.exists() {
        return Ok(Vec::new());
    }
    let mut templates = Vec::new();
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if name.ends_with(".md") {
            templates.push(Template { name, path });
        }
    }
    templates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(templates)
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
    fn lists_only_markdown_files_sorted() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        write(dir, "general.md", "a");
        write(dir, "notes.md", "b");
        write(dir, "ignored.txt", "x");
        write(dir, "no_extension", "y");
        let sub_dir = dir.join("a-subdir");
        std::fs::create_dir(&sub_dir).unwrap();
        write(&sub_dir, "nested.md", "z");

        let templates = list_templates(dir).unwrap();
        let names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["general.md", "notes.md"]);
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
}
