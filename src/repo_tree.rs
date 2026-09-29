//! The [`RepoTree`] abstraction: a top-level repo and all its (nested)
//! submodule repos, treated as a depth-first traversable tree.

use std::fs;
use std::path::{Path, PathBuf};

use git2::{Repository, StatusOptions, SubmoduleIgnore};

use crate::config::SgitConfig;
use crate::error::{Result, SgitError};

/// A handle to a git repository within the tree.
///
/// Owns its [`Repository`] and remembers where it sits in the tree.
pub struct RepoHandle {
    /// The opened libgit2 repository.
    pub repo: Repository,
    /// Absolute path to the working directory.
    pub workdir: PathBuf,
    /// Path relative to the top-level repo (`"."` for the root).
    pub prefix: PathBuf,
}

impl RepoHandle {
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
        if self.repo.head_detached().unwrap_or(false) {
            return None;
        }
        let head = self.repo.head().ok()?;
        head.shorthand().ok().map(str::to_string)
    }

    /// Return all staged, unstaged and untracked entries.
    pub fn list_status(&self) -> Result<(Vec<StatusEntry>, Vec<StatusEntry>, Vec<StatusEntry>)> {
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
        Ok((staged, unstaged, untracked))
    }

    /// Convenience: `true` if `RepoTree` has any staged, unstaged or untracked changes.
    pub fn has_changes(&self) -> Result<bool> {
        let (s, u, ut) = self.list_status()?;
        Ok(!s.is_empty() || !u.is_empty() || !ut.is_empty())
    }
}

/// A file change classification, modelled after `git status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    New,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
}

impl ChangeKind {
    /// Left-padded label matching `git status` output.
    pub fn label(&self) -> &'static str {
        match self {
            ChangeKind::New => "new file:   ",
            ChangeKind::Modified => "modified:   ",
            ChangeKind::Deleted => "deleted:    ",
            ChangeKind::Renamed => "renamed:    ",
            ChangeKind::TypeChange => "typechange: ",
        }
    }
}

/// A single status entry for a file.
#[derive(Debug, Clone)]
pub struct StatusEntry {
    pub kind: ChangeKind,
    /// Path relative to the owning repository's working directory.
    pub path: String,
}

/// The root repo and all its (nested) submodule repos, in depth-first order.
pub struct RepoTree {
    /// The top-level repository.
    pub root: RepoHandle,
    /// All submodules, deepest first.
    pub submodules: Vec<RepoHandle>,
}

impl RepoTree {
    /// Discover the repo containing `start` (defaulting to cwd) and
    /// recursively collect every nested submodule.
    pub fn discover(start: Option<&Path>) -> Result<Self> {
        let cwd = match start {
            Some(p) => p.to_path_buf(),
            None => std::env::current_dir()?,
        };
        let repo = Repository::discover(&cwd)?;
        let workdir = repo
            .workdir()
            .ok_or_else(|| SgitError::Other("repo has no working directory".into()))?
            .to_path_buf();
        // fs::canonicalize normalises symlinks; we want a stable top dir.
        let top = fs::canonicalize(&workdir).unwrap_or(workdir.clone());
        let root = RepoHandle {
            repo,
            workdir: top.clone(),
            prefix: PathBuf::from("."),
        };
        let mut submodules = Vec::new();
        let root_cfg = SgitConfig::load(&top);
        let mut cfg_chain = vec![(top.clone(), root_cfg)];
        collect_submodules(&root.repo, &top, &top, &mut cfg_chain, &mut submodules)?;
        Ok(Self { root, submodules })
    }

    /// All repos, depth-first, root last.
    pub fn all(&self) -> Vec<&RepoHandle> {
        let mut v: Vec<&RepoHandle> = self.submodules.iter().collect();
        v.push(&self.root);
        v
    }

    /// Resolve `filename` (relative to cwd) to the *deepest* repo that
    /// owns it, along with the path relative to that repo's workdir.
    pub fn resolve_file(&self, filename: &str) -> Option<(&RepoHandle, PathBuf)> {
        let cwd = std::env::current_dir().ok()?;
        self.resolve_file_from(&cwd, filename)
    }

