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
        let created = r.checkout(name, create.is_some())?;

        if created {
            println!("[{label}] Created and checked out '{name}'");
        } else {
            println!("[{label}] Checked out '{name}'");
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

