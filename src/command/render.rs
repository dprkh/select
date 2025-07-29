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
    editor,
    template::{self, TemplateName},
};

use std::{
    fs::File,
    io::{self, Write},
};

use clap::Args;
use color_eyre::eyre::{Result, WrapErr, eyre};
use serde::Serialize;

#[derive(Args)]
pub struct Render {
    /// Name of the template to use
    template: String,
}

#[derive(Serialize)]
struct RenderContext {
    task: String,
}

impl Render {
    pub fn run(self) -> Result<()> {
        let template_name = TemplateName::new(self.template);
        if !template::exists(&template_name)? {
            return Err(eyre!("Template '{}' does not exist.", template_name));
        }

        let task = get_task_from_editor()?;

        let context = RenderContext { task };

        let output = template::render(&template_name, &context)?;

        let mut stdout = io::stdout();

        // 1. Print rendered template
        writeln!(&mut stdout, "{}", output).wrap_err("failed to write to stdout")?;

        // 2. Print selected files
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

        // 3. Print rendered template again
        write!(&mut stdout, "{}", output).wrap_err("failed to write to stdout")?;

        Ok(())
    }
}

fn get_task_from_editor() -> Result<String> {
    const HEADER: &str =
        "# Enter your task description. Lines starting with '#' are ignored.\n\n\n";
    let cursor_line = HEADER.split('\n').count();

    let content = editor::get_user_input_from_file_content(HEADER, cursor_line)?;

    let task_content: String = content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(task_content)
}
