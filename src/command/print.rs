use crate::config::{Config, Selection};

use clap::Args;

use color_eyre::eyre::Result;

#[derive(Args)]
pub struct Print;

impl Print {
    pub fn run(self) -> Result<()> {
        let Config { selection } = Config::read()?;

        let Selection(paths) = selection;
    }
}
