use std::path::PathBuf;
use std::process::Command;

use anyhow::{ensure, Context, Result};

const BIN_NAME: &str = "context-manager";

fn main() {
    let result = (|| -> Result<()> {
        let repo_root = repo_root();
        build_release(&repo_root)?;
        copy_bin(&repo_root)?;
        Ok(())
    })();
    match result {
        Ok(()) => println!("installed {BIN_NAME} to {}", install_dir().display()),
        Err(err) => {
            eprintln!("install failed: {err:#}");
            std::process::exit(1);
        }
    }
}

fn repo_root() -> PathBuf {
    // installer/Cargo.toml -> workspace root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("installer lives under the workspace root")
        .to_path_buf()
}

fn build_release(repo_root: &std::path::Path) -> anyhow::Result<()> {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(repo_root.join("Cargo.toml"))
        .status()
        .context("failed to run cargo")?;
    ensure!(status.success(), "cargo build --release failed");
    Ok(())
}

fn copy_bin(repo_root: &std::path::Path) -> anyhow::Result<()> {
    let dir = install_dir();
    std::fs::create_dir_all(&dir)?;
    let source = repo_root.join("target").join("release").join(BIN_NAME);
    let dest = dir.join(BIN_NAME);
    std::fs::copy(&source, &dest)
        .with_context(|| format!("failed to copy {} to {}", source.display(), dest.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

fn install_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("WCM_INSTALL_DIR") {
        return PathBuf::from(dir);
    }
    let home = dirs::home_dir().expect("no home directory");
    home.join(".local").join("bin")
}
