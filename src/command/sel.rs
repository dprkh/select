use crate::{
    config::{Config, Selection},
    constants::CUSTOM_IGNORE_FILENAME,
};

use std::{collections::HashSet, env, fmt::Write, fs, path::PathBuf, process::Command};

use color_eyre::eyre::{eyre, Result, WrapErr};

use clap::Args;

use ignore::WalkBuilder;

use pathdiff::diff_paths;

use tempfile::NamedTempFile;

#[derive(Args)]
pub struct Sel {
    roots: Vec<PathBuf>,
}

impl Sel {
    pub fn run(self) -> Result<()> {
        let mut config = Config::read()?;

        let selection = self.select(config.selection.take())?;

        let selection_len = selection.0.len();

        config.selection.replace(selection);

        config.write()?;

        println!("{selection_len} paths selected");

        Ok(())
    }

    fn select(&self, previous_selection: Option<Selection>) -> Result<Selection> {
        let Self { roots } = self;

        let selected_paths = previous_selection.unwrap_or_default().0;
        let mut all_paths = selected_paths.clone();

        if let Some(first_root) = roots.first() {
            let mut walk_builder = WalkBuilder::new(first_root);

            for root in roots.iter().skip(1) {
                walk_builder.add(root);
            }

            walk_builder.add_custom_ignore_filename(CUSTOM_IGNORE_FILENAME);

            let walk = walk_builder.build();

            for result in walk {
                let item = result.wrap_err("failed to walk directories")?;

                if let Some(file_type) = item.file_type()
                    && file_type.is_dir()
                {
                    let path = item.into_path();

                    let canonical_path = path
                        //
                        .canonicalize()
                        //
                        .wrap_err_with(|| format!("failed to canonicalize {}", path.display()))?;

                    all_paths.insert(canonical_path);
                }
            }
        }

        if all_paths.is_empty() {
            return Ok(Selection::default());
        }

        let mut all_paths_vec: Vec<_> = all_paths.into_iter().collect();
        all_paths_vec.sort_unstable();
        all_paths_vec.dedup();

        let current_dir = env::current_dir().wrap_err("failed to get current dir")?;

        let mut buf = String::new();

        for path in &all_paths_vec {
            let relative_path = diff_paths(path, &current_dir)
                //
                .ok_or_else(|| eyre!("failed to construct relative path for {}", path.display()))?;

            if !selected_paths.contains(path) {
                buf.push_str("# ");
            }

            write!(&mut buf, "{}", relative_path.display()).unwrap();

            buf.push('\n');
        }

        let file = NamedTempFile::new().wrap_err("failed to create a temporary file")?;

        fs::write(file.path(), buf).wrap_err("failed to write to temporary file")?;

        let _ = Command::new("vim")
            //
            .arg(file.path())
            //
            .spawn()
            //
            .wrap_err("failed to start editor")?
            //
            .wait()
            //
            .wrap_err("failed to wait for editor")?;

        let result = fs::read_to_string(file.path())
            //
            .wrap_err("failed to read temporary file")?;

        drop(file);

        let mut paths = HashSet::new();

        let mut errors = Vec::new();

        let result_iter = result
            //
            .lines()
            //
            .filter(|line| !line.starts_with('#'))
            //
            .map(|line| {
                let trimmed_line = line.trim();

                fs::canonicalize(trimmed_line)
                    //
                    .wrap_err_with(|| format!("failed to canonicalize {trimmed_line}"))
            });

        for result in result_iter {
            match result {
                Ok(path) => {
                    paths.insert(path);
                }

                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            let selection = Selection(paths);

            Ok(selection)
        } else {
            let error = if errors.len() == 1 {
                errors.pop().unwrap()
            } else {
                errors.pop().unwrap()
                // # doesn't work
                //  errors
                //      //
                //      .into_iter()
                //      //
                //      .fold(eyre!("encountered multiple errors"), |report, e| {
                //          report.error(e)
                //      })
            };

            Err(error)
        }
    }
}
