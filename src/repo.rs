use std::fs;
use std::path::{Path, PathBuf};

use crate::config::SgitConfig;
use crate::error::Result;
use crate::git::GitCmd;
use crate::git::branch::BranchInfo;
use crate::git::clone::CloneCmd;
use crate::git::config::{ConfigCmd, ConfigOutput};
use crate::git::rev_parse::{self, RevParseCmd};
use crate::git::show_ref::ShowRefCmd;
use crate::git::status::{Status, StatusCmd};
use crate::git::symbolic_ref::SymbolicRefCmd;
/// A handle to a git repository within the tree.
///
/// Remembers its working directory, prefix, and configured Git executable.
pub struct Repo {
    /// Absolute path to the working directory.
    pub workdir: PathBuf,
    /// Path relative to the top-level repo (`"."` for the root).
    pub prefix: PathBuf,
}

impl AsRef<Path> for Repo {
    fn as_ref(&self) -> &Path {
        &self.workdir
    }
}

impl Repo {
    /// Discover the worktree containing `path` and return its root repository.
    pub fn discover(path: impl AsRef<Path>) -> Result<Repo> {
        let cmd = RevParseCmd::Parse(rev_parse::Parse {
            items: vec![rev_parse::Item::Query(rev_parse::RepoQuery::ShowToplevel)],
            ..Default::default()
        });
        let workdir = match cmd.run(path.as_ref())? {
            rev_parse::RevParseOutput::Path(path) => path,
            _ => return Err(crate::SgitError::ParseError("unexpected return".into())),
        };
        let workdir = fs::canonicalize(&workdir).unwrap_or(workdir);

        Ok(Repo {
            workdir,
            prefix: PathBuf::from("."),
        })
    }

    /// Execute a Git command in this repository's working directory.
    pub fn git<C: GitCmd>(&self, command: &C) -> Result<C::Output> {
        command.run(&self.workdir)
    }

    pub fn clone(
        cwd: impl AsRef<Path>,
        url: &str,
        destination: impl AsRef<Path>,
        recurse_submodules: bool,
    ) -> Result<Repo> {
        let cmd = CloneCmd {
            url: url.to_string(),
            destination: destination.as_ref().to_path_buf(),
            recurse_submodules,
        };
        cmd.run(cwd.as_ref())?;
        Ok(Repo {
            workdir: cmd.destination,
            prefix: PathBuf::from("."),
        })
    }

    /// Return all staged, unstaged and untracked entries.
    pub fn status(&self) -> Result<Status> {
        self.git(&StatusCmd {
            untracked_files: true,
            ignore_submodules_dirty: true,
        })
    }

    /// Human-readable label (relative path, or `"."` for root).
    pub fn label(&self) -> String {
        let p = self.prefix.to_string_lossy();
        if p.is_empty() {
            ".".into()
        } else {
            p.into_owned()
        }
    }

    /// Label suitable for `git status`-style output
    /// (`"(top-level)"` for the root).
    pub fn display_label(&self) -> String {
        if self.prefix.as_os_str().is_empty() || self.prefix == Path::new(".") {
            "(top-level)".into()
        } else {
            self.prefix.to_string_lossy().into_owned()
        }
    }

    /// Return the current branch name, or `None` if detached.
    pub fn branch_name(&self) -> Option<String> {
        use crate::git::symbolic_ref;
        let cmd = SymbolicRefCmd::Read(symbolic_ref::Read {
            name: "HEAD".into(),
            quiet: true,
            short: true,
            ..Default::default()
        });
        match self.git(&cmd) {
            Ok(symbolic_ref::SymbolicRefOutput::Ref(name)) => name,
            _ => None,
        }
    }

    /// Convenience: `true` if `RepoTree` has any staged, unstaged or untracked changes.
    pub fn has_changes(&self) -> Result<bool> {
        let status = self.status()?;
        Ok(!status.is_empty())
    }

