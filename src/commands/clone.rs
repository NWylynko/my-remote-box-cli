use anyhow::{bail, Context, Result};
use std::process::Command;

use crate::herdr;
use crate::util::{home_dir, validate_name};

pub fn clone_project(repo: &str) -> Result<()> {
    let (owner, name) = parse_github_repo(repo)?;
    validate_name(&name)?;

    let dir = home_dir()?.join(&name);
    let spec = format!("{owner}/{name}");

    if dir.exists() {
        if dir.join(".git").is_dir() {
            eprintln!("{} already exists — opening workspace", dir.display());
        } else {
            bail!("{} exists and is not a git repo", dir.display());
        }
    } else {
        let url = format!("https://github.com/{spec}.git");
        println!("cloning {spec} → {}", dir.display());
        let status = Command::new("gh")
            .args([
                "repo",
                "clone",
                &url,
                dir.to_str().context("invalid path")?,
            ])
            .status()
            .context("failed to run gh (is it installed and authenticated?)")?;

        if !status.success() {
            bail!("gh repo clone failed with {status}");
        }
    }

    herdr::open(&name, &dir)
}

fn parse_github_repo(input: &str) -> Result<(String, String)> {
    let input = input.trim().trim_end_matches('/').trim_end_matches(".git");

    let path = if let Some(rest) = input.strip_prefix("git@github.com:") {
        rest
    } else if let Some(rest) = input
        .strip_prefix("https://github.com/")
        .or_else(|| input.strip_prefix("http://github.com/"))
        .or_else(|| input.strip_prefix("ssh://git@github.com/"))
    {
        rest
    } else {
        input
    };

    let mut parts = path.split('/');
    let owner = parts.next().unwrap_or_default();
    let name = parts.next().unwrap_or_default();

    if owner.is_empty() || name.is_empty() || parts.next().is_some() {
        bail!("expected GitHub repo as owner/repo (got '{input}')");
    }

    Ok((owner.to_string(), name.to_string()))
}
