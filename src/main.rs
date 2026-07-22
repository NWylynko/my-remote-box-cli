mod commands;
mod herdr;
mod util;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::engine::ArgValueCompleter;
use clap_complete::CompleteEnv;

use commands::{
    clone_project, complete_projects, complete_sessions, list_projects, new_project, open_project,
    pause_project, show_info,
};

#[derive(Parser, Debug)]
#[command(
    name = "box",
    version,
    about = "CLI for managing my remote box",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all set up projects (default when run with no command)
    #[command(visible_alias = "ls", alias = "l")]
    List,
    /// Create a project folder, init a git repo on main, and open it in herdr
    #[command(alias = "n")]
    New {
        /// Project name (folder + herdr workspace)
        name: String,
    },
    /// Clone a GitHub repo into ~/<repo> and open it in herdr
    #[command(alias = "c")]
    Clone {
        /// GitHub repo (`owner/repo` or a github.com URL)
        repo: String,
    },
    /// Open an existing project folder in herdr
    #[command(alias = "o")]
    Open {
        /// Project folder (name under ~, or a path)
        #[arg(add = ArgValueCompleter::new(complete_projects))]
        path: String,
    },
    /// Show system and project info (adds current-project detail inside a workspace)
    #[command(alias = "i")]
    Info,
    /// Close a project's workspace to free memory (reopen fresh with `box open`)
    #[command(alias = "p")]
    Pause {
        /// Project/workspace to pause (defaults to the current workspace)
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        project: Option<String>,
    },
}

fn main() -> Result<()> {
    // When invoked by the shell's completion hook (COMPLETE env var set), this
    // computes candidates — including dynamic project names — and exits.
    CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::List) => list_projects()?,
        Some(Commands::New { name }) => new_project(&name)?,
        Some(Commands::Clone { repo }) => clone_project(&repo)?,
        Some(Commands::Open { path }) => open_project(&path)?,
        Some(Commands::Info) => show_info()?,
        Some(Commands::Pause { project }) => pause_project(project)?,
    }

    Ok(())
}
