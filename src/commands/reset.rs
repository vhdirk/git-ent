//! `sgit reset` - mixed, hard or soft reset across the whole tree.

use crate::RepoTree;
use crate::error::Result;

/// Perform recursive reset across the repo tree.
///
/// - `ref_`: optional revision to reset to, defaults to `HEAD`.
/// - `hard`: when `true`, uses hard reset; otherwise mixed reset.
pub fn run(ref_: Option<&str>, hard: bool) -> Result<()> {
    let target = ref_.unwrap_or("HEAD");
    let tree = RepoTree::discover(None)?;

    for r in tree.all() {
        let label = r.label();
        let out = r.reset(
            if hard {
                crate::git::ResetMode::Hard
            } else {
                crate::git::ResetMode::Mixed
            },
            Some(target),
        );

        match out {
            Ok(()) if hard => println!("[{label}] Hard reset to {target}"),
            Ok(()) => println!("[{label}] Reset (unstaged all changes)"),
            Err(e) => eprintln!("[{label}] Error resetting: {e}"),
        }
    }
    Ok(())
}
