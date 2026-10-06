//! `sgit commit` - depth-first commit with automatic submodule-pointer staging.

use crate::RepoTree;
use crate::cmd::command::{Command, Context};
use crate::error::{Result, SgitError};
use clap::Args;

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

impl Command for CommitCmd {
    /// Commit staged changes across all repos in depth-first order.
    ///
    /// - `message`: optional commit message; when absent, opens an editor.
    /// - `no_verify`: accepted for CLI parity (hooks are not run by libgit2).
    fn run(&self, ctx: &Context) -> Result<()> {
        // `no_verify` is accepted for CLI parity with `git commit`, but
        // libgit2 never invokes pre-commit/commit-msg hooks anyway - so
        // every commit made by sgit is implicitly "no-verify".
        let tree = RepoTree::discover(ctx.workdir.as_deref())?;
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

            let status = r.status()?;
            if status.staged.is_empty() {
                continue;
            }

            match r.create_commit(msg) {
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
