//! `sgit rebase <branch>` - recursive rebase using libgit2.

use clap::Args;
use git2::{BranchType, Rebase, RebaseOptions, Repository};

use crate::RepoTree;
use crate::cmd::command::{Cmd, Context};
use crate::error::{Result, SgitError};
use crate::git::signature;
use crate::repo::Repo;

/// Rebase all branches recursively onto the given branch.
#[derive(Default, Debug, Args)]
pub struct RebaseCmd {
    /// The branch to rebase onto.
    pub branch: String,
}

impl Cmd for RebaseCmd {
    /// Rebase every repo's current branch onto `branch` (depth-first).
    ///
    /// - `branch`: local branch name used as the rebase upstream.
    fn run(&self, ctx: &Context) -> Result<()> {
        let tree = RepoTree::discover(ctx.workdir.as_deref())?;
        for r in tree.all() {
            let label = r.label();
            if repo_has_branch(&r.repo, &self.branch).is_err() {
                println!(
                    "[{label}] Skipping: branch '{}'' does not exist",
                    &self.branch
                );
                continue;
            }
            match rebase_one(r, &self.branch) {
                Ok(()) => println!("[{label}] Rebased onto {}", &self.branch),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

/// Validate that `branch` exists locally in `repo`.
///
/// - `repo`: repository to validate.
/// - `branch`: local branch name that must exist.
fn repo_has_branch(repo: &Repository, branch: &str) -> Result<()> {
    repo.find_branch(branch, BranchType::Local)?;
    Ok(())
}

/// Rebase one repository onto `branch`.
///
/// - `r`: repo handle (used for repo + conflict context).
/// - `branch`: local upstream branch name.
fn rebase_one(r: &Repo, branch: &str) -> Result<()> {
    let repo = &r.repo;

    // Upstream (what we're rebasing onto).
    let upstream_oid = repo
        .find_branch(branch, BranchType::Local)?
        .get()
        .target()
        .ok_or_else(|| SgitError::Other(format!("branch '{branch}' has no target")))?;
    let upstream = repo.find_annotated_commit(upstream_oid)?;

    // "branch" (what we're rebasing) = current HEAD.
    let head_oid = repo
        .head()?
        .target()
        .ok_or_else(|| SgitError::Other("HEAD has no target".into()))?;
    let head_annotated = repo.find_annotated_commit(head_oid)?;

    let mut opts = RebaseOptions::new();
    let mut rebase: Rebase<'_> = repo.rebase(
        Some(&head_annotated),
        Some(&upstream),
        None,
        Some(&mut opts),
    )?;

    let sig = signature(repo)?;
    while let Some(step) = rebase.next() {
        let _op = step?;
        if repo.index()?.has_conflicts() {
            let _ = rebase.abort();
            return Err(SgitError::Conflict {
                repo: r.display_label(),
                workdir: r.workdir.clone(),
                hint: format!("git rebase {branch}"),
            });
        }
        // Commit this replayed commit; if it was a no-op (already applied)
        // libgit2 returns Error with class == Applied - skip in that case.
        match rebase.commit(None, &sig, None) {
            Ok(_) => {}
            Err(e) if e.class() == git2::ErrorClass::Rebase => {}
            Err(e) => {
                let _ = rebase.abort();
                return Err(e.into());
            }
        }
    }
    rebase.finish(Some(&sig))?;
    Ok(())
}
