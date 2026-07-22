use clap_complete::engine::CompletionCandidate;
use std::ffi::OsStr;

use crate::herdr;
use crate::util::project_names;

/// Dynamic completer for arguments that take an existing project (e.g. `box open`).
/// Offers the set up projects under `~`, filtered by what's typed so far.
pub fn complete_projects(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(current, project_names().unwrap_or_default())
}

/// Dynamic completer for arguments that take a running workspace (e.g. `box pause`).
/// Offers only live herdr workspaces, filtered by what's typed so far.
pub fn complete_sessions(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(current, herdr::workspace_names())
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
