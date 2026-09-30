//! `sgit restore` - restore working-tree files or unstage the index.

use std::path::PathBuf;

use crate::RepoTree;
use crate::error::Result;
use crate::git::RestoreCommand;
use crate::repo_tree::Repo;

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
                Some((repo, rel)) => match repo.restore(staged, vec![rel.clone()]) {
                    Ok(_) if staged => {
                        println!("[{}] Unstaged {}", repo.label(), rel.display())
                    }
                    Ok(_) => println!("[{}] Restored {}", repo.label(), rel.display()),
                    Err(e) => {
                        eprintln!("[{}] Error restoring {}: {e}", repo.label(), rel.display())
                    }
                },
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
// TODO: make this function (without the printlines) a method of Repo
fn restore_all_in(r: &Repo, staged: bool) {
    let label = r.label();
    let (st, un, _) = match r.list_status() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[{label}] Error listing status: {e}");
            return;
        }
    };
    let paths: Vec<PathBuf> = if staged {
        st.into_iter().map(|e| e.path.into()).collect()
    } else {
        un.into_iter().map(|e| e.path.into()).collect()
    };
    if paths.is_empty() {
        return;
    }

    let cmd = RestoreCommand {
        staged,
        paths: paths.clone(),
    };
    match r.git(&cmd) {
        Ok(_) if staged => println!("[{label}] Unstaged {} file(s)", paths.len()),
        Ok(_) => println!("[{label}] Restored {} file(s)", paths.len()),
        Err(e) if staged => eprintln!("[{label}] Error unstaging: {e}"),
        Err(e) => eprintln!("[{label}] Error restoring: {e}"),
    }
}
