use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn home_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(
        env::var("HOME").context("HOME is not set")?,
    ))
}

/// Names of set up projects: non-dotfile git repos directly under `~`, sorted.
pub fn project_names() -> Result<Vec<String>> {
    let home = home_dir()?;
    let mut projects: Vec<String> = fs::read_dir(&home)
        .with_context(|| format!("failed to read {}", home.display()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .filter(|name| home.join(name).join(".git").is_dir())
        .collect();
    projects.sort();
    Ok(projects)
}

pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("project name cannot be empty");
    }
    if name.contains('/') || name == "." || name == ".." {
        bail!("project name must be a single path segment");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        bail!("project name may only contain letters, numbers, '-' and '_'");
    }
    Ok(())
}
