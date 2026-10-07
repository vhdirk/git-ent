use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::RepoTree;
use crate::cli::command::{Cmd, Context};
use crate::error::{GitEntError, Result};
use clap::Args;
use tempfile::NamedTempFile;

/// Commit across all (sub)modules that have staged changes.
#[derive(Default, Debug, Args)]
pub struct CommitCmd {
    /// Commit message. Opens `$EDITOR` if omitted.
    #[arg(short = 'm', long = "message")]
    pub message: Option<String>,
    /// Skip pre-commit and commit-msg hooks.
    #[arg(long = "no-verify")]
    pub no_verify: bool,
}

impl Cmd for CommitCmd {
    /// Commit staged changes across all repos in depth-first order.
    fn run(&self, _ctx: &Context) -> Result<()> {
        let tree = RepoTree::discover(None)?;
        let owned: String;
        let msg: &str = match self.message.as_deref() {
            Some(m) => m,
            None => {
                let template = tree.build_commit_template()?;
                match commit_message_from_editor(&template) {
                    Some(m) => {
                        owned = m;
                        &owned
                    }
                    None => return Err(GitEntError::EmptyCommitMessage),
                }
            }
        };

        let mut committed_any = false;
        let mut hook_error = None;

        for r in tree.all() {
            let label = r.label();

            // Auto-stage moved submodule pointers in parents.
            if r.repo.submodules().map(|v| !v.is_empty()).unwrap_or(false) {
                if let Err(e) = tree.stage_submodule_pointers(r) {
                    eprintln!("[{label}] Error staging submodule refs: {e}");
                }
            }

            let status = r.status()?;
            if status.staged.is_empty() {
                continue;
            }

            let commit_message = if self.no_verify {
                msg.to_string()
            } else {
                match r.run_commit_hooks(msg) {
                    Ok(message) => message,
                    Err(e) => {
                        eprintln!("[{label}] Error committing: {e}");
                        if hook_error.is_none() {
                            hook_error = Some(e);
                        }
                        continue;
                    }
                }
            };

            let status_after_hooks = r.status()?;
            if status_after_hooks.staged.is_empty() {
                continue;
            }

            match r.create_commit(&commit_message) {
                Ok(()) => {
                    println!("[{label}] Committed: {commit_message}");
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
