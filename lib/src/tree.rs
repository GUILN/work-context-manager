use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::Result;

/// A node in the work context tree: either a folder or a context file.
///
/// Folders are listed before files, and names are sorted alphabetically
/// within each group. Hidden entries (dotfiles) are skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }

    pub fn is_file(&self) -> bool {
        !self.is_dir
    }
}

/// Builds the recursive tree of projects and contexts under the work context
/// repo. The root node represents the repo itself.
///
/// # Example
///
/// ```
/// use context_manager::{Config, tree};
///
/// let dir = std::env::temp_dir().join("wcm-doc-tree");
/// let repo = dir.join("repo");
/// std::fs::create_dir_all(repo.join("client-work/nested")).unwrap();
/// std::fs::create_dir_all(repo.join(".hidden")).unwrap();
/// std::fs::write(repo.join("client-work/one.md"), "").unwrap();
/// std::fs::write(repo.join("client-work/nested/two.md"), "").unwrap();
/// std::fs::write(repo.join("standalone.md"), "").unwrap();
/// std::fs::write(repo.join("ignored.txt"), "").unwrap();
///
/// let cfg = Config {
///     template_folder: dir.join("templates"),
///     work_context_repo: repo,
///     editor: None,
/// };
///
/// let root = tree::build_tree(&cfg).unwrap();
/// assert!(root.is_dir());
/// let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
/// assert_eq!(names, vec!["client-work", "standalone.md"]);
/// let client = &root.children[0];
/// assert_eq!(client.children.len(), 2);
///
/// std::fs::remove_dir_all(&dir).ok();
/// ```
pub fn build_tree(config: &Config) -> Result<TreeNode> {
    build_tree_from(&config.work_context_repo)
}

/// Builds the recursive tree starting at `root`. The root node represents
/// `root` itself; folders are listed before files, hidden entries are skipped.
///
/// # Example
///
/// ```
/// use context_manager::tree;
///
/// let dir = std::env::temp_dir().join("wcm-doc-tree-from");
/// std::fs::create_dir_all(dir.join("sub")).unwrap();
/// std::fs::write(dir.join("a.md"), "").unwrap();
/// std::fs::write(dir.join("sub/b.md"), "").unwrap();
///
/// let root = tree::build_tree_from(&dir).unwrap();
/// let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
/// assert_eq!(names, vec!["sub", "a.md"]);
///
/// std::fs::remove_dir_all(&dir).ok();
/// ```
pub fn build_tree_from(root: &Path) -> Result<TreeNode> {
    build_node(root)
}

fn build_node(path: &Path) -> Result<TreeNode> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    let mut children = Vec::new();
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let child_path = entry.path();
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }
            if child_path.is_dir() {
                children.push(build_node(&child_path)?);
            } else if name.ends_with(".md") {
                children.push(TreeNode {
                    name,
                    path: child_path,
                    is_dir: false,
                    children: Vec::new(),
                });
            }
        }
        children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });
    }

    Ok(TreeNode {
        name,
        path: path.to_path_buf(),
        is_dir: true,
        children,
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
    fn builds_recursive_tree_with_folders_first() {
        let (cfg, _dir) = tmp_config();
        let repo = &cfg.work_context_repo;
        std::fs::create_dir_all(repo.join("zeta/nested")).unwrap();
        std::fs::create_dir_all(repo.join("alpha")).unwrap();
        std::fs::write(repo.join("zeta/two.md"), "").unwrap();
        std::fs::write(repo.join("zeta/nested/deep.md"), "").unwrap();
        std::fs::write(repo.join("alpha/one.md"), "").unwrap();
        std::fs::write(repo.join("standalone.md"), "").unwrap();
        std::fs::write(repo.join("ignored.txt"), "").unwrap();

        let root = build_tree(&cfg).unwrap();
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "zeta", "standalone.md"]);

        let zeta = &root.children[1];
        assert!(zeta.is_dir());
        let zeta_names: Vec<&str> = zeta.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(zeta_names, vec!["nested", "two.md"]);
        assert!(zeta.children[0].children[0].name == "deep.md");
    }

    #[test]
    fn skips_hidden_entries() {
        let (cfg, _dir) = tmp_config();
        let repo = &cfg.work_context_repo;
        std::fs::create_dir_all(repo.join(".hidden")).unwrap();
        std::fs::write(repo.join(".obsidian"), "").unwrap();

        let root = build_tree(&cfg).unwrap();
        assert!(root.children.is_empty());
    }

    #[test]
    fn missing_repo_yields_empty_root() {
        let (cfg, _dir) = tmp_config();
        let root = build_tree(&cfg).unwrap();
        assert!(root.is_dir());
        assert!(root.children.is_empty());
    }
}
