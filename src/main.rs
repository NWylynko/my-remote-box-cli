use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(
    name = "box",
    version,
    about = "CLI for managing my remote box",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a project folder, init a git repo on main, and open it in tmux
    New {
        /// Project name (folder + tmux session)
        name: String,
    },
    /// Clone a GitHub repo into ~/<repo> and open it in tmux
    Clone {
        /// GitHub repo (`owner/repo` or a github.com URL)
        repo: String,
    },
    /// Open an existing project folder in tmux
    Open {
        /// Project folder (name under ~, or a path)
        path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => new_project(&name)?,
        Commands::Clone { repo } => clone_project(&repo)?,
        Commands::Open { path } => open_project(&path)?,
    }

    Ok(())
}

fn new_project(name: &str) -> Result<()> {
    validate_name(name)?;

    let dir = home_dir()?.join(name);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create {}", dir.display()))?;

    if !dir.join(".git").is_dir() {
        println!("initializing git repo on main → {}", dir.display());
        let status = Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(&dir)
            .status()
            .context("failed to run git (is it installed?)")?;

        if !status.success() {
            bail!("git init failed with {status}");
        }
    }

    open_in_tmux(name, &dir)
}

fn clone_project(repo: &str) -> Result<()> {
    let (owner, name) = parse_github_repo(repo)?;
    validate_name(&name)?;

    let dir = home_dir()?.join(&name);
    let spec = format!("{owner}/{name}");

    if dir.exists() {
        if dir.join(".git").is_dir() {
            eprintln!("{} already exists — opening session", dir.display());
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

    open_in_tmux(&name, &dir)
}

fn open_project(path: &str) -> Result<()> {
    let dir = resolve_project_dir(path)?;
    if !dir.is_dir() {
        bail!("{} is not a directory", dir.display());
    }

    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .context("project folder has an invalid name")?
        .to_string();
    validate_name(&name)?;

    open_in_tmux(&name, &dir)
}

fn open_in_tmux(name: &str, dir: &Path) -> Result<()> {
    let dir_str = dir
        .to_str()
        .context("project path contains invalid UTF-8")?;

    if !tmux_session_exists(name)? {
        create_tmux_session(name, dir_str)?;
    } else {
        eprintln!("tmux session '{name}' already exists — attaching");
    }

    attach_tmux(name)
}

fn home_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(
        env::var("HOME").context("HOME is not set")?,
    ))
}

fn resolve_project_dir(input: &str) -> Result<PathBuf> {
    let path = if let Some(rest) = input.strip_prefix("~/") {
        home_dir()?.join(rest)
    } else if input == "~" {
        home_dir()?
    } else if input.contains('/') || input == "." || input.starts_with('.') {
        let candidate = PathBuf::from(input);
        if candidate.is_absolute() {
            candidate
        } else {
            env::current_dir()
                .context("failed to get current directory")?
                .join(candidate)
        }
    } else {
        // bare name → ~/name (same as new/clone)
        home_dir()?.join(input)
    };

    if !path.exists() {
        bail!("{} does not exist", path.display());
    }

    fs::canonicalize(&path).with_context(|| format!("failed to resolve {}", path.display()))
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

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("project name cannot be empty");
    }
    if name.contains('/') || name == "." || name == ".." {
        bail!("project name must be a single path segment");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        bail!("project name may only contain letters, numbers, '-' and '_'");
    }
    Ok(())
}

fn tmux_session_exists(name: &str) -> Result<bool> {
    let status = Command::new("tmux")
        .args(["has-session", "-t", &format!("={name}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to run tmux (is it installed?)")?;
    Ok(status.success())
}

fn create_tmux_session(name: &str, dir: &str) -> Result<()> {
    // Window 0: claude | Window 1: agent | Window 2: shell
    run_tmux(&["new-session", "-d", "-s", name, "-n", "claude", "-c", dir])?;
    run_tmux(&["send-keys", "-t", &format!("{name}:claude"), "claude", "C-m"])?;

    run_tmux(&["new-window", "-t", name, "-n", "agent", "-c", dir])?;
    run_tmux(&["send-keys", "-t", &format!("{name}:agent"), "agent", "C-m"])?;

    run_tmux(&["new-window", "-t", name, "-n", "shell", "-c", dir])?;
    run_tmux(&["select-window", "-t", &format!("{name}:shell")])?;
    Ok(())
}

fn attach_tmux(name: &str) -> Result<()> {
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

fn run_tmux(args: &[&str]) -> Result<()> {
    let status = Command::new("tmux")
        .args(args)
        .status()
        .with_context(|| format!("failed to run tmux {}", args.join(" ")))?;

    if !status.success() {
        bail!("tmux {} failed with {status}", args.join(" "));
    }
    Ok(())
}
