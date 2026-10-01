//! `sgit branch` - list or create branches across the whole tree.

use crate::RepoTree;
use crate::error::Result;
use crate::git::branch::{self, BranchCmd};

/// List local branches or create one branch across all repos.
///
/// - `create`: optional branch name; `None` lists branches, `Some(name)` creates it.
pub fn run(create: Option<&str>) -> Result<()> {
    let tree = RepoTree::discover(None)?;

    let Some(name) = create else {
        for r in tree.all() {
            println!("\n\x1b[1;34m[{}]\x1b[0m", r.label());
            // let active = r.branch_name();
            let branches = r.list_branches().unwrap_or_default();
            if branches.is_empty() {
                println!("  (no branches yet)");
                continue;
            }
            for bn in &branches {
                let marker = if bn.is_current { "* " } else { "  " };
                println!("{}{}", marker, bn.name);
            }
        }
        return Ok(());
    };

    let cmd = BranchCmd::Create(branch::Create {
        name: name.to_string(),
        ..Default::default()
    });
    for r in tree.all() {
        let label = r.label();

        if r.branch_exists(name) {
            println!("[{label}] Branch '{name}' already exists");
            continue;
        }
        match r.git(&cmd) {
            Ok(_) => println!("[{label}] Created branch '{name}'"),
            Err(e) => {
                eprintln!("[{label}] Error creating branch: {e}");
            }
        }
    }
    Ok(())
}
