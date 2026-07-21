use anyhow::{bail, Context, Result};
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn open_in_tmux(name: &str, dir: &Path) -> Result<()> {
    let dir_str = dir
        .to_str()
        .context("project path contains invalid UTF-8")?;

    if !session_exists(name)? {
        create_session(name, dir_str)?;
    } else {
        eprintln!("tmux session '{name}' already exists — attaching");
    }

    attach(name)
}

/// Name of the tmux session we're currently inside, if any.
pub fn current_session() -> Option<String> {
    if env::var_os("TMUX").is_none() {
        return None;
    }
    let out = Command::new("tmux")
        .args(["display-message", "-p", "#S"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!name.is_empty()).then_some(name)
}

/// Window names for a session, in order (empty if the lookup fails).
pub fn window_names(session: &str) -> Vec<String> {
    Command::new("tmux")
        .args(["list-windows", "-t", &format!("={session}"), "-F", "#W"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub fn detach() -> Result<()> {
    if env::var_os("TMUX").is_none() {
        bail!("not inside a tmux session — nothing to exit");
    }
    run(&["detach-client"])
}

/// Kill a session and everything running in it.
pub fn kill_session(name: &str) -> Result<()> {
    run(&["kill-session", "-t", &format!("={name}")])
}

/// Names of all live sessions (empty if the lookup fails or the server is down).
pub fn session_names() -> Vec<String> {
    Command::new("tmux")
        .args(["list-sessions", "-F", "#S"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub fn session_exists(name: &str) -> Result<bool> {
    let status = Command::new("tmux")
        .args(["has-session", "-t", &format!("={name}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to run tmux (is it installed?)")?;
    Ok(status.success())
}

fn create_session(name: &str, dir: &str) -> Result<()> {
    // Window 0: claude | Window 1: agent | Window 2: remote | Window 3: shell
    run(&["new-session", "-d", "-s", name, "-n", "claude", "-c", dir])?;
    run(&["send-keys", "-t", &format!("{name}:claude"), "claude", "C-m"])?;

    run(&["new-window", "-t", name, "-n", "agent", "-c", dir])?;
    run(&["send-keys", "-t", &format!("{name}:agent"), "agent", "C-m"])?;

    run(&["new-window", "-t", name, "-n", "remote", "-c", dir])?;
    run(&[
        "send-keys",
        "-t",
        &format!("{name}:remote"),
        "claude remote-control",
        "C-m",
    ])?;

    run(&["new-window", "-t", name, "-n", "shell", "-c", dir])?;
    run(&["select-window", "-t", &format!("{name}:shell")])?;
    Ok(())
}

fn attach(name: &str) -> Result<()> {
    let mut cmd = Command::new("tmux");
    if env::var_os("TMUX").is_some() {
        cmd.args(["switch-client", "-t", &format!("={name}")]);
    } else {
        cmd.args(["attach-session", "-t", &format!("={name}")]);
    }

    let status = cmd
        .status()
        .context("failed to attach to tmux session")?;

    if !status.success() {
        bail!("tmux attach failed with {status}");
    }
    Ok(())
}

fn run(args: &[&str]) -> Result<()> {
    let status = Command::new("tmux")
        .args(args)
        .status()
        .with_context(|| format!("failed to run tmux {}", args.join(" ")))?;

    if !status.success() {
        bail!("tmux {} failed with {status}", args.join(" "));
    }
    Ok(())
}
