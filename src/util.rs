use anyhow::{bail, Context, Result};
use std::env;
use std::path::PathBuf;

pub fn home_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(
        env::var("HOME").context("HOME is not set")?,
    ))
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
