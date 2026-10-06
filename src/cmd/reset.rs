//! `sgit reset` - mixed, hard or soft reset across the whole tree.

use git2::{Repository, ResetType};

use crate::RepoTree;
use crate::cmd::command::{Command, Context};
use crate::error::Result;
use crate::git::resolve_revision;

use clap::Args;

/// Reset HEAD across all repos recursively.
#[derive(Default, Debug, Args)]
pub struct ResetCmd {
    /// Target ref. Defaults to `HEAD`.
    pub target_ref: Option<String>,
    /// Discard all changes and reset the working tree.
    #[arg(long = "hard")]
    pub hard: bool,
}

impl Command for ResetCmd {
    fn run(&self, ctx: &Context) -> Result<()> {
        let target = self.target_ref.as_deref().unwrap_or("HEAD");
        let tree = RepoTree::discover(ctx.workdir.as_deref())?;

        for r in tree.all() {
            let label = r.label();
            let kind = if self.hard {
                ResetType::Hard
            } else {
                ResetType::Mixed
            };
            match reset_to(&r.repo, target, kind) {
                Ok(()) if self.hard => println!("[{label}] Hard reset to {target}"),
                Ok(()) => println!("[{label}] Reset (unstaged all changes)"),
                Err(e) => eprintln!("[{label}] Error resetting: {e}"),
            }
        }
        Ok(())
    }
}

/// Reset a single repository to `target` using the requested reset kind.
///
/// - `repo`: repository to reset.
/// - `target`: revision expression (commit, tag, ref, etc.).
/// - `kind`: reset mode (`Mixed`, `Hard`, ...).
pub(crate) fn reset_to(repo: &Repository, target: &str, kind: ResetType) -> Result<()> {
    let oid = resolve_revision(repo, target)?;
    let obj = repo.find_object(oid, None)?;
    repo.reset(&obj, kind, None)?;
    Ok(())
}
