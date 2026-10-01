//! `sgit clone` - recursive clone via git CLI.

use std::path::{Path, PathBuf};

use crate::Repo;
use crate::error::Result;

/// Clone `url` into `dest` (inferred from `url` when omitted), recursively
/// including all submodules.
pub fn run(url: &str, dest: Option<&Path>) -> Result<()> {
    let inferred = crate::repo::infer_clone_destination(url);
    let dest: PathBuf = dest.unwrap_or(&inferred).into();

    println!("Cloning {} into {} (recursive)...", url, dest.display());

    Repo::clone(&std::env::current_dir()?, url, &dest, true)?;

    println!("Done.");
    Ok(())
}
