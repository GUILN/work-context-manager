use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

/// Opens `file` with the configured editor, running it from `file`'s folder.
///
/// This is equivalent to `cd <folder> && <editor> <file>`: the editor
/// process inherits the current terminal and runs until the user exits it.
///
/// # Example
///
/// ```no_run
/// use context_manager::editor::open_with;
///
/// let path = std::path::Path::new("/tmp/work/my-work.md");
/// open_with(path, "nvim").expect("editor should launch");
/// ```
pub fn open_with(file: &Path, editor: &str) -> Result<()> {
    let folder = file
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    let mut parts = tokenize(editor);
    let program = if parts.is_empty() {
        return Err(Error::EmptyEditor);
    } else {
        parts.remove(0)
    };
    if program.is_empty() {
        return Err(Error::EmptyEditor);
    }

    let status = Command::new(&program)
        .args(parts)
        .arg(file)
        .current_dir(folder)
        .status()
        .map_err(|e| Error::EditorLaunch {
            editor: editor.to_string(),
            source: e,
        })?;

    if !status.success() {
        return Err(Error::EditorExit {
            editor: editor.to_string(),
            code: status.code(),
        });
    }
    Ok(())
}

/// Splits an editor command into tokens, respecting single quotes.
///
/// Example: `"code -w"` -> `["code", "-w"]`,
/// `"sh -c 'exit 3'"` -> `["sh", "-c", "exit 3"]`.
fn tokenize(command: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut token_started = false;

    for c in command.chars() {
        match c {
            '\'' if !in_quotes => {
                in_quotes = true;
                token_started = true;
            }
            '\'' if in_quotes => {
                in_quotes = false;
                token_started = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if token_started {
                    tokens.push(std::mem::take(&mut current));
                    token_started = false;
                }
            }
            c => {
                current.push(c);
                token_started = true;
            }
        }
    }
    if token_started {
        tokens.push(current);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_file_from_its_folder() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("nested");
        std::fs::create_dir_all(&folder).unwrap();
        let file = folder.join("my-work.md");
        std::fs::write(&file, "").unwrap();

        let script = dir.path().join("fake-editor.sh");
        let pwd_out = dir.path().join("pwd.txt");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\nprintf '%s' \"$(pwd)\" > \"{}\"\n",
                pwd_out.display()
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        open_with(&file, &format!("'{}'", script.display())).unwrap();

        let ran_from = std::fs::read_to_string(&pwd_out).unwrap();
        let expected = std::fs::canonicalize(&folder).unwrap();
        assert_eq!(ran_from, expected.to_str().unwrap());
    }

    #[test]
    fn empty_editor_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("f.md");
        std::fs::write(&file, "").unwrap();
        assert!(matches!(open_with(&file, "   "), Err(Error::EmptyEditor)));
    }

    #[test]
    fn missing_editor_binary_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("f.md");
        std::fs::write(&file, "").unwrap();
        assert!(matches!(
            open_with(&file, "wcm-no-such-editor-12345"),
            Err(Error::EditorLaunch { .. })
        ));
    }

    #[test]
    fn non_zero_exit_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("f.md");
        std::fs::write(&file, "").unwrap();
        assert!(matches!(
            open_with(&file, "sh -c 'exit 3'"),
            Err(Error::EditorExit { code: Some(3), .. })
        ));
    }
}
