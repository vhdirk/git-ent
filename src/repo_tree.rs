//! The [`RepoTree`] abstraction: a top-level repo and all its (nested)
//! submodule repos, treated as a depth-first traversable tree.

use std::fs;
use std::path::{Path, PathBuf};

use crate::Repo;
use crate::config::SgitConfig;
use crate::error::Result;
use crate::git::add::AddCmd;
use crate::git::status::StatusEntry;

/// Join `prefix` and `path` for display (drops the empty prefix cleanly).
pub(crate) fn prefix_path(prefix: &Path, path: &str) -> String {
    if prefix.as_os_str().is_empty() {
        path.to_string()
    } else {
        prefix.join(path).to_string_lossy().into_owned()
    }
}

/// The root repo and all its (nested) submodule repos, in depth-first order.
pub struct RepoTree {
    /// The top-level repository.
    pub root: Repo,
    /// All submodules, deepest first.
    pub submodules: Vec<Repo>,
}

impl RepoTree {
    /// Discover the repo containing `start` (defaulting to cwd) and
    /// recursively collect every nested submodule.
    pub fn discover(start: Option<&Path>) -> Result<Self> {
        Self::discover_with_global(start, crate::config::global_config_path().as_deref())
    }

    /// Like [`discover`], but with an explicit global config path, used in tests.
    pub fn discover_with_global(start: Option<&Path>, global_path: Option<&Path>) -> Result<Self> {
        let cwd = match start {
            Some(p) => p.to_path_buf(),
            None => std::env::current_dir()?,
        };
        let root = Repo::discover(&cwd)?;
        let top = root.workdir.clone();
        let root_cfg = SgitConfig::load_with_global(&top, global_path);

        let mut submodules = Vec::new();
        let mut cfg_chain = vec![(top.clone(), root_cfg)];
        root.collect_submodules(&top, &mut cfg_chain, &mut submodules)?;
        Ok(Self { root, submodules })
    }

    /// Return all repositories in the tree, deepest submodule first,
    /// with the root repo last.
    pub fn all(&self) -> Vec<&Repo> {
        self.submodules
            .iter()
            .chain(std::iter::once(&self.root))
            .collect()
    }

    /// Find the repo that owns `filename` and the path relative to that repo.
    ///
    /// Chooses the deepest match so submodules shadow their parent directories.
    pub fn resolve_file(&self, filename: &str) -> Option<(&Repo, PathBuf)> {
        let raw = PathBuf::from(filename);
        let abs_raw = if raw.is_absolute() {
            raw
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(&raw))
                .unwrap_or(raw)
        };
        let abs = fs::canonicalize(&abs_raw).unwrap_or(abs_raw);

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

    /// Resolve `filename` relative to `from_dir`.
    pub fn resolve_file_from(&self, from_dir: &Path, filename: &str) -> Option<(&Repo, PathBuf)> {
        let path = Path::new(filename);
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            from_dir.join(path)
        };
        self.resolve_file(abs.to_str().unwrap_or(filename))
    }

    /// Stage any submodule entries whose working-dir HEAD differs from
    /// the index. Returns the staged paths.
    pub fn stage_submodule_pointers(&self, r: &Repo) -> Result<Vec<String>> {
        let changed = changed_submodule_paths(r)?;
        if !changed.is_empty() {
            r.git(&AddCmd::Paths(changed.iter().map(PathBuf::from).collect()))?;
        }
        Ok(changed)
    }

    /// Build a git-style commit message template (used by `commit` when the
    /// user gives no `-m`).
    pub(crate) fn build_commit_template(&self) -> Result<String> {
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

        let mut staged: Vec<StatusEntry> = Vec::new();
        let mut unstaged: Vec<StatusEntry> = Vec::new();
        let mut untracked: Vec<StatusEntry> = Vec::new();

        for r in self.all() {
            let prefix = if r.prefix == Path::new(".") {
                PathBuf::new()
            } else {
                r.prefix.clone()
            };
            let status = r.status()?;
            for e in &status.staged {
                staged.push(StatusEntry {
                    kind: e.kind.clone(),
                    path: prefix_path(&prefix, &e.path),
                });
            }
            for e in &status.unstaged {
                unstaged.push(StatusEntry {
                    kind: e.kind.clone(),
                    path: prefix_path(&prefix, &e.path),
                });
            }
            for e in &status.untracked {
                untracked.push(StatusEntry {
                    kind: e.kind.clone(),
                    path: prefix_path(&prefix, &e.path),
                });
            }
        }

        if !staged.is_empty() {
            lines.push("# Changes to be committed:".into());
            for e in &staged {
                lines.push(format!("#\t{}{}", e.kind.label(), e.path));
            }
            lines.push("#".into());
        }
        if !unstaged.is_empty() {
            lines.push("# Changes not staged for commit:".into());
            for e in &unstaged {
                lines.push(format!("#\t{}{}", e.kind.label(), e.path));
            }
            lines.push("#".into());
        }
        if !untracked.is_empty() {
            lines.push("# Untracked files:".into());
            for e in &untracked {
                lines.push(format!("#\t{}", e.path));
            }
            lines.push("#".into());
        }
        Ok(lines.join("\n"))
    }
}

// -- free helpers ------------------------------------------------------------

/// Return the submodule paths in `repo` whose in-tree HEAD differs from
/// the index entry - i.e. the pointer is dirty and wants committing.
pub fn changed_submodule_paths(repo: &Repo) -> Result<Vec<String>> {
    repo.changed_submodule_paths()
}
