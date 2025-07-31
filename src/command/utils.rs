use crate::{
    config::{selection::SelectedPath, Config, Selection},
    constants::CUSTOM_IGNORE_FILENAME,
    editor, git,
};
use color_eyre::eyre::{eyre, Result, WrapErr};
use ignore::WalkBuilder;
use pathdiff::diff_paths;
use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

/// Walks through all selected files and calls a closure for each file.
///
/// The closure `on_file` is called with two arguments:
/// 1. The absolute path of the file.
/// 2. The path of the file relative to the current working directory.
pub fn walk_selected_files<F>(selection: &Selection, mut on_file: F) -> Result<()>
where
    F: FnMut(&Path, &Path) -> Result<()>,
{
    let git_root = git::repo_root()?;
    let current_dir = env::current_dir().wrap_err("failed to get current dir")?;

    let selected_paths: Vec<SelectedPath> = selection
        .clone()
        .into_inner()
        .into_iter()
        .map(|p| SelectedPath::new(git_root.join(p.path), p.recursive))
        .collect();

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

            if let Some(file_type) = item.file_type() {
                if file_type.is_file() {
                    let absolute_path = item.path();
                    let relative_path =
                        diff_paths(absolute_path, &current_dir).ok_or_else(|| {
                            eyre!(
                                "failed to construct relative path for {}",
                                absolute_path.display()
                            )
                        })?;

                    on_file(absolute_path, &relative_path)?;
                }
            }
        }
    }

    Ok(())
}

/// Builds a string containing the contents of all selected files,
/// formatted with `<file>` tags.
pub fn get_selected_files_content_as_string(selection: &Selection) -> Result<String> {
    let mut buf = String::new();
    walk_selected_files(selection, |abs_path, rel_path| {
        let content = fs::read_to_string(abs_path)
            .wrap_err_with(|| format!("failed to read file {}", abs_path.display()))?;

        let error_message = "failed to write to buffer";

        write!(&mut buf, "<file path=\"{}\">\n", rel_path.display()).wrap_err(error_message)?;

        buf.push_str(&content);

        write!(&mut buf, "</file>\n").wrap_err(error_message)?;
        Ok(())
    })?;
    Ok(buf)
}

/// Builds a string containing the paths of all selected files, one per line.
pub fn get_selected_files_paths_as_string(selection: &Selection) -> Result<String> {
    let mut buf = String::new();
    walk_selected_files(selection, |_abs_path, rel_path| {
        writeln!(&mut buf, "{}", rel_path.display()).wrap_err("failed to write to buffer")
    })?;
    Ok(buf)
}

pub fn get_global_selection() -> Result<Selection> {
    let config = Config::read()?;
    Ok(config.selection.unwrap_or_default())
}

