use crate::cli::command::{Cmd, Context};
use clap::Args;
use std::path::PathBuf;

use git2::{FetchOptions, Repository, SubmoduleUpdateOptions, build::RepoBuilder};

use crate::error::Result;

/// Clone a repository recursively (including all submodules).
#[derive(Default, Debug, Args)]
pub struct CloneCmd {
    /// The repository URL.
    pub url: String,
    /// Destination directory (inferred from URL if omitted).
    pub dest: Option<PathBuf>,
}

impl Cmd for CloneCmd {
    fn run(&self, _ctx: &Context) -> Result<()> {
        let dest = self
            .dest
            .clone()
            .unwrap_or_else(|| infer_clone_destination(&self.url));

        println!(
            "Cloning {} into {} (recursive)...",
            &self.url,
            dest.display()
        );
        let mut fetch = FetchOptions::new();
        fetch.remote_callbacks(crate::cli::push::remote_callbacks());
        let repo = RepoBuilder::new()
            .fetch_options(fetch)
            .clone(&self.url, &dest)?;
        init_submodules_recursive(&repo)?;
        println!("Done.");
        Ok(())
    }
}

/// Initialize and update all submodules recursively after clone.
///
/// - `repo`: newly cloned repository root.
fn init_submodules_recursive(repo: &Repository) -> Result<()> {
    for mut sm in repo.submodules()? {
        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(crate::cli::push::remote_callbacks());
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
pub fn infer_clone_destination(url: &str) -> PathBuf {
    let trimmed = url.trim_end_matches('/');
    // Strip any final path separator-like component.
    let last = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed);
    PathBuf::from(last.strip_suffix(".git").unwrap_or(last))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Verifies `.git` suffix stripping from inferred destination names.
    fn infer_strips_git_suffix() {
        assert_eq!(
            infer_clone_destination("https://example.com/org/my-repo.git"),
            PathBuf::from("my-repo")
        );
    }

    #[test]
    /// Verifies trailing slash handling in destination inference.
    fn infer_strips_trailing_slash() {
        assert_eq!(
            infer_clone_destination("https://example.com/org/my-repo/"),
            PathBuf::from("my-repo")
        );
    }

    #[test]
    /// Verifies plain URL/repo-name inference without suffix transforms.
    fn infer_plain_name() {
        assert_eq!(
            infer_clone_destination("https://example.com/org/my-repo"),
            PathBuf::from("my-repo")
        );
    }
}
