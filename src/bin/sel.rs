use color_eyre::eyre::Result;

use clap::Parser;

use select::cli::{Cli, Command};

pub fn main() -> Result<()> {
    let Cli { command } = Cli::parse();

    color_eyre::install()?;

    match command {
        Command::Sel(command) => command.run()?,

        Command::Print(command) => command.run()?,
    }

    Ok(())
}
