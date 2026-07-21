use clap_complete::engine::CompletionCandidate;
use std::ffi::OsStr;

use crate::util::project_names;

/// Dynamic completer for arguments that take an existing project (e.g. `box open`).
/// Offers the set up projects under `~`, filtered by what's typed so far.
pub fn complete_projects(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(prefix) = current.to_str() else {
        return Vec::new();
    };

    project_names()
        .unwrap_or_default()
        .into_iter()
        .filter(|name| name.starts_with(prefix))
        .map(CompletionCandidate::new)
        .collect()
}
