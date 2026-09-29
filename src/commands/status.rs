//! `sgit status` - consolidated recursive status view.

use std::path::{Path, PathBuf};

use colored::Colorize;

use crate::RepoTree;
use crate::error::Result;
use crate::repo_tree::{ChangeKind, StatusEntry};

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

/// Join `prefix` and `path` for display (drops the empty prefix cleanly).
pub(crate) fn prefix_path(prefix: &Path, path: &str) -> String {
    if prefix.as_os_str().is_empty() {
        path.to_string()
    } else {
        prefix.join(path).to_string_lossy().into_owned()
    }
}

/// Build a git-style commit message template (used by `commit` when the
/// user gives no `-m`).
pub(crate) fn build_commit_template(tree: &RepoTree) -> Result<String> {
    let mut lines: Vec<String> = vec![
        "".into(),
        "# Please enter the commit message for your changes. Lines starting".into(),
        "# with '#' will be ignored, and an empty message aborts the commit.".into(),
        "#".into(),
    ];

    let branch_name = tree
        .root
        .branch_name()
        .unwrap_or_else(|| "(detached HEAD)".to_string());
    lines.push(format!("# On branch {branch_name}"));
    lines.push("#".into());

    let mut staged: Vec<StatusEntry> = Vec::new();
    let mut unstaged: Vec<StatusEntry> = Vec::new();
    let mut untracked: Vec<StatusEntry> = Vec::new();

    for r in tree.all() {
        let prefix = if r.prefix == Path::new(".") {
            PathBuf::new()
        } else {
            r.prefix.clone()
        };
        let (s, u, ut) = r.list_status()?;
        for e in s {
            staged.push(StatusEntry {
                kind: e.kind,
                path: prefix_path(&prefix, &e.path),
            });
        }
        for e in u {
            unstaged.push(StatusEntry {
                kind: e.kind,
                path: prefix_path(&prefix, &e.path),
            });
        }
        for e in ut {
            untracked.push(StatusEntry {
                kind: e.kind,
                path: prefix_path(&prefix, &e.path),
            });
        }
    }

    if !staged.is_empty() {
        lines.push("# Changes to be committed:".into());
        for e in &staged {
            lines.push(format!("#\t{}{}", e.kind.label(), e.path));
        }
        lines.push("#".into());
    }
    if !unstaged.is_empty() {
        lines.push("# Changes not staged for commit:".into());
        for e in &unstaged {
            lines.push(format!("#\t{}{}", e.kind.label(), e.path));
        }
        lines.push("#".into());
    }
    if !untracked.is_empty() {
        lines.push("# Untracked files:".into());
        for e in &untracked {
            lines.push(format!("#\t{}", e.path));
        }
        lines.push("#".into());
    }
    Ok(lines.join("\n"))
}
