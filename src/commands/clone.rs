//! `sgit clone` - recursive clone via git CLI.

use std::path::PathBuf;

use crate::error::Result;
use crate::git::{CloneCommand, Git, GitCommand};

/// Clone `url` into `dest` (inferred from `url` when omitted), recursively
/// including all submodules.
pub fn run(url: &str, dest: Option<&str>) -> Result<()> {
    let inferred = infer_dest(url);
    let dest = dest.unwrap_or(&inferred);
    let dest_path = PathBuf::from(dest);

    println!("Cloning {url} into {dest} (recursive)...");
    let git = Git::default();
    let current_dir = std::env::current_dir()?;
    let cmd = CloneCommand::recursive(url, Some(dest_path));
    cmd.run(&git, &current_dir)?;
    println!("Done.");
    Ok(())
}

/// Strip trailing slashes and a `.git` suffix, keep the final path segment.
pub fn infer_dest(url: &str) -> String {
    let trimmed = url.trim_end_matches('/');
    // Strip any final path separator-like component.
    let last = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed);
    last.strip_suffix(".git").unwrap_or(last).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Verifies `.git` suffix stripping from inferred destination names.
    fn infer_strips_git_suffix() {
        assert_eq!(infer_dest("https://example.com/org/my-repo.git"), "my-repo");
    }

    #[test]
    /// Verifies trailing slash handling in destination inference.
    fn infer_strips_trailing_slash() {
        assert_eq!(infer_dest("https://example.com/org/my-repo/"), "my-repo");
    }

    #[test]
    /// Verifies plain URL/repo-name inference without suffix transforms.
    fn infer_plain_name() {
        assert_eq!(infer_dest("https://example.com/org/my-repo"), "my-repo");
    }
}
