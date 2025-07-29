use crate::{config::Config, constants::CUSTOM_IGNORE_FILENAME};

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
pub struct Print;

impl Print {
    pub fn run(self) -> Result<()> {
        let Config { selection } = Config::read()?;

        let Some(selection) = selection else {
            return Ok(());
        };

        let mut path_iter = selection.into_inner().into_iter();

        let Some(first_path) = path_iter.next() else {
            return Ok(());
        };

        let mut walk_builder = WalkBuilder::new(first_path);

        for path in path_iter {
            walk_builder.add(path);
        }

        walk_builder.add_custom_ignore_filename(CUSTOM_IGNORE_FILENAME);

        let walk = walk_builder.build();

        let current_dir = env::current_dir().wrap_err("failed to get current dir")?;

        let mut stdout = io::stdout();

        for result in walk {
            let item = result.wrap_err("failed to walk directories")?;

            if let Some(file_type) = item.file_type()
                && file_type.is_file()
            {
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

        Ok(())
    }
}
