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
    git, template,
};

use std::{
    env,
    fs::File,
    io::{self, Write},
};

use clap::Args;

use color_eyre::eyre::{Result, WrapErr, eyre};

use ignore::WalkBuilder;

use pathdiff::diff_paths;

#[derive(Args)]
pub struct Print {
    /// Print only the file paths
    #[arg(long)]
    dry_run: bool,

    /// Use a template for printing
    #[arg(long, short, value_name = "TEMPLATE_NAME")]
    template: Option<String>,

    /// Positional arguments for the template
    #[arg()]
    template_args: Vec<String>,
}

impl Print {
    pub fn run(self) -> Result<()> {
        let Config { selection } = Config::read()?;

        let Some(selection) = selection else {
            return Ok(());
        };

        let git_root = git::repo_root()?;

        let selected_paths: Vec<SelectedPath> = selection
            .into_inner()
            .into_iter()
            .map(|p| SelectedPath::new(git_root.join(p.path), p.recursive))
            .collect();

        let current_dir = env::current_dir().wrap_err("failed to get current dir")?;

        let mut stdout = io::stdout();

        let rendered_output = if let Some(template_name_str) = &self.template {
            let template_name = template::TemplateName::new(template_name_str.clone());
            if self.dry_run {
                Some(template_name.to_string())
            } else {
                Some(template::render(&template_name, &self.template_args)?)
            }
        } else {
            None
        };

        if let Some(ref output) = rendered_output {
            writeln!(&mut stdout, "{}", output).wrap_err("failed to write to stdout")?;
        }

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
                        let relative_path = diff_paths(item.path(), &current_dir)
                            //
                            .ok_or_else(|| {
                                //
                                eyre!(
                                    //
                                    "failed to construct relative path for {}",
                                    //
                                    item.path().display()
                                )
                            })?;

                        if self.dry_run {
                            writeln!(&mut stdout, "{}", relative_path.display())
                                //
                                .wrap_err("failed to write to stdout")?;
                        } else {
                            let mut file = File::open(item.path())
                                //
                                .wrap_err_with(|| {
                                    //
                                    format!("failed to open file {}", item.path().display())
                                })?;

                            let error_message = "failed to write to stdout";

                            write!(&mut stdout, "<file path=\"{}\">\n", relative_path.display())
                                //
                                .wrap_err(error_message)?;

                            io::copy(&mut file, &mut stdout)
                                //
                                .wrap_err(error_message)?;

                            write!(&mut stdout, "</file>\n")
                                //
                                .wrap_err(error_message)?;
                        }
                    }
                }
            }
        }

        if let Some(ref output) = rendered_output {
            writeln!(&mut stdout, "{}", output).wrap_err("failed to write to stdout")?;
        }

        Ok(())
    }
}
