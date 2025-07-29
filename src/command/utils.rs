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
    config::{Config, selection::SelectedPath},
    constants::CUSTOM_IGNORE_FILENAME,
    git,
};
use color_eyre::eyre::{Result, WrapErr, eyre};
use ignore::WalkBuilder;
use pathdiff::diff_paths;
use std::{env, path::Path};

/// Walks through all selected files and calls a closure for each file.
///
/// The closure `on_file` is called with two arguments:
/// 1. The absolute path of the file.
/// 2. The path of the file relative to the current working directory.
pub fn walk_selected_files<F>(mut on_file: F) -> Result<()>
where
    F: FnMut(&Path, &Path) -> Result<()>,
{
    let Config { selection } = Config::read()?;
    let Some(selection) = selection else {
        return Ok(());
    };

    let git_root = git::repo_root()?;
    let current_dir = env::current_dir().wrap_err("failed to get current dir")?;

    let selected_paths: Vec<SelectedPath> = selection
        .into_inner()
        .into_iter()
        .map(|p| SelectedPath::new(git_root.join(p.path), p.recursive))
        .collect();

    for selected_path in selected_paths {
        let mut walk_builder = WalkBuilder::new(&selected_path.path);
        walk_builder.add_custom_ignore_filename(CUSTOM_IGNORE_FILENAME);
        if !selected_path.recursive {
            // max_depth 1 means the root and its direct children.
            // The root dir itself will be filtered out by `is_file()` check.
            walk_builder.max_depth(Some(1));
        }

        for result in walk_builder.build() {
            let item = result.wrap_err("failed to walk directories")?;

            if let Some(file_type) = item.file_type() {
                if file_type.is_file() {
                    let absolute_path = item.path();
                    let relative_path =
                        diff_paths(absolute_path, &current_dir).ok_or_else(|| {
                            eyre!(
                                "failed to construct relative path for {}",
                                absolute_path.display()
                            )
                        })?;

                    on_file(absolute_path, &relative_path)?;
                }
            }
        }
    }

    Ok(())
}
