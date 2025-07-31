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
    config::{Config, Selection},
    git,
};
use color_eyre::eyre::{eyre, Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FeatureName(String);

impl FeatureName {
    pub fn new(name: String) -> Self {
        // A simple slugify might be good here in a real-world app
        // to prevent path traversal, but for now this is fine.
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FeatureName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn features_dir() -> Result<PathBuf> {
    let dir = git::repo_root()?.join(".select").join("features");
    if !dir.exists() {
        fs::create_dir_all(&dir).wrap_err("failed to create features directory")?;
    }
    Ok(dir)
}

pub fn feature_path(name: &FeatureName) -> Result<PathBuf> {
    Ok(features_dir()?.join(name.as_str()))
}

fn selection_path(name: &FeatureName) -> Result<PathBuf> {
    Ok(feature_path(name)?.join("selection.toml"))
}

pub fn get_spec_path(name: &FeatureName) -> Result<PathBuf> {
    Ok(feature_path(name)?.join("spec.md"))
}

pub fn create(name: &FeatureName) -> Result<()> {
    let path = feature_path(name)?;
    if path.exists() {
        return Err(eyre!("Feature '{}' already exists.", name));
    }
    fs::create_dir_all(&path)
        .wrap_err_with(|| format!("Failed to create directory for feature '{}'", name))
}

pub fn exists(name: &FeatureName) -> Result<bool> {
    Ok(feature_path(name)?.exists())
}

pub fn delete(name: &FeatureName) -> Result<()> {
    if !exists(name)? {
        return Err(eyre!("Feature '{}' does not exist.", name));
    }
    let path = feature_path(name)?;
    fs::remove_dir_all(&path)
        .wrap_err_with(|| format!("failed to delete feature '{}'", name.as_str()))
}

pub fn list() -> Result<Vec<FeatureName>> {
    let dir = features_dir()?;
    let mut names = vec![];
    if !dir.exists() {
        return Ok(names);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                names.push(FeatureName::new(name.to_owned()));
            }
        }
    }
    names.sort();
    Ok(names)
}

pub fn read_selection(name: &FeatureName) -> Result<Option<Selection>> {
    let path = selection_path(name)?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .wrap_err_with(|| format!("Failed to read selection for feature '{}'", name))?;

    if content.trim().is_empty() {
        return Ok(Some(Selection::default()));
    }

    let config: Config = toml::from_str(&content).wrap_err("Failed to parse selection file")?;
    Ok(config.selection)
}

pub fn write_selection(name: &FeatureName, selection: &Selection) -> Result<()> {
    let path = selection_path(name)?;
    // Create feature directory if it doesn't exist. This can happen if a user
    // tries to write a selection to a feature that was created but whose directory
    // was somehow deleted.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config_to_write = Config {
        selection: Some(selection.clone()),
    };
    let content =
        toml::to_string_pretty(&config_to_write).wrap_err("Failed to serialize selection")?;
    fs::write(path, content)
        .wrap_err_with(|| format!("Failed to write selection for feature '{}'", name))?;
    Ok(())
}

pub fn read_spec(name: &FeatureName) -> Result<Option<String>> {
    let path = get_spec_path(name)?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    Ok(Some(content))
}

pub fn delete_spec(name: &FeatureName) -> Result<()> {
    let path = get_spec_path(name)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
