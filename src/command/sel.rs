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
    config::{Config, Selection, selection::SelectedPath},
    constants::CUSTOM_IGNORE_FILENAME,
    editor, git,
};

use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    fs,
    path::PathBuf,
};

use color_eyre::eyre::{eyre, Result, WrapErr};

use clap::Args;

use ignore::WalkBuilder;

use pathdiff::diff_paths;

#[derive(Args)]
pub struct Sel {
    roots: Vec<PathBuf>,
}

impl Sel {
    pub fn run(self) -> Result<()> {
        let mut config = Config::read()?;
        let git_root = git::repo_root()?;

        let selection = self.select(config.selection.take(), &git_root)?;

        let selection_len = selection.0.len();

        config.selection.replace(selection);

        config.write()?;

        println!("{selection_len} paths selected");

        Ok(())
    }

    fn select(
        &self,
        previous_selection: Option<Selection>,
        git_root: &PathBuf,
    ) -> Result<Selection> {
        // 1. Load existing selections from config, making paths absolute. This is our starting point.
        let mut final_paths: HashMap<PathBuf, bool> = previous_selection
            .unwrap_or_default()
            .into_inner()
            .into_iter()
            .map(|sp| (git_root.join(sp.path), sp.recursive))
            .collect();

        // Keep track of what was in the config to decide which paths are "new" suggestions.
        let originally_selected_paths: HashSet<PathBuf> = final_paths.keys().cloned().collect();

        // 2. Process explicit CLI roots.
        let canonical_roots: Vec<PathBuf> = self
            .roots
            .iter()
            .map(|p| {
                p.canonicalize()
                    .wrap_err_with(|| format!("Failed to find path {}", p.display()))
            })
            .collect::<Result<_>>()?;

        for path in &canonical_roots {
            // A CLI argument implies a recursive selection, but we don't override
            // an existing non-recursive (`*...`) setting from the config.
            final_paths.entry(path.clone()).or_insert(true);
        }

        // 3. Walk directories from CLI roots to discover NEW sub-directories.
        if let Some(first_root) = canonical_roots.first() {
            let mut walk_builder = WalkBuilder::new(first_root);

            for root in canonical_roots.iter().skip(1) {
                walk_builder.add(root);
            }

            walk_builder.add_custom_ignore_filename(CUSTOM_IGNORE_FILENAME);

            for result in walk_builder.build() {
                let item = result.wrap_err("failed to walk directories")?;

                if let Some(file_type) = item.file_type() {
                    if file_type.is_dir() {
                        // For discovered sub-directories, only add them if they are not
                        // already in our selection map. This preserves any `recursive: false`
                        // setting on existing selections.
                        final_paths.entry(item.into_path()).or_insert(true);
                    }
                }
            }
        }

        if final_paths.is_empty() {
            return Ok(Selection::default());
        }

        // 4. Prepare the buffer for the editor.
        let mut all_paths_vec: Vec<_> = final_paths
            .into_iter()
            .map(|(path, recursive)| SelectedPath::new(path, recursive))
            .collect();
        all_paths_vec.sort_unstable();

        let current_dir = env::current_dir().wrap_err("failed to get current dir")?;

        const HEADER: &str = "# Lines starting with '#' are ignored.\n\
                              # To select a path recursively, use its name: path/to/dir\n\
                              # To select a path non-recursively (only files in the directory), prefix with '*': *path/to/dir\n\n";

        let mut buf = String::from(HEADER);

        for path_item in &all_paths_vec {
            let relative_path = diff_paths(&path_item.path, &current_dir).ok_or_else(|| {
                eyre!(
                    "failed to construct relative path for {}",
                    path_item.path.display()
                )
            })?;

            // Comment out paths that are new suggestions (i.e., not in the original config).
            if !originally_selected_paths.contains(&path_item.path) {
                buf.push_str("# ");
            }

            let path_to_write = SelectedPath::new(relative_path, path_item.recursive);
            write!(&mut buf, "{}", path_to_write.to_string()).unwrap();
            buf.push('\n');
        }

        let cursor_line = HEADER.lines().count() + 1;
        let result = editor::get_user_input_from_file_content(&buf, cursor_line)?;

        // 5. Parse the user's final selection from the editor buffer.
        let mut paths = HashSet::new();
        let mut errors = Vec::new();

        let result_iter = result
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
            .map(|line| {
                let trimmed_line = line.trim();
                let selected_path_relative: SelectedPath = trimmed_line.parse().unwrap();
                fs::canonicalize(&selected_path_relative.path)
                    .map(|canonical| SelectedPath::new(canonical, selected_path_relative.recursive))
                    .wrap_err_with(|| {
                        format!(
                            "failed to canonicalize {}",
                            selected_path_relative.path.display()
                        )
                    })
            });

        for result in result_iter {
            match result {
                Ok(path) => {
                    paths.insert(path);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            let relative_paths = paths
                .into_iter()
                .map(|p| {
                    diff_paths(&p.path, git_root)
                        .ok_or_else(|| {
                            eyre!("failed to construct relative path for {}", p.path.display())
                        })
                        .map(|relative| SelectedPath::new(relative, p.recursive))
                })
                .collect::<Result<HashSet<_>>>()
                .wrap_err("failed to convert absolute paths to relative paths")?;

            let selection = Selection(relative_paths);

            Ok(selection)
        } else {
            let error = if errors.len() == 1 {
                errors.pop().unwrap()
            } else {
                errors
                    .into_iter()
                    .fold(eyre!("encountered multiple errors"), |acc, e| {
                        acc.wrap_err(e)
                    })
            };

            Err(error)
        }
    }
}
