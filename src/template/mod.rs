use crate::git;

use std::{fs, path::PathBuf};

use color_eyre::eyre::{Result, WrapErr, eyre};
use minijinja::{Environment, context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TemplateName(String);

impl TemplateName {
    pub fn new(name: String) -> Self {
        // A simple slugify might be good here in a real-world app
        // to prevent path traversal, but for now this is fine.
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TemplateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn template_dir() -> Result<PathBuf> {
    let dir = git::repo_root()?.join(".select").join("templates");
    if !dir.exists() {
        fs::create_dir_all(&dir).wrap_err("failed to create templates directory")?;
    }
    Ok(dir)
}

fn template_path(name: &TemplateName) -> Result<PathBuf> {
    Ok(template_dir()?.join(name.as_str()))
}

pub fn exists(name: &TemplateName) -> Result<bool> {
    Ok(template_path(name)?.exists())
}

pub fn read(name: &TemplateName) -> Result<String> {
    let path = template_path(name)?;
    fs::read_to_string(&path)
        .wrap_err_with(|| format!("failed to read template '{}' from {}", name, path.display()))
}

pub fn delete(name: &TemplateName) -> Result<()> {
    if !exists(name)? {
        return Err(eyre!("Template '{}' does not exist.", name));
    }
    let path = template_path(name)?;
    fs::remove_file(&path).wrap_err_with(|| {
        format!(
            "failed to delete template '{}' from {}",
            name,
            path.display()
        )
    })
}

pub fn list() -> Result<Vec<TemplateName>> {
    let dir = template_dir()?;
    let mut names = vec![];
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                names.push(TemplateName::new(name.to_owned()));
            }
        }
    }
    names.sort();
    Ok(names)
}

pub fn render(name: &TemplateName, args: &[String]) -> Result<String> {
    let content = read(name)?;

    let mut env = Environment::new();
    env.add_template(name.as_str(), &content)
        .wrap_err("Failed to parse template")?;

    let tpl = env
        .get_template(name.as_str())
        .expect("template was just added");

    tpl.render(context! { args => args })
        .wrap_err("Failed to render template")
}

pub fn file_path(name: &TemplateName) -> Result<PathBuf> {
    template_path(name)
}
