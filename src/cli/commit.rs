use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::RepoTree;
use crate::cli::command::{Cmd, Context};
use crate::error::{GitNestError, Result};
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
    fn run(&self, ctx: &Context) -> Result<()> {
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
                    None => return Err(GitNestError::EmptyCommitMessage),
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
                match run_commit_hooks(&r.repo, msg) {
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

fn run_commit_hooks(repo: &git2::Repository, message: &str) -> Result<String> {
    run_hook(repo, "pre-commit", None)?;

    let mut message_file = NamedTempFile::new()?;
    message_file.write_all(message.as_bytes())?;
    message_file.flush()?;
    run_hook(repo, "commit-msg", Some(message_file.path()))?;
    Ok(std::fs::read_to_string(message_file.path())?)
}

fn run_hook(repo: &git2::Repository, name: &str, argument: Option<&Path>) -> Result<()> {
    let path = hook_path(repo, name)?;
    let metadata = match std::fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if !is_executable(&metadata) {
        return Ok(());
    }

    let workdir = repo.workdir().unwrap_or(repo.path());
    let mut command = Command::new(path);
    command
        .current_dir(workdir)
        .env("GIT_DIR", repo.path())
        .env_remove("GIT_INDEX_FILE");
    if repo.workdir().is_some() {
        command.env("GIT_WORK_TREE", workdir);
    } else {
        command.env_remove("GIT_WORK_TREE");
    }
    if let Some(argument) = argument {
        command.arg(argument);
    }

    let status = command.status()?;
    if !status.success() {
        return Err(crate::error::GitNestError::Other(format!(
            "{name} hook failed with status {status}"
        )));
    }
    Ok(())
}

fn hook_path(repo: &git2::Repository, name: &str) -> Result<PathBuf> {
    let config = repo.config()?;
    let hooks_dir = match config.get_path("core.hookspath") {
        Ok(path) => path,
        Err(error) if error.code() == git2::ErrorCode::NotFound => repo.path().join("hooks"),
        Err(error) => return Err(error.into()),
    };
    let hooks_dir = if hooks_dir.is_absolute() {
        hooks_dir
    } else {
        repo.workdir().unwrap_or(repo.path()).join(hooks_dir)
    };
    Ok(hooks_dir.join(name))
}

#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    metadata.is_file()
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
