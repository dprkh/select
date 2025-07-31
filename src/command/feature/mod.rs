use crate::{
    command::utils,
    editor,
    feature::{self, FeatureName},
    git, token,
};

use clap::{Args, Subcommand};
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;

#[derive(Args)]
pub struct Feature {
    #[command(subcommand)]
    pub command: Command,
}

impl Feature {
    pub fn run(self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new feature
    #[command(visible_alias = "c")]
    Create(Create),
    /// Delete a feature
    #[command(visible_alias = "d")]
    Delete(Delete),
    /// List all features
    #[command(visible_alias = "l")]
    List(List),
    /// Create or modify a selection for a feature
    #[command(visible_alias = "s")]
    Select(Select),
    /// Manage a feature's specification
    #[command(visible_alias = "sp")]
    Spec(Spec),
}

impl Command {
    fn run(self) -> Result<()> {
        match self {
            Command::Create(cmd) => cmd.run(),
            Command::Delete(cmd) => cmd.run(),
            Command::List(cmd) => cmd.run(),
            Command::Select(cmd) => cmd.run(),
            Command::Spec(cmd) => cmd.run(),
        }
    }
}

#[derive(Args)]
pub struct Create {
    /// Name of the feature to create
    name: String,
}

impl Create {
    fn run(self) -> Result<()> {
        let name = FeatureName::new(self.name);
        feature::create(&name)?;
        println!("Feature '{}' created.", name);
        Ok(())
    }
}

#[derive(Args)]
pub struct Delete {
    /// Name of the feature to delete
    name: String,
}

impl Delete {
    fn run(self) -> Result<()> {
        let name = FeatureName::new(self.name);
        feature::delete(&name)?;
        println!("Feature '{}' deleted.", name);
        Ok(())
    }
}

#[derive(Args)]
pub struct List {}

impl List {
    fn run(self) -> Result<()> {
        let features = feature::list()?;
        if features.is_empty() {
            println!("No features found.");
        } else {
            println!("Available features:");
            for name in features {
                println!("- {}", name);
            }
        }
        Ok(())
    }
}

#[derive(Args)]
pub struct Select {
    /// Name of the feature
    name: String,
    /// Paths to select from
    roots: Vec<PathBuf>,
}

impl Select {
    fn run(self) -> Result<()> {
        let name = FeatureName::new(self.name);
        if !feature::exists(&name)? {
            return Err(eyre!(
                "Feature '{}' does not exist. Create it first with 'feature create'.",
                name
            ));
        }

        let git_root = git::repo_root()?;
        let previous_selection = feature::read_selection(&name)?.unwrap_or_default();

        let selection =
            utils::interactive_selection(&self.roots, Some(previous_selection), &git_root)?;

        feature::write_selection(&name, &selection)?;

        let selection_len = selection.0.len();
        if selection_len > 0 {
            let files_content = utils::get_selected_files_content_as_string(&selection)?;
            let token_count = token::estimate(&files_content);
            println!(
                "Selection for feature '{name}' updated: {selection_len} paths. Approximate token count: {token_count}"
            );
        } else {
            println!("Selection for feature '{name}' is empty: 0 paths selected");
        }

        Ok(())
    }
}

#[derive(Args)]
pub struct Spec {
    #[command(subcommand)]
    pub command: SpecCommand,
}

impl Spec {
    fn run(self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Subcommand)]
pub enum SpecCommand {
    /// Create or edit a feature's spec
    #[command(visible_aliases = &["e", "c"])]
    Edit(EditSpec),
    /// Delete a feature's spec
    #[command(visible_alias = "d")]
    Delete(DeleteSpec),
}

impl SpecCommand {
    fn run(self) -> Result<()> {
        match self {
            SpecCommand::Edit(cmd) => cmd.run(),
            SpecCommand::Delete(cmd) => cmd.run(),
        }
    }
}

#[derive(Args)]
pub struct EditSpec {
    /// Name of the feature whose spec to edit
    name: String,
}

impl EditSpec {
    fn run(self) -> Result<()> {
        let name = FeatureName::new(self.name);
        if !feature::exists(&name)? {
            return Err(eyre!("Feature '{}' does not exist.", name));
        }
        let path = feature::get_spec_path(&name)?;
        if !path.exists() {
            let placeholder = "<!-- Specification for feature '{}'. This content will be included when rendering templates with this feature. -->\n";
            std::fs::write(&path, placeholder.replace("{}", name.as_str()))?;
            editor::open_in_vim(&path, 2)?;
            println!("Spec for feature '{}' created.", name);
        } else {
            editor::open_in_vim(&path, 1)?;
            println!("Spec for feature '{}' updated.", name);
        }
        Ok(())
    }
}

#[derive(Args)]
pub struct DeleteSpec {
    /// Name of the feature whose spec to delete
    name: String,
}

impl DeleteSpec {
    fn run(self) -> Result<()> {
        let name = FeatureName::new(self.name);
        if !feature::exists(&name)? {
            return Err(eyre!("Feature '{}' does not exist.", name));
        }
        feature::delete_spec(&name)?;
        println!("Spec for feature '{}' deleted.", name);
        Ok(())
    }
}
