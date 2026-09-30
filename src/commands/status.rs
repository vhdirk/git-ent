//! `sgit status` - consolidated recursive status view.

use std::path::{Path, PathBuf};

use colored::Colorize;

use crate::RepoTree;
use crate::error::Result;
use crate::git::ChangeKind;
use crate::repo_tree::prefix_path;

/// Print a consolidated git-style status across all repos/submodules.
pub fn run() -> Result<()> {
    let tree = RepoTree::discover(None)?;

    let mut all_staged: Vec<(ChangeKind, String)> = Vec::new();
    let mut all_unstaged: Vec<(ChangeKind, String)> = Vec::new();
    let mut all_untracked: Vec<String> = Vec::new();

    for r in tree.all() {
        let prefix = if r.prefix == Path::new(".") {
            PathBuf::new()
        } else {
            r.prefix.clone()
        };
        let (staged, unstaged, untracked) = r.list_status()?;
        for s in staged {
            all_staged.push((s.kind, prefix_path(&prefix, &s.path)));
        }
        for u in unstaged {
            all_unstaged.push((u.kind, prefix_path(&prefix, &u.path)));
        }
        for u in untracked {
            all_untracked.push(prefix_path(&prefix, &u.path));
        }
    }

    if all_staged.is_empty() && all_unstaged.is_empty() && all_untracked.is_empty() {
        return Ok(());
    }

    let branch_name = tree
        .root
        .branch_name()
        .unwrap_or_else(|| "(detached HEAD)".to_string());
    println!("On branch {branch_name}");

    if !all_staged.is_empty() {
        println!("\nChanges to be committed:");
        println!("  (use \"sgit restore --staged <file>...\" to unstage)");
        for (k, p) in &all_staged {
            println!("\t{}", format!("{}{}", k.label(), p).green());
        }
    }

    if !all_unstaged.is_empty() {
        println!("\nChanges not staged for commit:");
        println!("  (use \"sgit add <file>...\" to update what will be committed)");
        println!("  (use \"sgit restore <file>...\" to discard changes in working directory)");
        for (k, p) in &all_unstaged {
            println!("\t{}", format!("{}{}", k.label(), p).red());
        }
    }

    if !all_untracked.is_empty() {
        println!("\nUntracked files:");
        println!("  (use \"sgit add <file>...\" to include in what will be committed)");
        for p in &all_untracked {
            println!("\t{}", p.red());
        }
    }

    Ok(())
}
