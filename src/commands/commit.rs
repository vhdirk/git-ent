//! `sgit commit` - depth-first commit with automatic submodule-pointer staging.

use git2::Repository;

use crate::RepoTree;
use crate::commands::status::build_commit_template;
use crate::error::{Result, SgitError};
use crate::git::{head_commit, signature};

/// Commit staged changes across all repos in depth-first order.
///
/// - `message`: optional commit message; when absent, opens an editor.
/// - `no_verify`: accepted for CLI parity (hooks are not run by libgit2).
pub fn run(message: Option<&str>, no_verify: bool) -> Result<()> {
    // `no_verify` is accepted for CLI parity with `git commit`, but
    // libgit2 never invokes pre-commit/commit-msg hooks anyway - so
    // every commit made by sgit is implicitly "no-verify".
    let _ = no_verify;

    let tree = RepoTree::discover(None)?;
    let owned: String;
    let msg: &str = match message {
        Some(m) => m,
        None => {
            let template = build_commit_template(&tree)?;
            match commit_message_from_editor(&template) {
                Some(m) => {
                    owned = m;
                    &owned
                }
                None => return Err(SgitError::EmptyCommitMessage),
            }
        }
    };

    let mut committed_any = false;

    for r in tree.all() {
        let label = r.label();

        // Auto-stage moved submodule pointers in parents.
        if r.repo.submodules().map(|v| !v.is_empty()).unwrap_or(false) {
            if let Err(e) = tree.stage_submodule_pointers(r) {
                eprintln!("[{label}] Error staging submodule refs: {e}");
            }
        }

        let (staged, _, _) = r.list_status()?;
        if staged.is_empty() {
            continue;
        }

        match create_commit(&r.repo, msg) {
            Ok(()) => {
                println!("[{label}] Committed: {msg}");
                committed_any = true;
            }
            Err(e) => eprintln!("[{label}] Error committing: {e}"),
        }
    }

    if !committed_any {
        println!("Nothing to commit.");
    }
    Ok(())
}

/// Create a commit on the current branch from the repo's index.
fn create_commit(repo: &Repository, message: &str) -> Result<()> {
    let sig = signature(repo)?;
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    // Collect parents: HEAD if any.
    let parents = match head_commit(repo) {
        Ok(c) => vec![c],
        Err(_) => Vec::new(),
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)?;
    Ok(())
}

/// Open `$EDITOR`/`$VISUAL` with `template` pre-filled and return the
/// non-comment, trimmed body - or `None` if the user saved an empty
/// message (= abort, matching `git commit`).
fn commit_message_from_editor(template: &str) -> Option<String> {
    let edited = match edit::edit(template) {
        Ok(s) => s,
        Err(_) => return None,
    };
    let body = edited
        .lines()
        .filter(|l| !l.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let trimmed = body.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}
