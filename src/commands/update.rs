//! `sgit update` - init + update submodules recursively, via git CLI.

use crate::RepoTree;
use crate::error::Result;
use crate::git::SubmoduleUpdateCommand;

/// Initialize and update all submodules recursively for the root repo.
pub fn run() -> Result<()> {
    let tree = RepoTree::discover(None)?;
    let label = tree.root.label();
    match tree.root.git(&SubmoduleUpdateCommand) {
        Ok(()) => println!("[{label}] Submodules updated (init + recursive)"),
        Err(e) => eprintln!("[{label}] Error updating submodules: {e}"),
    }
    Ok(())
}
