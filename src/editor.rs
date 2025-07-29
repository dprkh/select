// MIT License
//
// Copyright (c) 2025 Dmytro Prokhorov
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::{fs, path::Path, process::Command};

use color_eyre::eyre::{Result, WrapErr};
use tempfile::NamedTempFile;

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

pub fn get_user_input_from_file_content(content: &str, cursor_line: usize) -> Result<String> {
    let file = NamedTempFile::new().wrap_err("failed to create a temporary file")?;

    fs::write(file.path(), content).wrap_err("failed to write to temporary file")?;

    open_in_vim(file.path(), cursor_line)?;

    fs::read_to_string(file.path()).wrap_err("failed to read temporary file")
}
