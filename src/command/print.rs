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

use crate::command::utils;

use std::{
    fs::File,
    io::{self, Write},
};

use clap::Args;

use color_eyre::eyre::{Result, WrapErr};

#[derive(Args)]
pub struct Print {
    /// Print only the file paths
    #[arg(long)]
    dry_run: bool,
}

impl Print {
    pub fn run(self) -> Result<()> {
        let mut stdout = io::stdout();

        if self.dry_run {
            utils::walk_selected_files(|_abs_path, rel_path| {
                writeln!(&mut stdout, "{}", rel_path.display())
                    .wrap_err("failed to write to stdout")
            })?;
        } else {
            utils::walk_selected_files(|abs_path, rel_path| {
                let mut file = File::open(abs_path)
                    .wrap_err_with(|| format!("failed to open file {}", abs_path.display()))?;

                let error_message = "failed to write to stdout";

                write!(&mut stdout, "<file path=\"{}\">\n", rel_path.display())
                    .wrap_err(error_message)?;

                io::copy(&mut file, &mut stdout).wrap_err(error_message)?;

                write!(&mut stdout, "</file>\n").wrap_err(error_message)?;
                Ok(())
            })?;
        }

        Ok(())
    }
}
