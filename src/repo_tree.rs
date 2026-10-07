//! The [`RepoTree`] abstraction: a top-level repo and all its (nested)
//! submodule repos, treated as a depth-first traversable tree.

use std::fs;
use std::path::{Path, PathBuf};

use git2::{Repository, SubmoduleIgnore};

use crate::config::GitEntConfig;
use crate::error::{GitEntError, Result};
use crate::repo::{Repo, RepoStatus};

/// A file change classification, modelled after `git status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    New,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
    Untracked,
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
            ChangeKind::Untracked => "untracked:  ",
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
    pub root: Repo,
    /// All submodules, deepest first.
    pub submodules: Vec<Repo>,
}

/// Join `prefix` and `path` for display (drops the empty prefix cleanly).
pub(crate) fn prefix_path(prefix: &Path, path: &str) -> String {
    if prefix.as_os_str().is_empty() {
        path.to_string()
    } else {
        prefix.join(path).to_string_lossy().into_owned()
    }
}

impl RepoTree {
    /// Discover the repo containing `start` (defaulting to cwd) and
    /// recursively collect every nested submodule.
    ///
    /// TODO: traverse directories upwards to find the root of the repo.
    /// It's the one where .git is an actual directory
    pub fn discover(start: Option<&Path>) -> Result<Self> {
        let cwd = match start {
            Some(p) => p.to_path_buf(),
            None => std::env::current_dir()?,
        };
        let repo = Repository::discover(&cwd)?;
        let workdir = repo
            .workdir()
            .ok_or_else(|| GitEntError::Other("repo has no working directory".into()))?
            .to_path_buf();

        // TODO: `start` will not always be the root of the repo tree:
        // If we cd into a submodule and type 'git-ent status', we should also see changes
        // of parent and sibling submodules.

        // fs::canonicalize normalises symlinks; we want a stable top dir.
        let top = fs::canonicalize(&workdir).unwrap_or(workdir.clone());
        let root = Repo {
            repo,
            workdir: top.clone(),
            prefix: None,
        };

        let mut submodules = Vec::new();
        let root_cfg = GitEntConfig::load(&top);
        let mut cfg_chain = vec![(top.clone(), root_cfg)];
        collect_submodules(&root.repo, &top, &top, &mut cfg_chain, &mut submodules)?;
        Ok(Self { root, submodules })
    }

    /// All repos, depth-first, root last.
    pub fn all(&self) -> Vec<&Repo> {
        let mut v: Vec<&Repo> = self.submodules.iter().collect();
        v.push(&self.root);
        v
    }

    /// Resolve `filename` (relative to cwd) to the *deepest* repo that
    /// owns it, along with the path relative to that repo's workdir.
    pub fn resolve_file(&self, filename: &Path) -> Option<(&Repo, PathBuf)> {
        let cwd = std::env::current_dir().ok()?;
        self.resolve_file_from(&cwd, filename)
    }

    pub fn stage(&self, filename: &Path) -> Result<()> {
        if let Some((repo, rel)) = self.resolve_file(filename) {
            repo.stage(&rel)
        } else {
            Err(GitEntError::Other(format!(
                "File {} is not in any known repo/submodule",
                filename.display()
            )))
        }
    }

    pub fn status(&self) -> Result<RepoStatus> {
        let mut all_staged: Vec<StatusEntry> = Vec::new();
        let mut all_unstaged: Vec<StatusEntry> = Vec::new();
        let mut all_untracked: Vec<StatusEntry> = Vec::new();

        for r in self.all() {
            let prefix = if r.prefix == None {
                PathBuf::new()
            } else {
                r.prefix.clone().unwrap()
            };
            let status = r.status()?;
            for s in status.staged {
                all_staged.push(StatusEntry {
                    kind: s.kind,
                    path: prefix_path(&prefix, &s.path),
                });
            }
            for u in status.unstaged {
                all_unstaged.push(StatusEntry {
                    kind: u.kind,
                    path: prefix_path(&prefix, &u.path),
                });
            }
            for u in status.untracked {
                all_untracked.push(StatusEntry {
                    kind: u.kind,
                    path: prefix_path(&prefix, &u.path),
                });
            }
        }
        Ok(RepoStatus {
            staged: all_staged,
            unstaged: all_unstaged,
            untracked: all_untracked,
        })
    }

    /// Like [`resolve_file`](Self::resolve_file) but with an explicit base
    /// directory, so callers (and tests) don't need to mutate the process
    /// working directory.
    pub fn resolve_file_from(&self, base: &Path, filename: &Path) -> Option<(&Repo, PathBuf)> {
        let raw = base.join(filename);
        let abs = fs::canonicalize(&raw).unwrap_or(raw);

        let mut best: Option<(&Repo, PathBuf, usize)> = None;
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
    pub fn stage_submodule_pointers(&self, r: &Repo) -> Result<Vec<String>> {
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

    /// Build a git-style commit message template (used by `commit` when the
    /// user gives no `-m`).
    pub fn build_commit_template(&self) -> Result<String> {
        let mut lines: Vec<String> = vec![
            "".into(),
            "# Please enter the commit message for your changes. Lines starting".into(),
            "# with '#' will be ignored, and an empty message aborts the commit.".into(),
            "#".into(),
        ];

        let branch_name = self
            .root
            .branch_name()
            .unwrap_or_else(|| "(detached HEAD)".to_string());
        lines.push(format!("# On branch {branch_name}"));
        lines.push("#".into());

        let status = self.status()?;

        if !status.staged.is_empty() {
            lines.push("# Changes to be committed:".into());
            for e in &status.staged {
                lines.push(format!("#\t{}{}", e.kind.label(), e.path));
            }
            lines.push("#".into());
        }
        if !status.unstaged.is_empty() {
            lines.push("# Changes not staged for commit:".into());
            for e in &status.unstaged {
                lines.push(format!("#\t{}{}", e.kind.label(), e.path));
            }
            lines.push("#".into());
        }
        if !status.untracked.is_empty() {
            lines.push("# Untracked files:".into());
            for e in &status.untracked {
                lines.push(format!("#\t{}", e.path));
            }
            lines.push("#".into());
        }
        Ok(lines.join("\n"))
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
    cfg_chain: &mut Vec<(PathBuf, GitEntConfig)>,
    acc: &mut Vec<Repo>,
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
        // Load this submodule's own .git-ent.toml and make it active for the
        // subtree rooted at this submodule.
        let sub_cfg = GitEntConfig::load(&sub_workdir);
        cfg_chain.push((sub_workdir.clone(), sub_cfg));
        // recurse first (depth-first: deepest appears first)
        collect_submodules(&sub_repo, top, &sub_workdir, cfg_chain, acc)?;
        cfg_chain.pop();
        let prefix = sub_workdir
            .strip_prefix(top)
            .unwrap_or(&sub_workdir)
            .to_path_buf();
        acc.push(Repo {
            repo: sub_repo,
            workdir: sub_workdir,
            prefix: Some(prefix),
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
