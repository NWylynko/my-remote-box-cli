mod commands;
mod tmux;
mod util;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::engine::ArgValueCompleter;
use clap_complete::CompleteEnv;

use commands::{clone_project, complete_projects, list_projects, new_project, open_project};

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
    #[command(visible_alias = "ls")]
    List,
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
        #[arg(add = ArgValueCompleter::new(complete_projects))]
        path: String,
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
    }

    Ok(())
}
