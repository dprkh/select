use crate::config::Selection;
use crate::git;

use std::{fs, io, path::PathBuf};

use color_eyre::eyre::{Report, Result, WrapErr};

use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub selection: Option<Selection>,
}

fn file_path() -> Result<PathBuf> {
    let file_path = git::repo_root()?
        //
        .join(".select")
        //
        .join("select.toml");

    Ok(file_path)
}

impl Config {
    pub fn read() -> Result<Self> {
        let file_path = file_path()?;

        let bytes = match fs::read(&file_path) {
            Ok(bytes) => bytes,

            Err(e) => {
                return if let io::ErrorKind::NotFound = e.kind() {
                    Ok(Self::default())
                } else {
                    let message = format!("failed to read config from {}", file_path.display());

                    Err(Report::new(e).wrap_err(message))
                };
            }
        };

        let config = toml::from_slice(&bytes).wrap_err("failed to parse config file")?;

        Ok(config)
    }

    pub fn write(&self) -> Result<()> {
        let string = toml::to_string_pretty(self).wrap_err("failed to serialize config")?;

        let file_path = file_path()?;

        if let Some(parent_path) = file_path.parent() {
            fs::create_dir_all(parent_path)
                //
                .wrap_err_with(|| {
                    format!("failed to create directories for {}", parent_path.display())
                })?;
        }

        fs::write(&file_path, string)
            //
            .wrap_err_with(|| format!("failed to write config to {}", file_path.display()))?;

        Ok(())
    }
}
