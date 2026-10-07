use crate::RepoTree;
use crate::cli::command::{Cmd, Context};
use crate::error::{GitEntError, Result};
use clap::Args;
use git2::{BranchType, ErrorCode, Repository, build::CheckoutBuilder};
use std::path::Path;
use std::path::PathBuf;

/// Checkout a branch recursively across all repos.
#[derive(Default, Debug, Args)]
pub struct CheckoutCmd {
    /// Create a new branch in every repo and check it out.
    #[arg(short = 'b', conflicts_with = "branch")]
    pub create: Option<String>,

    /// Existing branch to check out where it exists.
    pub branch: Option<String>,

    /// Files to restore from <branch> (use `--` before the first path).
    #[arg(last = true, conflicts_with = "create")]
    pub paths: Vec<PathBuf>,
}

impl Cmd for CheckoutCmd {
    /// Run recursive checkout behavior.
    ///
    /// - `branch`: branch to switch to, or source branch for path checkout.
    /// - `create`: optional branch name to create (`-b`) before switching.
    /// - `paths`: optional files to checkout from `branch` when non-empty.
    fn run(&self, _ctx: &Context) -> Result<()> {
        if !self.paths.is_empty() {
            let Some(name) = &self.branch else {
                return Err(GitEntError::InvalidUsage(
                    "usage: git-ent checkout <branch> -- <files...>".into(),
                ));
            };
            return checkout_paths_from_branch(None, name, &self.paths);
        }

        let Some(name) = self.create.as_deref().or(self.branch.as_deref()) else {
            return Err(GitEntError::InvalidUsage(
                "usage: git-ent checkout <branch> | git-ent checkout -b <branch> | git-ent checkout <branch> -- <files...>".into(),
            ));
        };

        let tree = RepoTree::discover(None)?;
        for r in tree.all() {
            let label = r.label();
            let created = r.checkout(name, self.create.is_some())?;

            if created {
                println!("[{label}] Created and checked out '{name}'");
            } else {
                println!("[{label}] Checked out '{name}'");
            }
        }

        Ok(())
    }
}

/// Checkout one or more files from `branch`, routing each file to its owning repo.
///
/// - `branch`: source branch used for file restoration.
/// - `paths`: user-provided file paths, relative to current working directory.
fn checkout_paths_from_branch(
    workdir: Option<&Path>,
    branch: &str,
    paths: &[PathBuf],
) -> Result<()> {
    let tree = RepoTree::discover(workdir)?;
    for filename in paths {
        match tree.resolve_file(filename) {
            None => eprintln!(
                "Error: {} is not in any known repo/submodule",
                filename.display()
            ),
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
        GitEntError::Other(format!(
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
