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

use crate::config::Selection;
use crate::git;

use std::{fs, io, path::PathBuf};

use color_eyre::eyre::{Report, Result, WrapErr};

use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub selection: Option<Selection>,
}

fn file_path() -> Result<PathBuf> {
    let file_path = git::repo_root()?
        //
        .join(".select")
        //
        .join("select.toml");

    Ok(file_path)
}

impl Config {
    pub fn read() -> Result<Self> {
        let file_path = file_path()?;

        let bytes = match fs::read(&file_path) {
            Ok(bytes) => bytes,

            Err(e) => {
                return if let io::ErrorKind::NotFound = e.kind() {
                    Ok(Self::default())
                } else {
                    let message = format!("failed to read config from {}", file_path.display());

                    Err(Report::new(e).wrap_err(message))
                };
            }
        };

        let config = toml::from_slice(&bytes).wrap_err("failed to parse config file")?;

        Ok(config)
    }

    pub fn write(&self) -> Result<()> {
        let string = toml::to_string_pretty(self).wrap_err("failed to serialize config")?;

        let file_path = file_path()?;

        if let Some(parent_path) = file_path.parent() {
            fs::create_dir_all(parent_path)
                //
                .wrap_err_with(|| {
                    format!("failed to create directories for {}", parent_path.display())
                })?;
        }

        fs::write(&file_path, string)
            //
            .wrap_err_with(|| format!("failed to write config to {}", file_path.display()))?;

        Ok(())
    }
}
