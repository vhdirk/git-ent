//! `sgit branch` - list or create branches across the whole tree.

use git2::BranchType;

use crate::RepoTree;
use crate::error::Result;

/// List local branches or create one branch across all repos.
///
/// - `create`: optional branch name; `None` lists branches, `Some(name)` creates it.
pub fn run(create: Option<&str>) -> Result<()> {
    let tree = RepoTree::discover(None)?;

    let Some(name) = create else {
        for r in tree.all() {
            println!("\n\x1b[1;34m[{}]\x1b[0m", r.label());
            let active = r.branch_name();
            let branches = match r.repo.branches(Some(BranchType::Local)) {
                Ok(it) => it,
                Err(_) => {
                    println!("  (no branches yet)");
                    continue;
                }
            };
            let mut any = false;
            for b in branches.flatten() {
                if let Ok(Some(bn)) = b.0.name() {
                    any = true;
                    let marker = if Some(bn.to_string()) == active {
                        "* "
                    } else {
                        "  "
                    };
                    println!("{marker}{bn}");
                }
            }
            if !any {
                println!("  (no branches yet)");
            }
        }
        return Ok(());
    };

    for r in tree.all() {
        let label = r.label();
        // `idempotent create` matches the Python behaviour (duplicate OK).
        let head = match r.repo.head() {
            Ok(h) => h,
            Err(e) => {
                eprintln!("[{label}] Error creating branch: {e}");
                continue;
            }
        };
        let commit = match head.peel_to_commit() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[{label}] Error creating branch: {e}");
                continue;
            }
        };
        match r.repo.branch(name, &commit, false) {
            Ok(_) => println!("[{label}] Created branch '{name}'"),
            Err(e) if e.code() == git2::ErrorCode::Exists => {
                // Idempotent: report as created for parity with Python.
                println!("[{label}] Created branch '{name}'");
            }
            Err(e) => eprintln!("[{label}] Error creating branch: {e}"),
        }
    }
    Ok(())
}
