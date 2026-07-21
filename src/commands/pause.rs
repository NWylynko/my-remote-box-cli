use anyhow::{bail, Context, Result};

use crate::tmux;
use crate::util::validate_name;

/// Kill a project's tmux session (and everything in it — claude, agent, pnpm
/// dev, …) to free memory. Reopen fresh later with `box open <name>`.
pub fn pause_project(project: Option<String>) -> Result<()> {
    let name = match project {
        Some(name) => name,
        None => tmux::current_session()
            .context("no project given and not inside a session")?,
    };
    validate_name(&name)?;

    if !tmux::session_exists(&name)? {
        bail!("no active session for '{name}' — nothing to pause");
    }

    // Note: when pausing the session you're attached to, this also drops you out
    // of it (the client's session is gone), so the line below may not be seen.
    println!("pausing '{name}' — reopen with `box open {name}`");
    tmux::kill_session(&name)
}
