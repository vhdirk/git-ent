//! `sgit clone` - recursive clone via libgit2.

use std::path::PathBuf;

use git2::{FetchOptions, Repository, SubmoduleUpdateOptions, build::RepoBuilder};

use crate::error::Result;

/// Clone `url` into `dest` (inferred from `url` when omitted), recursively
/// including all submodules.
pub fn run(url: &str, dest: Option<&str>) -> Result<()> {
    let inferred = infer_dest(url);
    let dest = dest.unwrap_or(&inferred);
    let dest_path = PathBuf::from(dest);

    println!("Cloning {url} into {dest} (recursive)...");
    let mut fetch = FetchOptions::new();
    fetch.remote_callbacks(crate::commands::push::remote_callbacks());
    let repo = RepoBuilder::new()
        .fetch_options(fetch)
        .clone(url, &dest_path)?;
    init_submodules_recursive(&repo)?;
    println!("Done.");
    Ok(())
}

/// Initialize and update all submodules recursively after clone.
///
/// - `repo`: newly cloned repository root.
fn init_submodules_recursive(repo: &Repository) -> Result<()> {
    for mut sm in repo.submodules()? {
        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(crate::commands::push::remote_callbacks());
        let mut update_opts = SubmoduleUpdateOptions::new();
        update_opts.fetch(fetch_opts);
        sm.update(true, Some(&mut update_opts))?;
        // Recurse into the (now-initialised) submodule.
        if let Ok(sub_repo) = sm.open() {
            init_submodules_recursive(&sub_repo)?;
        }
    }
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
