//! `git-ent merge <branch>` - recursive merge, depth-first.
//!
//! For each (sub)module, merge `<branch>` into the current branch using
//! libgit2's merge machinery:
//!
//! - fast-forward when possible,
//! - proper merge-commit otherwise,
//! - conflict --> abort with a helpful [`GitEntError::Conflict`].
//!
//! When a submodule's HEAD advances as a result of the merge, the parent
//! repo automatically records the moved submodule pointer in a follow-up
//! commit (matching git-ent's commit-time behaviour).

use clap::Args;
use git2::{
    AnnotatedCommit, BranchType, MergeOptions, Repository, ResetType, build::CheckoutBuilder,
};

use crate::RepoTree;
use crate::cli::command::{Cmd, Context};
use crate::error::{GitEntError, Result};
use crate::git::{head_commit, signature};
use crate::repo::Repo;
use crate::repo_tree::changed_submodule_paths;

/// Merge a branch recursively across all submodules.
#[derive(Default, Debug, Args)]
pub struct MergeCmd {
    /// The branch to merge into the current branch.
    pub branch: String,
}

impl Cmd for MergeCmd {
    fn run(&self, _ctx: &Context) -> Result<()> {
        let tree = RepoTree::discover(None)?;

        for r in tree.all() {
            let label = r.label();
            if !r.has_branch(&self.branch) {
                println!(
                    "[{label}] Skipping: branch '{}' does not exist",
                    &self.branch
                );
                continue;
            }
            match merge_one(r, &self.branch) {
                Ok(()) => println!("[{label}] Merged '{}'", &self.branch),
                Err(e) => return Err(e),
            }
            if let Err(e) = record_submodule_updates(&r.repo) {
                eprintln!("[{label}] Error recording submodule refs: {e}");
            } else if !changed_submodule_paths(&r.repo)?.is_empty() {
                // record_submodule_updates already committed; print once.
                println!("[{label}] Committed updated submodule ref(s)");
            }
        }
        Ok(())
    }
}

/// Merge one repository with fast-forward or merge-commit behavior.
///
/// - `r`: repository handle used for merge and conflict reporting.
/// - `branch`: local branch merged into current HEAD.
fn merge_one(r: &Repo, branch: &str) -> Result<()> {
    let repo = &r.repo;
    let br = repo.find_branch(branch, BranchType::Local)?;
    let target_oid = br
        .get()
        .target()
        .ok_or_else(|| GitEntError::Other(format!("branch '{branch}' has no target")))?;

    let annotated: AnnotatedCommit = repo.find_annotated_commit(target_oid)?;
    let (analysis, _pref) = repo.merge_analysis(&[&annotated])?;

    if analysis.is_up_to_date() {
        return Ok(());
    }

    // Figure out the current branch name (refname).
    let head_ref = repo.head()?;
    let refname = head_ref.name().map_err(GitEntError::from)?.to_string();

    if analysis.is_fast_forward() {
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(target_oid, &format!("fast-forward merge of {branch}"))?;
        repo.set_head(&refname)?;
        let mut co = CheckoutBuilder::new();
        co.force();
        repo.checkout_head(Some(&mut co))?;
        return Ok(());
    }

    // Normal (non-ff) merge.
    let mut merge_opts = MergeOptions::new();
    let mut co = CheckoutBuilder::new();
    co.allow_conflicts(true);
    repo.merge(&[&annotated], Some(&mut merge_opts), Some(&mut co))?;

    let mut index = repo.index()?;
    if index.has_conflicts() {
        // Reset the working tree to HEAD to abort the merge cleanly.
        let head = head_commit(repo)?;
        repo.reset(head.as_object(), ResetType::Hard, None)?;
        repo.cleanup_state()?;
        return Err(GitEntError::Conflict {
            repo: r.display_label(),
            workdir: r.workdir.clone(),
            hint: format!("git merge {branch}"),
        });
    }

    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let sig = signature(repo)?;
    let parent_head = head_commit(repo)?;
    let parent_other = repo.find_commit(target_oid)?;
    let msg = format!("Merge branch '{branch}'");
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &tree,
        &[&parent_head, &parent_other],
    )?;
    repo.cleanup_state()?;
    Ok(())
}

/// If any submodule pointers are dirty (moved), stage them and create a
/// follow-up "Update submodule refs" commit on the current branch.
fn record_submodule_updates(repo: &Repository) -> Result<()> {
    let changed = changed_submodule_paths(repo)?;
    if changed.is_empty() {
        return Ok(());
    }
    let mut index = repo.index()?;
    for p in &changed {
        index.add_path(std::path::Path::new(p))?;
    }
    index.write()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let sig = signature(repo)?;
    let parents = match head_commit(repo) {
        Ok(c) => vec![c],
        Err(_) => Vec::new(),
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Update submodule refs",
        &tree,
        &parent_refs,
    )?;
    Ok(())
}
