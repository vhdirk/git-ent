//! The `clap`-derived CLI surface for sgit.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands;
use crate::error::Result;

/// sgit (/ʃɪt/) - manage projects with (nested) git submodules.
#[derive(Debug, Parser)]
#[command(name = "sgit", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Clone a repository recursively (including all submodules).
    Clone {
        /// The repository URL.
        url: String,
        /// Destination directory (inferred from URL if omitted).
        dest: Option<PathBuf>,
    },

    /// Initialize and update all submodules recursively.
    Update,

    /// Show status recursively across all submodules.
    Status,

    /// List or create branches across the whole tree.
    Branch {
        /// Name of a branch to create in every repo.
        #[arg(short = 'c', long = "create")]
        create: Option<String>,
    },

    /// Checkout a branch recursively across all repos.
    Checkout {
        /// Create a new branch in every repo and check it out.
        #[arg(short = 'b', conflicts_with = "branch")]
        create: Option<String>,
        /// Existing branch to check out where it exists.
        branch: Option<String>,
        /// Files to restore from <branch> (use `--` before the first path).
        #[arg(last = true)]
        paths: Vec<String>,
    },

    /// Switch branches recursively across all repos.
    Switch {
        /// Create and switch to a new branch in every repo.
        #[arg(short = 'c', long = "create", conflicts_with_all = ["force_create", "detach"])]
        create: Option<String>,
        /// Reset or create the branch, then switch to it in every repo.
        #[arg(short = 'C', long = "force-create", conflicts_with_all = ["create", "detach"])]
        force_create: Option<String>,
        /// Switch to a detached HEAD at the target commit-ish.
        #[arg(long = "detach", conflicts_with_all = ["create", "force_create"])]
        detach: bool,
        /// Existing branch to switch to, or start-point for `-c`/`-C`.
        target: Option<String>,
    },

    /// Add files to the git index (paths can be from any submodule).
    Add {
        /// Files to add.
        filenames: Vec<String>,
        /// Stage all changes (modified, deleted, untracked) everywhere.
        #[arg(short = 'A', long = "all")]
        all: bool,
        /// Stage tracked-file changes only (skip untracked) everywhere.
        #[arg(short = 'u', long = "update")]
        update: bool,
    },

    /// Commit across all (sub)modules that have staged changes.
    Commit {
        /// Commit message. Opens `$EDITOR` if omitted.
        #[arg(short = 'm', long = "message")]
        message: Option<String>,
        /// Skip pre-commit and commit-msg hooks.
        #[arg(long = "no-verify")]
        no_verify: bool,
    },

    /// Push to remote across all repos that have commits to push.
    Push {
        /// Push options (`-o`). Can be specified multiple times.
        #[arg(short = 'o', long = "push-option")]
        push_option: Vec<String>,
    },

    /// Restore working tree files or unstage changes, recursively.
    Restore {
        /// Files to restore (empty = all changed files).
        filenames: Vec<String>,
        /// Unstage files instead of discarding working-tree changes.
        #[arg(short = 'S', long = "staged")]
        staged: bool,
    },

    /// Reset HEAD across all repos recursively.
    Reset {
        /// Target ref. Defaults to `HEAD`.
        ref_: Option<String>,
        /// Discard all changes and reset the working tree.
        #[arg(long = "hard")]
        hard: bool,
    },

    /// Merge a branch recursively across all submodules.
    Merge {
        /// The branch to merge into the current branch.
        branch: String,
    },

    /// Rebase all branches recursively onto the given branch.
    Rebase {
        /// The branch to rebase onto.
        branch: String,
    },

    /// Squash all commits since the common ancestor with <branch>.
    ///
    /// Mirrors GitHub/GitLab's "squash and merge" semantics but depth-first
    /// across submodules: each submodule is squashed first, then the
    /// updated submodule pointer is rolled into the parent's squash commit.
    Squash {
        /// The base branch whose merge-base defines the squash range.
        branch: String,
        /// Commit message for the squash. Defaults to the concatenated
        /// subjects of the squashed commits.
        #[arg(short = 'm', long = "message")]
        message: Option<String>,
    },
}

/// Dispatch a parsed [`Cli`] to the appropriate command handler.
pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Clone { url, dest } => commands::clone::run(&url, dest.as_deref()),
        // Command::Update => commands::update::run(),
        Command::Status => commands::status::run(),
        Command::Branch { create } => commands::branch::run(create.as_deref()),
        Command::Checkout {
            create,
            branch,
            paths,
        } => commands::checkout::run(branch.as_deref(), create.as_deref(), &paths),
        // Command::Switch {
        //     create,
        //     force_create,
        //     detach,
        //     target,
        // } => commands::switch::run(
        //     target.as_deref(),
        //     create.as_deref(),
        //     force_create.as_deref(),
        //     detach,
        // ),
        Command::Add {
            filenames,
            all,
            update,
        } => commands::add::run(&filenames, all, update),
        Command::Commit { message, no_verify } => {
            commands::commit::run(message.as_deref(), no_verify)
        }
        Command::Push { push_option } => commands::push::run(&push_option),
        // Command::Restore { filenames, staged } => commands::restore::run(&filenames, staged),
        // Command::Reset { ref_, hard } => commands::reset::run(ref_.as_deref(), hard),
        // Command::Merge { branch } => commands::merge::run(&branch),
        // Command::Rebase { branch } => commands::rebase::run(&branch),
        // Command::Squash { branch, message } => commands::squash::run(&branch, message.as_deref()),
        _ => {
            unimplemented!("This command is not yet implemented")
        }
    }
}