pub fn interactive_selection(
    roots: &[PathBuf],
    previous_selection: Option<Selection>,
    git_root: &PathBuf,
) -> Result<Selection> {
    // 1. Load existing selections from config, making paths absolute. This is our starting point.
    let mut final_paths: HashMap<PathBuf, bool> = previous_selection
        .unwrap_or_default()
        .into_inner()
        .into_iter()
        .map(|sp| (git_root.join(sp.path), sp.recursive))
        .collect();

    // Keep track of what was in the config to decide which paths are "new" suggestions.
    let originally_selected_paths: HashSet<PathBuf> = final_paths.keys().cloned().collect();

    // 2. Process explicit CLI roots.
    let canonical_roots: Vec<PathBuf> = roots
        .iter()
        .map(|p| {
            p.canonicalize()
                .wrap_err_with(|| format!("Failed to find path {}", p.display()))
        })
        .collect::<Result<_>>()?;

    for path in &canonical_roots {
        // A CLI argument implies a recursive selection, but we don't override
        // an existing non-recursive (`*...`) setting from the config.
        final_paths.entry(path.clone()).or_insert(true);
    }

    // 3. Walk directories from CLI roots to discover NEW sub-directories.
    if let Some(first_root) = canonical_roots.first() {
        let mut walk_builder = WalkBuilder::new(first_root);

        for root in canonical_roots.iter().skip(1) {
            walk_builder.add(root);
        }

        walk_builder.add_custom_ignore_filename(CUSTOM_IGNORE_FILENAME);

        for result in walk_builder.build() {
            let item = result.wrap_err("failed to walk directories")?;

            if let Some(file_type) = item.file_type() {
                if file_type.is_dir() {
                    // For discovered sub-directories, only add them if they are not
                    // already in our selection map. This preserves any `recursive: false`
                    // setting on existing selections.
                    final_paths.entry(item.into_path()).or_insert(true);
                }
            }
        }
    }

    if final_paths.is_empty() {
        return Ok(Selection::default());
    }

    // 4. Prepare the buffer for the editor.
    let all_paths_vec: Vec<_> = final_paths
        .into_iter()
        .map(|(path, recursive)| SelectedPath::new(path, recursive))
        .collect();

    let (mut selected_paths, mut new_suggested_paths): (Vec<_>, Vec<_>) = all_paths_vec
        .into_iter()
        .partition(|p| originally_selected_paths.contains(&p.path));

    selected_paths.sort_unstable();
    new_suggested_paths.sort_unstable();

    let current_dir = env::current_dir().wrap_err("failed to get current dir")?;

    const HEADER: &str = "# Lines starting with '#' are ignored.\n\
                          # To select a path recursively, use its name: path/to/dir\n\
                          # To select a path non-recursively (only files in the directory), prefix with '*': *path/to/dir\n\n";

    let mut buf = String::from(HEADER);

    let to_relative_string = |path_item: &SelectedPath| -> Result<String> {
        let relative_path = diff_paths(&path_item.path, &current_dir).ok_or_else(|| {
            eyre!(
                "failed to construct relative path for {}",
                path_item.path.display()
            )
        })?;
        let path_to_write = SelectedPath::new(relative_path, path_item.recursive);
        Ok(path_to_write.to_string())
    };

    for path_item in &selected_paths {
        writeln!(&mut buf, "{}", to_relative_string(path_item)?).unwrap();
    }

    if !selected_paths.is_empty() && !new_suggested_paths.is_empty() {
        buf.push('\n');
    }

    for path_item in &new_suggested_paths {
        writeln!(&mut buf, "# {}", to_relative_string(path_item)?).unwrap();
    }

    let cursor_line = HEADER.lines().count() + 1;
    let result = editor::get_user_input_from_file_content(&buf, cursor_line, None)?;

    // 5. Parse the user's final selection from the editor buffer.
    let mut paths = HashSet::new();
    let mut errors = Vec::new();

    let result_iter = result
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| {
            let trimmed_line = line.trim();
            let selected_path_relative: SelectedPath = trimmed_line.parse().unwrap();
            fs::canonicalize(&selected_path_relative.path)
                .map(|canonical| SelectedPath::new(canonical, selected_path_relative.recursive))
                .wrap_err_with(|| {
                    format!(
                        "failed to canonicalize {}",
                        selected_path_relative.path.display()
                    )
                })
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
        let relative_paths = paths
            .into_iter()
            .map(|p| {
                diff_paths(&p.path, git_root)
                    .ok_or_else(|| {
                        eyre!("failed to construct relative path for {}", p.path.display())
                    })
                    .map(|relative| SelectedPath::new(relative, p.recursive))
            })
            .collect::<Result<HashSet<_>>>()
            .wrap_err("failed to convert absolute paths to relative paths")?;

        let selection = Selection(relative_paths);

        Ok(selection)
    } else {
        let error = if errors.len() == 1 {
            errors.pop().unwrap()
        } else {
            errors
                .into_iter()
                .fold(eyre!("encountered multiple errors"), |acc, e| {
                    acc.wrap_err(e)
                })
        };

        Err(error)
    }
}
