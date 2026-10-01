//! `sgit commit` - depth-first commit with automatic submodule-pointer staging.

use crate::RepoTree;
use crate::error::{Result, SgitError};
use crate::git::commit::CommitCmd;

/// Commit staged changes across all repos in depth-first order.
///
/// - `message`: optional commit message; when absent, opens an editor.
/// - `no_verify`: skip the `pre-commit` and `commit-msg` hooks.
pub fn run(message: Option<&str>, no_verify: bool) -> Result<()> {
    let tree = RepoTree::discover(None)?;
    let owned: String;
    let msg: &str = match message {
        Some(m) => m,
        None => {
            let template = tree.build_commit_template()?;
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
    let mut commit_error = None;

    for r in tree.all() {
        let label = r.label();

        // Auto-stage moved submodule pointers in parents.
        if r.workdir.join(".gitmodules").exists() {
            if let Err(e) = tree.stage_submodule_pointers(r) {
                eprintln!("[{label}] Error staging submodule refs: {e}");
            }
        }

        let s = r.status()?;
        if s.is_empty() {
            continue;
        }

        let cmd = CommitCmd {
            message: Some(msg.into()),
            no_verify,
            ..Default::default()
        };

        match r.git(&cmd) {
            // TODO: print commit hash
            Ok(_) => {
                println!("[{label}] Committed: {msg}");
                committed_any = true;
            }
            Err(e) => {
                eprintln!("[{label}] Error committing: {e}");
                if commit_error.is_none() {
                    commit_error = Some(e);
                }
            }
        }
    }

    if !committed_any && commit_error.is_none() {
        println!("Nothing to commit.");
    }
    if let Some(error) = commit_error {
        return Err(error);
    }
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
