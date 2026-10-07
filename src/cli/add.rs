//! `sgit add` - route paths (from anywhere in the tree) to the right repo.

use clap::Args;
use std::path::PathBuf;

use crate::RepoTree;
use crate::cli::command::{Cmd, Context};
use crate::error::Result;

/// Add files to the git index (paths can be from any submodule).
#[derive(Default, Debug, Args)]
pub struct AddCmd {
    /// Files to add.
    #[arg(required_unless_present_any = ["all","update"])]
    pub paths: Vec<PathBuf>,

    /// Stage all changes (modified, deleted, untracked) everywhere.
    #[arg(short = 'A', long = "all")]
    pub all: bool,

    /// Stage tracked-file changes only (skip untracked) everywhere.
    #[arg(short = 'u', long = "update", conflicts_with = "all")]
    pub update: bool,
}

impl Cmd for AddCmd {
    /// Stage file changes across the repo tree.
    ///
    /// - `filenames`: paths to stage, routed to the deepest owning repo.
    /// - `all`: when `true`, behaves like `git add -A` in every repo.
    /// - `update`: when `true`, behaves like `git add -u` in every repo.
    fn run(&self, _ctx: &Context) -> Result<()> {
        let tree = RepoTree::discover(None)?;

        if self.all || self.update {
            for r in tree.all() {
                let label = r.label();
                let result = if self.all {
                    r.stage_all()
                } else {
                    r.stage_update()
                };
                match result {
                    Ok(()) => {
                        let status = r.status()?;
                        if !status.staged.is_empty() {
                            println!("[{label}] Staged {} file(s)", status.staged.len());
                        }
                    }
                    Err(e) => eprintln!("[{label}] Error: {e}"),
                }
            }
            return Ok(());
        }

        for filename in &self.paths {
            match tree.resolve_file(filename) {
                None => {
                    eprintln!(
                        "Error: {} is not in any known repo/submodule",
                        filename.display()
                    );
                }
                Some((repo, rel)) => match repo.stage(&rel) {
                    Ok(()) => println!("[{}] Added {}", repo.label(), rel.display()),
                    Err(e) => eprintln!("Error adding {}: {}", filename.display(), e),
                },
            }
        }
        Ok(())
    }
}
