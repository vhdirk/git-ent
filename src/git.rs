//! Small libgit2 helpers shared by the command modules.
//!
//! sgit never shells out to the `git` binary - every operation is
//! implemented on top of [`git2`]. This module houses the handful of
//! small helpers that didn't fit in [`crate::repo_tree`].

use git2::{Commit, ObjectType, Oid, Remote, Repository, Signature};

use crate::error::{Result, SgitError};

/// Build a [`Signature`] for `repo`.
///
/// Prefers the repo's configured `user.name`/`user.email` but falls back
/// to the standard `GIT_AUTHOR_*` / `GIT_COMMITTER_*` environment
/// variables - which `libgit2` does **not** consult by default - so
/// hermetic test environments still work.
pub fn signature(repo: &Repository) -> Result<Signature<'static>> {
    if let Ok(sig) = repo.signature() {
        // Clone into a 'static signature.
        return Ok(Signature::now(
            sig.name().unwrap_or("Unknown"),
            sig.email().unwrap_or("unknown@example.com"),
        )?);
    }
    let name = std::env::var("GIT_AUTHOR_NAME")
        .or_else(|_| std::env::var("GIT_COMMITTER_NAME"))
        .unwrap_or_else(|_| "Unknown".into());
    let email = std::env::var("GIT_AUTHOR_EMAIL")
        .or_else(|_| std::env::var("GIT_COMMITTER_EMAIL"))
        .unwrap_or_else(|_| "unknown@example.com".into());
    Ok(Signature::now(&name, &email)?)
}

/// Peel the current `HEAD` to a commit.
pub fn head_commit(repo: &Repository) -> Result<Commit<'_>> {
    Ok(repo.head()?.peel_to_commit()?)
}

/// Resolve a revision string (e.g. `"HEAD~1"`, a branch name, a SHA)
/// to an [`Oid`].
pub fn resolve_revision(repo: &Repository, rev: &str) -> Result<Oid> {
    let obj = repo.revparse_single(rev)?;
    Ok(obj.peel(ObjectType::Commit)?.id())
}

/// Return the name of the repo's first remote, or `None`.
pub fn first_remote(repo: &Repository) -> Option<String> {
    let list = repo.remotes().ok()?;
    list.get(0).ok()?.map(str::to_string)
}

/// Find a remote by name.
pub fn find_remote<'a>(repo: &'a Repository, name: &str) -> Result<Remote<'a>> {
    repo.find_remote(name)
        .map_err(|e| SgitError::Other(format!("remote '{name}' not found: {e}")))
}
