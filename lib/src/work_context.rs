use std::path::PathBuf;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::template::Template;

/// Creates a new work context file in the configured repo, rendering `template`.
pub fn new_work_context(config: &Config, name: &str, template: &Template) -> Result<PathBuf> {
    let name = sanitize_name(name)?;
    std::fs::create_dir_all(&config.work_context_repo)?;

    let file_name = format!("{name}.md");
    let path = config.work_context_repo.join(file_name);
    let contents = template.render(&name)?;
    std::fs::write(&path, contents)?;
    Ok(path)
}

/// Sanitizes a work name into a valid & tidy file name (kebab-case).
pub fn sanitize_name(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(Error::EmptyWorkName);
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
            return Err(Error::InvalidWorkName(name.to_string(), c.to_string()));
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        return Err(Error::EmptyWorkName);
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
    fn new_work_context_writes_rendered_file() {
        let (cfg, _dir) = tmp_config();
        std::fs::create_dir_all(&cfg.template_folder).unwrap();
        let template_path = cfg.template_folder.join("general.md");
        std::fs::write(&template_path, "# {{ name }}\n").unwrap();
        let template = Template {
            name: "general.md".into(),
            path: template_path,
        };

        let written = new_work_context(&cfg, "My Work", &template).unwrap();
        let expected = cfg.work_context_repo.join("my-work.md");
        assert_eq!(written, expected);
        assert_eq!(std::fs::read_to_string(&expected).unwrap(), "# my-work\n");
    }
}
