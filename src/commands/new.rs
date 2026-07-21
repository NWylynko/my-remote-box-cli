use anyhow::{bail, Context, Result};
use std::fs;
use std::process::Command;

use crate::tmux;
use crate::util::{home_dir, validate_name};

pub fn new_project(name: &str) -> Result<()> {
    validate_name(name)?;

    let dir = home_dir()?.join(name);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create {}", dir.display()))?;

    if !dir.join(".git").is_dir() {
        println!("initializing git repo on main → {}", dir.display());
        let status = Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(&dir)
            .status()
            .context("failed to run git (is it installed?)")?;

        if !status.success() {
            bail!("git init failed with {status}");
        }
    }

    tmux::open_in_tmux(name, &dir)
}
