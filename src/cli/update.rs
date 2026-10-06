use crate::cli::command::{Cmd, Context};
use clap::Args;
use git2::{FetchOptions, Repository, SubmoduleUpdateOptions};

use crate::RepoTree;
use crate::cli::push::remote_callbacks;
use crate::error::Result;

/// Initialize and update all submodules recursively.
#[derive(Default, Debug, Args)]
pub struct UpdateCmd;

impl Cmd for UpdateCmd {
    /// Initialize and update all submodules recursively for the root repo.
    fn run(&self, ctx: &Context) -> Result<()> {
        let tree = RepoTree::discover(ctx.workdir.as_deref())?;
        let label = tree.root.label();
        match update_recursive(&tree.root.repo) {
            Ok(()) => println!("[{label}] Submodules updated (init + recursive)"),
            Err(e) => eprintln!("[{label}] Error updating submodules: {e}"),
        }
        Ok(())
    }
}

/// Recursively run submodule update for `repo` and each nested submodule.
///
/// - `repo`: repository whose submodule tree is updated.
fn update_recursive(repo: &Repository) -> Result<()> {
    for mut sm in repo.submodules()? {
        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(remote_callbacks());
        let mut opts = SubmoduleUpdateOptions::new();
        opts.fetch(fetch_opts);
        sm.update(true, Some(&mut opts))?;
        if let Ok(sub_repo) = sm.open() {
            update_recursive(&sub_repo)?;
        }
    }
    Ok(())
}
