use clap_complete::engine::CompletionCandidate;
use std::ffi::OsStr;

use crate::tmux;
use crate::util::project_names;

/// Dynamic completer for arguments that take an existing project (e.g. `box open`).
/// Offers the set up projects under `~`, filtered by what's typed so far.
pub fn complete_projects(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(current, project_names().unwrap_or_default())
}

/// Dynamic completer for arguments that take a running session (e.g. `box pause`).
/// Offers only live tmux sessions, filtered by what's typed so far.
pub fn complete_sessions(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(current, tmux::session_names())
}

fn candidates(current: &OsStr, names: Vec<String>) -> Vec<CompletionCandidate> {
    let Some(prefix) = current.to_str() else {
        return Vec::new();
    };
    names
        .into_iter()
        .filter(|name| name.starts_with(prefix))
        .map(CompletionCandidate::new)
        .collect()
}
