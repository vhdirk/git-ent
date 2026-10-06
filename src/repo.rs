use crate::error::GitNestError;
use crate::error::Result;
use crate::git::{head_commit, signature};
use crate::repo_tree::{ChangeKind, StatusEntry};
use git2::StatusOptions;
use git2::build::CheckoutBuilder;
use git2::{BranchType, ErrorCode, IndexAddOption, Repository};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct RepoStatus {
    pub staged: Vec<StatusEntry>,
    pub unstaged: Vec<StatusEntry>,
    pub untracked: Vec<StatusEntry>,
}

impl RepoStatus {
    /// Check if the repo has no changes.
    pub fn is_empty(&self) -> bool {
        self.staged.is_empty() && self.unstaged.is_empty() && self.untracked.is_empty()
    }
}

/// A handle to a git repository within the tree.
///
/// Owns its [`Repository`] and remembers where it sits in the tree.
pub struct Repo {
    /// The opened libgit2 repository.
    pub repo: Repository,
    /// Absolute path to the working directory.
    pub workdir: PathBuf,
    /// Path relative to the top-level repo (`"."` for the root).
    pub prefix: Option<PathBuf>,
}

impl Repo {
    /// Human-readable label (relative path, or `"."` for root).
    pub fn label(&self) -> String {
        let p = self.prefix.clone().unwrap_or(PathBuf::from(""));
        let ps = p.to_string_lossy();
        if ps.is_empty() {
            ".".into()
        } else {
            ps.into_owned()
        }
    }

    /// Label suitable for `git status`-style output
    /// (`"(top-level)"` for the root).
    pub fn display_label(&self) -> String {
        if self.prefix.is_none() {
            "(top-level)".into()
        } else {
            self.prefix.as_ref().unwrap().to_string_lossy().into_owned()
        }
    }

    /// Return the current branch name, or `None` if detached.
    pub fn branch_name(&self) -> Option<String> {
        if self.repo.head_detached().unwrap_or(false) {
            return None;
        }
        let head = self.repo.head().ok()?;
        head.shorthand().ok().map(str::to_string)
    }

    /// Stage one path - add if it exists, remove if it was deleted.
    pub fn stage(&self, rel: &Path) -> Result<()> {
        let mut index = self.repo.index()?;
        let full = self
            .repo
            .workdir()
            .map(|w| w.join(rel))
            .unwrap_or_else(|| rel.to_path_buf());
        println!("Staging path: {} {} ", full.display(), rel.display());
        if full.exists() {
            index.add_all([rel], IndexAddOption::DEFAULT, None)?;
        } else {
            index.remove_path(rel)?;
        }
        index.write()?;
        Ok(())
    }

    /// Create a commit on the current branch from the repo's index.
    pub fn create_commit(&self, message: &str) -> Result<()> {
        let sig = signature(&self.repo)?;
        let mut index = self.repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;

        // Collect parents: HEAD if any.
        let parents = match head_commit(&self.repo) {
            Ok(c) => vec![c],
            Err(_) => Vec::new(),
        };
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

        self.repo
            .commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)?;
        Ok(())
    }

    /// Return all staged, unstaged and untracked entries.
    pub fn status(&self) -> Result<RepoStatus> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true)
            .recurse_untracked_dirs(true)
            // treat submodules as unchanged when only their workdir differs;
            // we handle submodule pointer staging explicitly elsewhere.
            .exclude_submodules(false);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();

        for s in statuses.iter() {
            let flags = s.status();
            let path = match s.path().ok() {
                Some(p) => p.to_string(),
                None => continue,
            };

            // -- index (staged) --
            let kind_staged = if flags.contains(git2::Status::INDEX_NEW) {
                Some(ChangeKind::New)
            } else if flags.contains(git2::Status::INDEX_DELETED) {
                Some(ChangeKind::Deleted)
            } else if flags.contains(git2::Status::INDEX_RENAMED) {
                Some(ChangeKind::Renamed)
            } else if flags.contains(git2::Status::INDEX_TYPECHANGE) {
                Some(ChangeKind::TypeChange)
            } else if flags.contains(git2::Status::INDEX_MODIFIED) {
                Some(ChangeKind::Modified)
            } else {
                None
            };
            if let Some(k) = kind_staged {
                staged.push(StatusEntry {
                    kind: k,
                    path: path.clone(),
                });
            }

            // -- workdir (unstaged / untracked) --
            if flags.contains(git2::Status::WT_NEW) {
                untracked.push(StatusEntry {
                    kind: ChangeKind::New,
                    path: path.clone(),
                });
            } else if flags.contains(git2::Status::WT_DELETED) {
                unstaged.push(StatusEntry {
                    kind: ChangeKind::Deleted,
                    path: path.clone(),
                });
            } else if flags.contains(git2::Status::WT_RENAMED) {
                unstaged.push(StatusEntry {
                    kind: ChangeKind::Renamed,
                    path: path.clone(),
                });
            } else if flags.contains(git2::Status::WT_TYPECHANGE) {
                unstaged.push(StatusEntry {
                    kind: ChangeKind::TypeChange,
                    path,
                });
            } else if flags.contains(git2::Status::WT_MODIFIED) {
                unstaged.push(StatusEntry {
                    kind: ChangeKind::Modified,
                    path,
                });
            }
        }
        Ok(RepoStatus {
            staged,
            unstaged,
            untracked,
        })
    }

    /// Convenience: `true` if `RepoTree` has any staged, unstaged or untracked changes.
    pub fn has_changes(&self) -> Result<bool> {
        let status = self.status()?;
        Ok(
            !status.staged.is_empty()
                || !status.unstaged.is_empty()
                || !status.untracked.is_empty(),
        )
    }

    /// `git add -A`: stage untracked + modified, remove deleted tracked files.
    ///
    /// - `repo`: repository whose index is updated.
    pub fn stage_all(&self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.update_all(["*"].iter(), None)?;
        index.write()?;
        Ok(())
    }

    /// `git add -u`: tracked-file changes only (skip untracked).
    ///
    /// - `repo`: repository whose index is updated.
    pub fn stage_update(&self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.update_all(["*"].iter(), None)?;
        index.write()?;
        Ok(())
    }

    /// Create `name` from current HEAD when missing, then check it out.
    ///
    /// - `repo`: repository to update.
    /// - `name`: local branch name.
    pub fn checkout(&self, name: &str, create: bool) -> Result<bool> {
        let (branch, created) = match self.repo.find_branch(name, BranchType::Local) {
            Ok(b) => (b, false),
            Err(e) if e.code() == ErrorCode::NotFound => {
                if !create {
                    return Err(GitNestError::BranchNotFound(name.to_string()));
                }
                let head = self.repo.head()?.peel_to_commit()?;
                let branch = self.repo.branch(name, &head, false)?;
                (branch, true)
            }
            Err(e) => return Err(e.into()),
        };

        let reference = branch.get();
        let refname = reference.name()?;

        self.repo.set_head(refname)?;
        let mut co = CheckoutBuilder::new();
        self.repo.checkout_head(Some(&mut co))?;
        Ok(created)
    }
}
