use crate::{
    command::utils,
    editor,
    feature::{self, FeatureName},
    output,
    template::{self, TemplateName},
    token,
};

use std::{fmt::Write, fs};

use clap::Args;
use color_eyre::eyre::{eyre, Result, WrapErr};
use serde::Serialize;

#[derive(Args)]
pub struct Render {
    /// Name of the template to use
    template: String,

    /// Use a feature's selection and spec
    #[arg(short, long)]
    feature: Option<String>,
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

        let (selection, spec) = if let Some(feature_name_str) = self.feature {
            let feature_name = FeatureName::new(feature_name_str);
            if !feature::exists(&feature_name)? {
                return Err(eyre!("Feature '{}' does not exist.", feature_name));
            }
            let selection = feature::read_selection(&feature_name)?.unwrap_or_default();
            let spec = feature::read_spec(&feature_name)?;
            (selection, spec)
        } else {
            (utils::get_global_selection()?, None)
        };

        let task = get_task_from_editor()?;

        let context = RenderContext { task };

        let rendered_template = template::render(&template_name, &context)?;

        let mut buf = String::new();

        let spec_block = spec.map(|s| {
            let cleaned_spec = remove_comments_from_string(s);
            format!("<spec>\n{cleaned_spec}\n</spec>\n")
        });

        // 1. Print spec and rendered template
        if let Some(ref s) = spec_block {
            buf.push_str(s);
        }
        writeln!(&mut buf, "{}", rendered_template).wrap_err("failed to write to buffer")?;

        // 2. Print selected files
        utils::walk_selected_files(&selection, |abs_path, rel_path| {
            write!(&mut buf, "<file path=\"{}\">\n", rel_path.display())
                .wrap_err("failed to write file header to buffer")?;

            let file_content = fs::read_to_string(abs_path)
                .wrap_err_with(|| format!("failed to read file {}", abs_path.display()))?;
            buf.push_str(&file_content);

            write!(&mut buf, "</file>\n").wrap_err("failed to write file footer to buffer")?;
            Ok(())
        })?;

        // 3. Print spec and rendered template again
        if let Some(ref s) = spec_block {
            buf.push_str(s);
        }
        write!(&mut buf, "{}", rendered_template).wrap_err("failed to write to buffer")?;

        let token_count = token::estimate(&buf);

        output::copy_to_clipboard(buf)?;

        eprintln!("Approximate token count: {token_count}");

        Ok(())
    }
}

fn get_task_from_editor() -> Result<String> {
    const HEADER: &str =
        "<!-- Enter your task description. Content in markdown comments will be ignored. -->\n\n";
    let cursor_line = HEADER.lines().count();

    let content = editor::get_user_input_from_file_content(HEADER, cursor_line, Some(".md"))?;

    let cleaned_content = remove_comments_from_string(content);

    let task_content: String = cleaned_content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(task_content)
}

fn remove_comments_from_string(mut text: String) -> String {
    loop {
        if let Some(start) = text.find("<!--") {
            if let Some(end) = text[start..].find("-->") {
                text.replace_range(start..start + end + 3, "");
            } else {
                // unclosed comment, remove it to avoid infinite loop
                text.replace_range(start..start + 4, "");
            }
        } else {
            break;
        }
    }
    text
}
