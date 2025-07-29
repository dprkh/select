use crate::command::*;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Select files.
    #[command(visible_alias = "s")]
    Select(Select),

    /// Print selected files.
    #[command(visible_alias = "p")]
    Print(Print),
}
