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
    editor,
    template::{self, TemplateName},
};

use clap::{Args, Subcommand};
use color_eyre::eyre::{Result, eyre};

#[derive(Args)]
pub struct Template {
    #[command(subcommand)]
    pub command: Command,
}

impl Template {
    pub fn run(self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new template
    #[command(visible_alias = "c")]
    Create(Create),
    /// Edit an existing template
    #[command(visible_alias = "e")]
    Edit(Edit),
    /// Delete a template
    #[command(visible_alias = "d")]
    Delete(Delete),
    /// List all templates
    #[command(visible_alias = "l")]
    List(List),
}

impl Command {
    fn run(self) -> Result<()> {
        match self {
            Command::Create(cmd) => cmd.run(),
            Command::Edit(cmd) => cmd.run(),
            Command::Delete(cmd) => cmd.run(),
            Command::List(cmd) => cmd.run(),
        }
    }
}

#[derive(Args)]
pub struct Create {
    /// Name of the template to create
    name: String,
}

impl Create {
    fn run(self) -> Result<()> {
        let name = TemplateName::new(self.name);
        if template::exists(&name)? {
            return Err(eyre!(
                "Template '{}' already exists. Use 'edit' command.",
                name
            ));
        }

        let path = template::file_path(&name)?;
        let placeholder = "{# Template for 'sel print'. Use {{ args[0] }}, {{ args[1] }}, etc. for arguments. #}\n";
        std::fs::write(&path, placeholder)?;

        editor::open_in_vim(&path, 2)?;
        println!("Template '{}' created.", name);
        Ok(())
    }
}

#[derive(Args)]
pub struct Edit {
    /// Name of the template to edit
    name: String,
}

impl Edit {
    fn run(self) -> Result<()> {
        let name = TemplateName::new(self.name);
        if !template::exists(&name)? {
            return Err(eyre!(
                "Template '{}' does not exist. Use 'create' command.",
                name
            ));
        }
        let path = template::file_path(&name)?;
        editor::open_in_vim(&path, 1)?;
        println!("Template '{}' updated.", name);
        Ok(())
    }
}

#[derive(Args)]
pub struct Delete {
    /// Name of the template to delete
    name: String,
}

impl Delete {
    fn run(self) -> Result<()> {
        let name = TemplateName::new(self.name);
        template::delete(&name)?;
        println!("Template '{}' deleted.", name);
        Ok(())
    }
}

#[derive(Args)]
pub struct List {}

impl List {
    fn run(self) -> Result<()> {
        let templates = template::list()?;
        if templates.is_empty() {
            println!("No templates found.");
        } else {
            println!("Available templates:");
            for name in templates {
                println!("- {}", name);
            }
        }
        Ok(())
    }
}
