use crate::{
    config::{selection::SelectedPath, Config},
    constants::CUSTOM_IGNORE_FILENAME,
    git,
};

use std::{
    env,
    fs::File,
    io::{self, Write},
};

use clap::Args;

use color_eyre::eyre::{eyre, Result, WrapErr};

use ignore::WalkBuilder;

use pathdiff::diff_paths;

#[derive(Args)]
pub struct Print {
    /// Print only the file paths
    #[arg(long)]
    dry_run: bool,
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

        Ok(())
    }
}
