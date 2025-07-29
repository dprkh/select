use std::path::Path;
use std::process::Command;

use color_eyre::eyre::{Result, WrapErr};

pub fn open_in_vim(path: &Path, cursor_line: usize) -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    Command::new(editor)
        .arg(format!("+{}", cursor_line))
        .arg(path)
        .spawn()
        .wrap_err("failed to start editor")?
        .wait()
        .wrap_err("failed to wait for editor")?;

    Ok(())
}
