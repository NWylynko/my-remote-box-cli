use anyhow::Result;
use std::collections::HashSet;

use crate::herdr;
use crate::util::project_names;

pub fn list_projects() -> Result<()> {
    let projects = project_names()?;

    if projects.is_empty() {
        println!("no projects yet — create one with `box new <name>`");
        return Ok(());
    }

    let active: HashSet<String> = herdr::workspace_names().into_iter().collect();
    for name in projects {
        let marker = if active.contains(&name) { "*" } else { " " };
        println!("{marker} {name}");
    }
    Ok(())
}
