//! `sgit restore` - restore working-tree files or unstage the index.

use std::path::Path;

use git2::{ObjectType, Repository, build::CheckoutBuilder};

use crate::RepoTree;
use crate::error::Result;
use crate::git::head_commit;
use crate::repo_tree::RepoHandle;

/// Restore working-tree changes or unstage index changes recursively.
///
/// - `filenames`: optional explicit files; empty means operate on all changed files.
/// - `staged`: when `true`, unstage (`--staged` behavior) instead of restore.
pub fn run(filenames: &[String], staged: bool) -> Result<()> {
    let tree = RepoTree::discover(None)?;

    if !filenames.is_empty() {
        for filename in filenames {
            match tree.resolve_file(filename) {
                None => {
                    eprintln!("Error: {filename} is not in any known repo/submodule");
                }
                Some((repo, rel)) => {
                    let result = if staged {
                        unstage(&repo.repo, std::slice::from_ref(&rel))
                    } else {
                        checkout_paths(&repo.repo, std::slice::from_ref(&rel))
                    };
                    match result {
                        Ok(()) if staged => {
                            println!("[{}] Unstaged {}", repo.label(), rel.display())
                        }
                        Ok(()) => println!("[{}] Restored {}", repo.label(), rel.display()),
                        Err(e) => {
                            eprintln!("[{}] Error restoring {}: {e}", repo.label(), rel.display())
                        }
                    }
                }
            }
        }
        return Ok(());
    }

    for r in tree.all() {
        restore_all_in(r, staged);
    }
    Ok(())
}

/// Restore or unstage all relevant changed files in one repo.
///
/// - `r`: repository handle to process.
/// - `staged`: controls whether to unstage or restore working tree files.
fn restore_all_in(r: &RepoHandle, staged: bool) {
    let label = r.label();
    let (st, un, _) = match r.list_status() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[{label}] Error listing status: {e}");
            return;
        }
    };
    let paths: Vec<std::path::PathBuf> = if staged {
        st.into_iter().map(|e| e.path.into()).collect()
    } else {
        un.into_iter().map(|e| e.path.into()).collect()
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
