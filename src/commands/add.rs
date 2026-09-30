//! `sgit add` - route paths (from anywhere in the tree) to the right repo.

use crate::RepoTree;
use crate::error::{Result, SgitError};
use crate::git::AddCommand;

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
        let cmd = if all {
            AddCommand::All
        } else {
            AddCommand::Update
        };
        for r in tree.all() {
            let label = r.label();
            match r.git(&cmd) {
                Ok(_) => {
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
            Some((repo, rel)) => {
                let cmd = AddCommand::Paths(vec![rel.clone()]);
                match repo.git(&cmd) {
                    Ok(_) => println!("[{}] Added {}", repo.label(), rel.display()),
                    Err(e) => eprintln!("Error adding {filename}: {e}"),
                }
            }
        }
    }
    Ok(())
}
