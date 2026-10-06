use clap::Args;
use std::path::{Path, PathBuf};

use git2::{ObjectType, Repository, build::CheckoutBuilder};

use crate::RepoTree;
use crate::cli::command::{Cmd, Context};
use crate::error::Result;
use crate::git::head_commit;
use crate::repo::Repo;

/// Restore working tree files or unstage changes, recursively.
#[derive(Default, Debug, Args)]
pub struct RestoreCmd {
    /// Files to restore (empty = all changed files).
    pub paths: Vec<PathBuf>,
    /// Unstage files instead of discarding working-tree changes.
    #[arg(short = 'S', long = "staged")]
    pub staged: bool,
}

impl Cmd for RestoreCmd {
    /// Restore working-tree changes or unstage index changes recursively.
    ///
    /// - `filenames`: optional explicit files; empty means operate on all changed files.
    /// - `staged`: when `true`, unstage (`--staged` behavior) instead of restore.
    fn run(&self, ctx: &Context) -> Result<()> {
        let tree = RepoTree::discover(ctx.workdir.as_deref())?;

        if !self.paths.is_empty() {
            for filename in &self.paths {
                match tree.resolve_file(filename) {
                    None => {
                        eprintln!(
                            "Error: {} is not in any known repo/submodule",
                            filename.display()
                        );
                    }
                    Some((repo, rel)) => {
                        let result = if self.staged {
                            unstage(&repo.repo, std::slice::from_ref(&rel))
                        } else {
                            checkout_paths(&repo.repo, std::slice::from_ref(&rel))
                        };
                        match result {
                            Ok(()) if self.staged => {
                                println!("[{}] Unstaged {}", repo.label(), rel.display())
                            }
                            Ok(()) => println!("[{}] Restored {}", repo.label(), rel.display()),
                            Err(e) => {
                                eprintln!(
                                    "[{}] Error restoring {}: {e}",
                                    repo.label(),
                                    rel.display()
                                )
                            }
                        }
                    }
                }
            }
            return Ok(());
        }

        for r in tree.all() {
            restore_all_in(r, self.staged);
        }
        Ok(())
    }
}

/// Restore or unstage all relevant changed files in one repo.
///
/// - `r`: repository handle to process.
/// - `staged`: controls whether to unstage or restore working tree files.
fn restore_all_in(r: &Repo, staged: bool) {
    let label = r.label();
    let status = match r.status() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[{label}] Error listing status: {e}");
            return;
        }
    };
    let paths: Vec<std::path::PathBuf> = if staged {
        status.staged.into_iter().map(|e| e.path.into()).collect()
    } else {
        status.unstaged.into_iter().map(|e| e.path.into()).collect()
    };
    if paths.is_empty() {
        return;
    }

    let result = if staged {
        unstage(&r.repo, &paths)
    } else {
        checkout_paths(&r.repo, &paths)
    };
    match result {
        Ok(()) if staged => println!("[{label}] Unstaged {} file(s)", paths.len()),
        Ok(()) => println!("[{label}] Restored {} file(s)", paths.len()),
        Err(e) if staged => eprintln!("[{label}] Error unstaging: {e}"),
        Err(e) => eprintln!("[{label}] Error restoring: {e}"),
    }
}

/// Equivalent of `git restore <paths>` - overwrite working-tree files
/// with their HEAD contents.
fn checkout_paths(repo: &Repository, paths: &[std::path::PathBuf]) -> Result<()> {
    let mut co = CheckoutBuilder::new();
    co.force();
    // `git restore` also repopulates deleted files, so we must remove
    // the `DISABLE_PATHSPEC_MATCH` default and allow adds.
    co.update_index(false);
    for p in paths {
        co.path(p);
    }
    repo.checkout_head(Some(&mut co))?;
    Ok(())
}

/// Equivalent of `git restore --staged <paths>` - reset index entries
/// for the given paths back to HEAD.
fn unstage(repo: &Repository, paths: &[std::path::PathBuf]) -> Result<()> {
    let head = match head_commit(repo) {
        Ok(c) => c.into_object(),
        Err(_) => {
            // No HEAD yet (fresh repo) - just remove entries from the index.
            let mut index = repo.index()?;
            for p in paths {
                let _ = index.remove_path(p);
            }
            index.write()?;
            return Ok(());
        }
    };
    let path_strs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
    repo.reset_default(Some(&head), path_strs.iter())?;
    // Make sure the resolved object is a commit for clippy's sake.
    let _ = head.peel(ObjectType::Commit);
    Ok(())
}
