//! `sgit add` - route paths (from anywhere in the tree) to the right repo.

use std::path::Path;

use git2::{IndexAddOption, Repository};

use crate::RepoTree;
use crate::error::{Result, SgitError};

/// Stage file changes across the repo tree.
///
/// - `filenames`: paths to stage, routed to the deepest owning repo.
/// - `all`: when `true`, behaves like `git add -A` in every repo.
/// - `update`: when `true`, behaves like `git add -u` in every repo.
pub fn run(filenames: &[String], all: bool, update: bool) -> Result<()> {
    if all && update {
        return Err(SgitError::InvalidUsage(
            "-A and -U are mutually exclusive".into(),
        ));
    }

    let tree = RepoTree::discover(None)?;

    if all || update {
        for r in tree.all() {
            let label = r.label();
            let result = if all {
                r.stage_all()
            } else {
                r.stage_update()
            };
            match result {
                Ok(()) => {
                    let (staged, _, _) = r.list_status()?;
                    if !staged.is_empty() {
                        println!("[{label}] Staged {} file(s)", staged.len());
                    }
                }
                Err(e) => eprintln!("[{label}] Error: {e}"),
            }
        }
        return Ok(());
    }

    if filenames.is_empty() {
        return Err(SgitError::InvalidUsage(
            "no files specified (use -A or -U for bulk staging)".into(),
        ));
    }

    for filename in filenames {
        match tree.resolve_file(filename) {
            None => {
                eprintln!("Error: {filename} is not in any known repo/submodule");
            }
            Some((repo, rel)) => match stage_single(&repo.repo, &rel) {
                Ok(()) => println!("[{}] Added {}", repo.label(), rel.display()),
                Err(e) => eprintln!("Error adding {filename}: {e}"),
            },
        }
    }
    Ok(())
}

/// Stage one path - add if it exists, remove if it was deleted.
fn stage_single(repo: &Repository, rel: &Path) -> Result<()> {
    let mut index = repo.index()?;
    let full = repo
        .workdir()
        .map(|w| w.join(rel))
        .unwrap_or_else(|| rel.to_path_buf());
    if full.exists() {
        index.add_path(rel)?;
    } else {
        index.remove_path(rel)?;
    }
    index.write()?;
    Ok(())
}

