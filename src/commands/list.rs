use anyhow::Result;

use crate::tmux;
use crate::util::project_names;

pub fn list_projects() -> Result<()> {
    let projects = project_names()?;

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
