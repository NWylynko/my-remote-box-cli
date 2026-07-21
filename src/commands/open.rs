use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

use crate::tmux;
use crate::util::{home_dir, validate_name};

pub fn open_project(path: &str) -> Result<()> {
    let dir = resolve_project_dir(path)?;
    if !dir.is_dir() {
        bail!("{} is not a directory", dir.display());
    }

    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .context("project folder has an invalid name")?
        .to_string();
    validate_name(&name)?;

    tmux::open_in_tmux(&name, &dir)
}

fn resolve_project_dir(input: &str) -> Result<PathBuf> {
    let path = if let Some(rest) = input.strip_prefix("~/") {
        home_dir()?.join(rest)
    } else if input == "~" {
        home_dir()?
    } else if input.contains('/') || input == "." || input.starts_with('.') {
        let candidate = PathBuf::from(input);
        if candidate.is_absolute() {
            candidate
        } else {
            env::current_dir()
                .context("failed to get current directory")?
                .join(candidate)
        }
    } else {
        // bare name → ~/name (same as new/clone)
        home_dir()?.join(input)
    };

    if !path.exists() {
        bail!("{} does not exist", path.display());
    }

    fs::canonicalize(&path).with_context(|| format!("failed to resolve {}", path.display()))
}
