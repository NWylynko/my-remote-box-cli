use anyhow::{Context, Result};
use std::fs;

use crate::tmux;
use crate::util::home_dir;

pub fn list_projects() -> Result<()> {
    let home = home_dir()?;

    let mut projects: Vec<String> = fs::read_dir(&home)
        .with_context(|| format!("failed to read {}", home.display()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .filter(|name| home.join(name).join(".git").is_dir())
        .collect();
    projects.sort();

    if projects.is_empty() {
        println!("no projects yet — create one with `box new <name>`");
        return Ok(());
    }

    for name in projects {
        let marker = if tmux::session_exists(&name)? { "*" } else { " " };
        println!("{marker} {name}");
    }
    Ok(())
}
