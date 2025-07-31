use std::{fs, io::Write, path::Path, process::Command};

use color_eyre::eyre::{Result, WrapErr};
use tempfile::Builder;

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

pub fn get_user_input_from_file_content(
    content: &str,
    cursor_line: usize,
    extension: Option<&str>,
) -> Result<String> {
    let mut builder = Builder::new();
    if let Some(ext) = extension {
        builder.suffix(ext);
    }

    let mut file = builder
        .tempfile_in(std::env::temp_dir())
        .wrap_err("failed to create a temporary file")?;

    file.write_all(content.as_bytes())
        .wrap_err("failed to write to temporary file")?;

    let temp_path = file.into_temp_path();

    open_in_vim(&temp_path, cursor_line)?;

    let result = fs::read_to_string(&temp_path).wrap_err("failed to read temporary file");

    result
}
