use std::fs;
use std::path::{Path, PathBuf};

use crate::config::SgitConfig;
use crate::error::Result;
use crate::git::{
    AddCommand, BranchExistsCommand, Git, GitCommand, ListBranchesCommand, ResetCommand, ResetMode,
    RestoreCommand, RevParseCommand, StatusCommand, StatusEntry, SubmodulePathsCommand,
    SymbolicRefCommand,
};

/// A handle to a git repository within the tree.
///
/// Remembers its working directory, prefix, and configured Git executable.
pub struct Repo {
    /// Absolute path to the working directory.
    pub workdir: PathBuf,
    /// Path relative to the top-level repo (`"."` for the root).
    pub prefix: PathBuf,
    /// Configured Git executable runner for this repository.
    pub git: Git,
}

impl AsRef<Path> for Repo {
    fn as_ref(&self) -> &Path {
        &self.workdir
    }
}

impl Repo {
    // /// Execute a Git command in this repository's working directory using its configured Git executable.
    // pub fn git<C: GitCommand>(&self, command: &C) -> Result<C::Output> {
    //     command.run(&self.git, self)
    // }

    // /// Execute a Git command, returning Output even if exit status is non-zero, using its configured Git executable.
    // pub fn try_git<C: GitCommand>(&self, command: &C) -> Result<C::Output> {
    //     command.try_run(&self.git, self)
    // }

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
        self.try_git(&SymbolicRefCommand::head()).ok().flatten()
    }

    /// Return all staged, unstaged and untracked entries.
    pub fn list_status(&self) -> Result<(Vec<StatusEntry>, Vec<StatusEntry>, Vec<StatusEntry>)> {
        self.git(&StatusCommand {
            untracked_files: true,
            ignore_submodules_dirty: true,
        })
    }

    /// Convenience: `true` if `RepoTree` has any staged, unstaged or untracked changes.
    pub fn has_changes(&self) -> Result<bool> {
        let (s, u, ut) = self.list_status()?;
        Ok(!s.is_empty() || !u.is_empty() || !ut.is_empty())
    }

    /// Return `true` if a local branch with `name` exists in this repository.
    pub fn branch_exists(&self, name: &str) -> bool {
        self.try_git(&BranchExistsCommand::new(name))
            .unwrap_or(false)
    }

    /// Return all local branch names in this repository.
    pub fn list_branches(&self) -> Result<Vec<String>> {
        self.git(&ListBranchesCommand)
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

        let sm_paths = self.try_git(&SubmodulePathsCommand).unwrap_or_default();

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
            let check_init = RevParseCommand::resolve_git_dir(&git_pointer);
            if check_init.try_run(&self.git, &self.workdir).is_err() {
                continue;
            }

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
                git: self.git.clone(),
            };

            cfg_chain.push((sub_workdir.clone(), sub_cfg));
            // recurse first (depth-first: deepest appears first)
            sub.collect_submodules(top, cfg_chain, acc)?;
            cfg_chain.pop();

            acc.push(sub);
        }
        Ok(())
    }

    pub fn restore(&self, staged: bool, paths: Vec<PathBuf>) -> Result<()> {

        let cmd = RestoreCommand { staged, paths };
        self.git(&cmd)
    }

    pub fn add(&self, cmd: &AddCommand) -> Result<()> {
        self.git(cmd)
    }

    pub fn reset(&self, mode: ResetMode, target: Option<&str>) -> Result<()> {
        self.git(&ResetCommand { mode, target })
    }
}
