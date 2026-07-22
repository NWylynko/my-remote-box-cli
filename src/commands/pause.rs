use anyhow::{bail, Context, Result};

use crate::herdr;
use crate::util::validate_name;

/// Close a project's herdr workspace (and everything in it — claude, agent, pnpm
/// dev, …) to free memory. Reopen fresh later with `box open <name>`.
pub fn pause_project(project: Option<String>) -> Result<()> {
    let name = match project {
        Some(name) => name,
        None => herdr::current_workspace()
            .context("no project given and not inside a workspace")?,
    };
    validate_name(&name)?;

    if !herdr::workspace_exists(&name)? {
        bail!("no active workspace for '{name}' — nothing to pause");
    }

    // Note: when pausing the workspace you're focused on, herdr drops you out of
    // it once it's closed, so the line below may not be seen.
    println!("pausing '{name}' — reopen with `box open {name}`");
    herdr::close_workspace(&name)
}
