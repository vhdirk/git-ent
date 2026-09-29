//! `sgit checkout` - checkout branches recursively across all repos.

use git2::{BranchType, ErrorCode, Repository, build::CheckoutBuilder};
use std::path::Path;

use crate::RepoTree;
use crate::error::{Result, SgitError};

/// Run recursive checkout behavior.
///
/// - `branch`: branch to switch to, or source branch for path checkout.
/// - `create`: optional branch name to create (`-b`) before switching.
/// - `paths`: optional files to checkout from `branch` when non-empty.
pub fn run(branch: Option<&str>, create: Option<&str>, paths: &[String]) -> Result<()> {
    if !paths.is_empty() {
        if create.is_some() {
            return Err(SgitError::InvalidUsage(
                "usage: sgit checkout <branch> -- <files...> (cannot combine with -b)".into(),
            ));
        }
        let Some(name) = branch else {
            return Err(SgitError::InvalidUsage(
                "usage: sgit checkout <branch> -- <files...>".into(),
            ));
        };
        return checkout_paths_from_branch(name, paths);
    }

    let Some(name) = create.or(branch) else {
        return Err(SgitError::InvalidUsage(
            "usage: sgit checkout <branch> | sgit checkout -b <branch> | sgit checkout <branch> -- <files...>".into(),
        ));
    };

    let tree = RepoTree::discover(None)?;
    for r in tree.all() {
        let label = r.label();
        let res = if create.is_some() {
            create_and_checkout(&r.repo, name)
        } else {
            checkout_existing(&r.repo, name)
        };
        match res {
            Ok(CheckoutOutcome::CheckedOut) => println!("[{label}] Checked out '{name}'"),
            Ok(CheckoutOutcome::CreatedAndCheckedOut) => {
                println!("[{label}] Created and checked out '{name}'")
            }
            Ok(CheckoutOutcome::Missing) => {
                eprintln!("[{label}] Warning: branch '{name}' does not exist")
            }
            Err(e) => eprintln!("[{label}] Error checking out '{name}': {e}"),
        }
    }

    Ok(())
}

/// Checkout one or more files from `branch`, routing each file to its owning repo.
///
/// - `branch`: source branch used for file restoration.
/// - `paths`: user-provided file paths, relative to current working directory.
fn checkout_paths_from_branch(branch: &str, paths: &[String]) -> Result<()> {
    let tree = RepoTree::discover(None)?;
    for filename in paths {
        match tree.resolve_file(filename) {
            None => eprintln!("Error: {filename} is not in any known repo/submodule"),
            Some((repo, rel)) => match checkout_path_from_branch(&repo.repo, branch, &rel) {
                Ok(PathCheckoutOutcome::CheckedOut) => println!(
                    "[{}] Checked out {} from '{}'",
                    repo.label(),
                    rel.display(),
                    branch
                ),
                Ok(PathCheckoutOutcome::MissingBranch) => {
                    eprintln!(
                        "[{}] Warning: branch '{branch}' does not exist",
                        repo.label()
                    )
                }
                Err(e) => eprintln!(
                    "[{}] Error checking out {} from '{}': {e}",
                    repo.label(),
                    rel.display(),
                    branch
                ),
            },
        }
    }
    Ok(())
}

enum PathCheckoutOutcome {
    CheckedOut,
    MissingBranch,
}

/// Restore one path from `branch` into the working tree and index.
///
/// - `repo`: repository that owns the path.
/// - `branch`: source branch name.
/// - `rel`: path relative to `repo` workdir.
fn checkout_path_from_branch(
    repo: &Repository,
    branch: &str,
    rel: &Path,
) -> Result<PathCheckoutOutcome> {
    let br = match repo.find_branch(branch, BranchType::Local) {
        Ok(b) => b,
        Err(e) if e.code() == ErrorCode::NotFound => return Ok(PathCheckoutOutcome::MissingBranch),
        Err(e) => return Err(e.into()),
    };

    let commit = br.get().peel_to_commit()?;
    let tree = commit.tree()?;
    tree.get_path(rel).map_err(|_| {
        SgitError::Other(format!(
            "path '{}' does not exist on branch '{branch}'",
            rel.display()
        ))
    })?;

    let mut co = CheckoutBuilder::new();
    co.force();
    co.update_index(true);
    co.path(rel);
    repo.checkout_tree(tree.as_object(), Some(&mut co))?;
    Ok(PathCheckoutOutcome::CheckedOut)
}

enum CheckoutOutcome {
    CheckedOut,
    CreatedAndCheckedOut,
    Missing,
}

/// Switch to an existing local branch in one repository.
///
/// - `repo`: repository whose HEAD is updated.
/// - `name`: existing local branch name.
fn checkout_existing(repo: &Repository, name: &str) -> Result<CheckoutOutcome> {
    let branch = match repo.find_branch(name, BranchType::Local) {
        Ok(b) => b,
        Err(e) if e.code() == ErrorCode::NotFound => return Ok(CheckoutOutcome::Missing),
        Err(e) => return Err(e.into()),
    };
    let reference = branch.get();
    let refname = reference.name().map_err(SgitError::from)?;

    repo.set_head(refname)?;
    let mut co = CheckoutBuilder::new();
    repo.checkout_head(Some(&mut co))?;
    Ok(CheckoutOutcome::CheckedOut)
}

/// Create `name` from current HEAD when missing, then check it out.
///
/// - `repo`: repository to update.
/// - `name`: local branch name.
fn create_and_checkout(repo: &Repository, name: &str) -> Result<CheckoutOutcome> {
    if let Ok(branch) = repo.find_branch(name, BranchType::Local) {
        // Branch already exists: keep command idempotent and just switch to it.
        let refname = branch.get().name().map_err(SgitError::from)?;
        repo.set_head(refname)?;
        let mut co = CheckoutBuilder::new();
        repo.checkout_head(Some(&mut co))?;
        return Ok(CheckoutOutcome::CheckedOut);
    }

    let head = repo.head()?.peel_to_commit()?;
    let _ = repo.branch(name, &head, false)?;
    let refname = format!("refs/heads/{name}");
    repo.set_head(&refname)?;
    let mut co = CheckoutBuilder::new();
    repo.checkout_head(Some(&mut co))?;
    Ok(CheckoutOutcome::CreatedAndCheckedOut)
}