    /// Like [`resolve_file`](Self::resolve_file) but with an explicit base
    /// directory, so callers (and tests) don't need to mutate the process
    /// working directory.
    pub fn resolve_file_from(&self, base: &Path, filename: &str) -> Option<(&RepoHandle, PathBuf)> {
        let raw = base.join(filename);
        let abs = fs::canonicalize(&raw).unwrap_or(raw);

        let mut best: Option<(&RepoHandle, PathBuf, usize)> = None;
        for r in self.all() {
            if let Ok(rel) = abs.strip_prefix(&r.workdir) {
                let depth = r.workdir.components().count();
                if best.as_ref().is_none_or(|b| depth > b.2) {
                    best = Some((r, rel.to_path_buf(), depth));
                }
            }
        }
        best.map(|(r, p, _)| (r, p))
    }

    /// Stage any submodule entries whose working-dir HEAD differs from
    /// the index. Returns the staged paths.
    pub fn stage_submodule_pointers(&self, r: &RepoHandle) -> Result<Vec<String>> {
        let changed = changed_submodule_paths(&r.repo)?;
        if !changed.is_empty() {
            let mut index = r.repo.index()?;
            for p in &changed {
                index.add_path(Path::new(p))?;
            }
            index.write()?;
        }
        Ok(changed)
    }
}

// -- free helpers ------------------------------------------------------------

/// Recursively collect initialized submodules depth-first.
///
/// - `repo`: parent repository whose submodules are inspected.
/// - `top`: absolute workdir of the top-level repository.
/// - `parent_workdir`: workdir of the current parent repository.
/// - `cfg_chain`: active exclusion configs, each paired with the repo root
///   path it is relative to.
/// - `acc`: depth-first accumulator of discovered submodule handles.
fn collect_submodules(
    repo: &Repository,
    top: &Path,
    parent_workdir: &Path,
    cfg_chain: &mut Vec<(PathBuf, SgitConfig)>,
    acc: &mut Vec<RepoHandle>,
) -> Result<()> {
    let subs = match repo.submodules() {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    for sm in subs {
        let sub_path = parent_workdir.join(sm.path());
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
        if !sub_path.join(".git").exists() {
            // not initialised yet
            continue;
        }
        let sub_workdir = fs::canonicalize(&sub_path).unwrap_or(sub_path);
        let sub_repo = match Repository::open(&sub_workdir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        // Load this submodule's own .sgit.toml and make it active for the
        // subtree rooted at this submodule.
        let sub_cfg = SgitConfig::load(&sub_workdir);
        cfg_chain.push((sub_workdir.clone(), sub_cfg));
        // recurse first (depth-first: deepest appears first)
        collect_submodules(&sub_repo, top, &sub_workdir, cfg_chain, acc)?;
        cfg_chain.pop();
        let prefix = sub_workdir
            .strip_prefix(top)
            .unwrap_or(&sub_workdir)
            .to_path_buf();
        acc.push(RepoHandle {
            repo: sub_repo,
            workdir: sub_workdir,
            prefix,
        });
    }
    Ok(())
}

/// Return the submodule paths in `repo` whose in-tree HEAD differs from
/// the index entry - i.e. the pointer is dirty and wants committing.
pub fn changed_submodule_paths(repo: &Repository) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let subs = match repo.submodules() {
        Ok(v) => v,
        Err(_) => return Ok(out),
    };
    for sm in subs {
        let status = repo.submodule_status(sm.name().unwrap_or(""), SubmoduleIgnore::None)?;
        if status.contains(git2::SubmoduleStatus::WD_MODIFIED)
            || status.contains(git2::SubmoduleStatus::INDEX_MODIFIED)
        {
            if let Some(p) = sm.path().to_str() {
                out.push(p.to_string());
            }
        }
    }
    Ok(out)
}
