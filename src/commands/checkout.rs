//! `sgit checkout` - checkout branches recursively across all repos.

use std::path::Path;

use crate::error::{Result, SgitError};
use crate::git::checkout::CheckoutCmd;
use crate::{Repo, RepoTree};

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
            create_and_checkout(r, name)
        } else {
            checkout_existing(r, name)
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
            Some((repo, rel)) => match checkout_path_from_branch(repo, branch, &rel) {
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
fn checkout_path_from_branch(repo: &Repo, branch: &str, rel: &Path) -> Result<PathCheckoutOutcome> {
    if !repo.branch_exists(branch) {
        return Ok(PathCheckoutOutcome::MissingBranch);
    }

    repo.git(&CheckoutCmd::Restore {
        target: Some(branch.into()),
        paths: vec![rel.to_path_buf()],
    })?;

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
fn checkout_existing(repo: &Repo, name: &str) -> Result<CheckoutOutcome> {
    if !repo.branch_exists(name) {
        return Ok(CheckoutOutcome::Missing);
    }

    repo.git(&CheckoutCmd::Switch {
        target: Some(name.into()),
        create: None,
    })?;

    Ok(CheckoutOutcome::CheckedOut)
}

/// Create `name` from current HEAD when missing, then check it out.
///
/// - `repo`: repository to update.
/// - `name`: local branch name.
fn create_and_checkout(repo: &Repo, name: &str) -> Result<CheckoutOutcome> {
    if repo.branch_exists(name) {
        // Branch already exists: keep command idempotent and just switch to it.
        repo.git(&CheckoutCmd::Switch {
            target: Some(name.into()),
            create: None,
        })?;

        return Ok(CheckoutOutcome::CheckedOut);
    }

    repo.git(&CheckoutCmd::Switch {
        target: None,
        create: Some(name.into()),
    })?;
    Ok(CheckoutOutcome::CreatedAndCheckedOut)
}
