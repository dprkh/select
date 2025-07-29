use std::{path::PathBuf, process::Command};

use color_eyre::eyre::{eyre, Result, WrapErr};

pub fn repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .wrap_err("Failed to execute `git rev-parse --show-toplevel`. Is git installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!(
            "Could not find git repository root. `select` must be run inside a git repository.\nGit error: {}",
            stderr.trim()
        ));
    }

    let git_root_str = String::from_utf8(output.stdout)
        .wrap_err("`git rev-parse --show-toplevel` output was not valid UTF-8")?;

    let git_root = PathBuf::from(git_root_str.trim());
    Ok(git_root)
}
