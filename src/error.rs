use std::path::PathBuf;

use thiserror::Error;

/// The git-ent error type.
#[derive(Debug, Error)]
pub enum GitEntError {
    /// A libgit2 call failed.
    #[error("git2: {0}")]
    Git2(#[from] git2::Error),

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
pub type Result<T> = std::result::Result<T, GitEntError>;
