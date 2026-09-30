//! Error types for sgit.

use std::path::PathBuf;

use thiserror::Error;

/// The sgit error type.
#[derive(Debug, Error)]
pub enum SgitError {
    /// Failed to execute git process (e.g. executable not found or permission denied).
    #[error("failed to execute git in {}: {source}", workdir.display())]
    GitLaunch {
        workdir: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Git command exited with a non-zero status.
    #[error("git failed in {}: {stderr}", workdir.display())]
    GitExit {
        workdir: PathBuf,
        code: Option<i32>,
        stderr: String,
    },

    /// An I/O error occurred.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// A file could not be resolved to any (sub)module in the tree.
    #[error("{0} is not in any known repo/submodule")]
    FileNotInTree(String),

    /// The target branch does not exist in the repo.
    #[error("branch '{0}' does not exist")]
    BranchNotFound(String),

    /// A merge or rebase conflict requires manual resolution.
    #[error("conflict in {repo}: resolve manually, then `cd {workdir}` and run `{hint}`")]
    Conflict {
        repo: String,
        workdir: PathBuf,
        hint: String,
    },

    /// The user provided incompatible CLI arguments.
    #[error("{0}")]
    InvalidUsage(String),

    /// The commit message was empty (user aborted).
    #[error("aborting commit due to empty commit message")]
    EmptyCommitMessage,

    /// A generic, message-only error.
    #[error("{0}")]
    Other(String),
}

/// The crate's [`Result`](std::result::Result) alias.
pub type Result<T> = std::result::Result<T, SgitError>;