    /// Return `true` if a local branch with `name` exists in this repository.
    pub fn branch_exists(&self, name: &str) -> bool {
        use crate::git::show_ref;

        let cmd = ShowRefCmd::Verify(show_ref::Verify {
            refs: vec![format!("refs/heads/{}", name)],
            quiet: true,
            ..Default::default()
        });

        match self.git(&cmd) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Return all local branch names in this repository.
    pub fn list_branches(&self) -> Result<Vec<BranchInfo>> {
        use crate::git::branch;

        let cmd = branch::BranchCmd::List(branch::List {
            format: Some("%(refname:short)".into()),
            ..Default::default()
        });

        match self.git(&cmd)? {
            Some(names) => Ok(names),
            None => Ok(vec![]),
        }
    }

    pub fn submodule_paths(&self) -> Result<Vec<PathBuf>> {
        let cmd = ConfigCmd::Get(crate::git::config::Get {
            all: true,
            show_names: true,
            regexp: true,
            value_pattern: Some(r"^submodule\..*\.path$".into()),
            file: Some(".gitmodules".into()),
            ..Default::default()
        });

        match self.git(&cmd)? {
            ConfigOutput::Values(v) => Ok(v.iter().map(PathBuf::from).collect()),
            _ => Ok(vec![]),
        }
    }

    /// Recursively collect initialized submodules depth-first.
    ///
    /// - `top`: absolute workdir of the top-level repository.
    /// - `cfg_chain`: active exclusion configs, each paired with the repo root
    ///   path it is relative to.
    /// - `acc`: depth-first accumulator of discovered submodule handles.
    pub fn collect_submodules(
        &self,
        top: &Path,
        cfg_chain: &mut Vec<(PathBuf, SgitConfig)>,
        acc: &mut Vec<Repo>,
    ) -> Result<()> {
        if !self.workdir.join(".gitmodules").exists() {
            return Ok(());
        }

        let sm_paths = self.submodule_paths()?;

        for sm_rel in sm_paths {
            let sub_path = self.workdir.join(&sm_rel);
            // Skip submodules excluded by any active config. Each config matches
            // against paths relative to the repo root that owns that config.

            let excluded = cfg_chain.iter().any(|(cfg_root, cfg)| {
                sub_path
                    .strip_prefix(cfg_root)
                    .ok()
                    .is_some_and(|rel| cfg.is_excluded(rel))
            });
            if excluded {
                continue;
            }

            // Check if initialized: modern Git submodules have a .git file or directory.
            // Confirm with git rev-parse --resolve-git-dir.
            let git_pointer = sub_path.join(".git");
            if !git_pointer.exists() {
                continue;
            }

            let rev_parse_cmd = RevParseCmd::Parse(rev_parse::Parse {
                items: vec![rev_parse::Item::Query(rev_parse::RepoQuery::ResolveGitDir(
                    git_pointer,
                ))],
                ..Default::default()
            });

            let sub_path = if let Ok(rev_parse::RevParseOutput::Path(p)) = self.git(&rev_parse_cmd)
            {
                p
            } else {
                continue;
            };

            let sub_workdir = fs::canonicalize(&sub_path).unwrap_or(sub_path);

            // Load this submodule's config, inheriting the parent repo's git executable.
            let parent_cfg = &cfg_chain.last().unwrap().1;
            let sub_cfg = parent_cfg.load_submodule(&sub_workdir);

            let prefix = sub_workdir
                .strip_prefix(top)
                .unwrap_or(&sub_workdir)
                .to_path_buf();
            let sub = Repo {
                workdir: sub_workdir.clone(),
                prefix,
            };

            cfg_chain.push((sub_workdir.clone(), sub_cfg));
            // recurse first (depth-first: deepest appears first)
            sub.collect_submodules(top, cfg_chain, acc)?;
            cfg_chain.pop();

            acc.push(sub);
        }
        Ok(())
    }
}

/// Strip trailing slashes and a `.git` suffix, keep the final path segment.
pub(crate) fn infer_clone_destination(url: &str) -> PathBuf {
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
            "my-repo"
        );
    }

    #[test]
    /// Verifies trailing slash handling in destination inference.
    fn infer_strips_trailing_slash() {
        assert_eq!(
            infer_clone_destination("https://example.com/org/my-repo/"),
            "my-repo"
        );
    }

    #[test]
    /// Verifies plain URL/repo-name inference without suffix transforms.
    fn infer_plain_name() {
        assert_eq!(
            infer_clone_destination("https://example.com/org/my-repo"),
            "my-repo"
        );
    }
}
