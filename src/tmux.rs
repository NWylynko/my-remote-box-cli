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

pub fn detach() -> Result<()> {
    if env::var_os("TMUX").is_none() {
        bail!("not inside a tmux session — nothing to exit");
    }
    run(&["detach-client"])
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
