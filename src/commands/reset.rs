//! `sgit reset` - mixed, hard or soft reset across the whole tree.

use git2::{Repository, ResetType};

use crate::RepoTree;
use crate::error::Result;
use crate::git::resolve_revision;

/// Perform recursive reset across the repo tree.
///
/// - `ref_`: optional revision to reset to, defaults to `HEAD`.
/// - `hard`: when `true`, uses hard reset; otherwise mixed reset.
pub fn run(ref_: Option<&str>, hard: bool) -> Result<()> {
    let target = ref_.unwrap_or("HEAD");
    let tree = RepoTree::discover(None)?;

    for r in tree.all() {
        let label = r.label();
        let kind = if hard {
            ResetType::Hard
        } else {
            ResetType::Mixed
        };
        match reset_to(&r.repo, target, kind) {
            Ok(()) if hard => println!("[{label}] Hard reset to {target}"),
            Ok(()) => println!("[{label}] Reset (unstaged all changes)"),
            Err(e) => eprintln!("[{label}] Error resetting: {e}"),
        }
    }
    Ok(())
}

/// Reset a single repository to `target` using the requested reset kind.
///
/// - `repo`: repository to reset.
/// - `target`: revision expression (commit, tag, ref, etc.).
/// - `kind`: reset mode (`Mixed`, `Hard`, ...).
pub(crate) fn reset_to(repo: &Repository, target: &str, kind: ResetType) -> Result<()> {
    let oid = resolve_revision(repo, target)?;
    let obj = repo.find_object(oid, None)?;
    repo.reset(&obj, kind, None)?;
    Ok(())
}
