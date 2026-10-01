//! `sgit rebase <branch>` - recursive rebase using git CLI.

use crate::RepoTree;
use crate::error::{Result, SgitError};
use crate::git::rebase::RebaseCmd;
use crate::repo::Repo;

/// Rebase every repo's current branch onto `branch` (depth-first).
///
/// - `branch`: local branch name used as the rebase upstream.
pub fn run(branch: &str) -> Result<()> {
    let tree = RepoTree::discover(None)?;
    for r in tree.all() {
        let label = r.label();
        if !r.branch_exists(branch) {
            println!("[{label}] Skipping: branch '{branch}' does not exist");
            continue;
        }
        match rebase_one(r, branch) {
            Ok(()) => println!("[{label}] Rebased onto {branch}"),
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Rebase one repository onto `branch`.
///
/// - `r`: repo handle (used for repo + conflict context).
/// - `branch`: local upstream branch name.
fn rebase_one(r: &Repo, branch: &str) -> Result<()> {
    let cmd = RebaseCmd::new(branch);
    match r.git(&cmd) {
        Ok(_) => Ok(()),
        Err(_) => {
            let _ = r.git(&RebaseCmd::Abort);
            Err(SgitError::Conflict {
                repo: r.display_label(),
                workdir: r.workdir.clone(),
                hint: format!("git rebase {branch}"),
            })
        }
    }
}
