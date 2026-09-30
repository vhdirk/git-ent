//! `sgit merge <branch>` - recursive merge, depth-first.
//!
//! For each (sub)module, merge `<branch>` into the current branch using
//! git CLI merge machinery:
//!
//! - fast-forward when possible,
//! - proper merge-commit otherwise,
//! - conflict → abort with a helpful [`SgitError::Conflict`].
//!
//! When a submodule's HEAD advances as a result of the merge, the parent
//! repo automatically records the moved submodule pointer in a follow-up
//! commit (matching sgit's commit-time behavior).

use crate::RepoTree;
use crate::error::{Result, SgitError};
use crate::git::{AddCommand, CommitCommand, MergeAbortCommand, MergeCommand, Repo};
use crate::repo_tree::changed_submodule_paths;

/// Merge `branch` into each repo's current branch, depth-first.
///
/// - `branch`: local branch name to merge from.
pub fn run(branch: &str) -> Result<()> {
    let tree = RepoTree::discover(None)?;

    for r in tree.all() {
        let label = r.label();
        if !r.branch_exists(branch) {
            println!("[{label}] Skipping: branch '{branch}' does not exist");
            continue;
        }
        match merge_one(r, branch) {
            Ok(()) => println!("[{label}] Merged '{branch}'"),
            Err(e) => return Err(e),
        }
        if let Err(e) = record_submodule_updates(r) {
            eprintln!("[{label}] Error recording submodule refs: {e}");
        } else if !changed_submodule_paths(r)?.is_empty() {
            // record_submodule_updates already committed; print once.
            println!("[{label}] Committed updated submodule ref(s)");
        }
    }
    Ok(())
}

/// Merge one repository with fast-forward or merge-commit behavior.
///
/// - `r`: repository handle used for merge and conflict reporting.
/// - `branch`: local branch merged into current HEAD.
fn merge_one(r: &Repo, branch: &str) -> Result<()> {
    let cmd = MergeCommand::new(branch);
    match r.git(&cmd) {
        Ok(()) => Ok(()),
        Err(_) => {
            // Abort the merge to clean up conflict state.
            let _ = r.git(&MergeAbortCommand);
            Err(SgitError::Conflict {
                repo: r.display_label(),
                workdir: r.workdir.clone(),
                hint: format!("git merge {branch}"),
            })
        }
    }
}

/// If any submodule pointers are dirty (moved), stage them and create a
/// follow-up "Update submodule refs" commit on the current branch.
fn record_submodule_updates(r: &Repo) -> Result<()> {
    let changed = changed_submodule_paths(r)?;
    if changed.is_empty() {
        return Ok(());
    }
    r.git(&AddCommand::Paths(
        changed.iter().map(std::path::PathBuf::from).collect(),
    ))?;
    let commit_cmd = CommitCommand {
        message: "Update submodule refs".to_string(),
        no_verify: false,
    };
    r.git(&commit_cmd)?;
    Ok(())
}
