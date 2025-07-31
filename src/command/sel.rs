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

use crate::{
    command::utils,
    config::Config,
    git, token,
};

use std::path::PathBuf;


use color_eyre::eyre::Result;

use clap::Args;

#[derive(Args)]
pub struct Sel {
    roots: Vec<PathBuf>,
}

impl Sel {
    pub fn run(self) -> Result<()> {
        let mut config = Config::read()?;
        let git_root = git::repo_root()?;

        let selection =
            utils::interactive_selection(&self.roots, config.selection.take(), &git_root)?;

        let selection_len = selection.0.len();

        config.selection.replace(selection.clone());

        config.write()?;

        if selection_len > 0 {
            let files_content = utils::get_selected_files_content_as_string(&selection)?;
            let token_count = token::estimate(&files_content);
            println!("{selection_len} paths selected. Approximate token count: {token_count}");
        } else {
            println!("0 paths selected");
        }

        Ok(())
    }
}
