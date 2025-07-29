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

use crate::{command::utils, output, token};

use std::{fmt::Write, fs};

use clap::Args;

use color_eyre::eyre::{Result, WrapErr};

#[derive(Args)]
pub struct Print {
    /// Print only the file paths
    #[arg(long)]
    dry_run: bool,

    /// Copy the output to the clipboard instead of printing it
    #[arg(long, short)]
    copy: bool,
}

impl Print {
    pub fn run(self) -> Result<()> {
        let mut buf = String::new();
        let mut token_count = None;

        if self.dry_run {
            utils::walk_selected_files(|_abs_path, rel_path| {
                writeln!(&mut buf, "{}", rel_path.display())
                    .wrap_err("failed to write path to buffer")
            })?;
        } else {
            utils::walk_selected_files(|abs_path, rel_path| {
                write!(&mut buf, "<file path=\"{}\">\n", rel_path.display())
                    .wrap_err("failed to write file header to buffer")?;

                let file_content = fs::read_to_string(abs_path)
                    .wrap_err_with(|| format!("failed to read file {}", abs_path.display()))?;
                buf.push_str(&file_content);

                write!(&mut buf, "</file>\n").wrap_err("failed to write file footer to buffer")?;
                Ok(())
            })?;
            token_count = Some(token::estimate(&buf));
        }

        output::write(buf, self.copy)?;

        if let Some(count) = token_count {
            eprintln!("Approximate token count: {count}");
        }

        Ok(())
    }
}
