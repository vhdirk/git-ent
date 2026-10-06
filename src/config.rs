//! git-nest configuration loading.
//!
//! Two config files are consulted for every repo in the tree, merged in order:
//!
//! 1. **Global** - `$XDG_CONFIG_HOME/git-nest/git-nest.toml` (typically
//!    `~/.config/git-nest/git-nest.toml`). Exclusions here apply to every repo.
//! 2. **Repo-local** - `.git-nest.toml` at the root of each individual repo.
//!    Exclusions here apply only to that repo's direct submodules.
//!
//! Example `.git-nest.toml` / `git-nest.toml`:
//! ```toml
//! exclude = ["vendor/some-lib", "third_party/other"]
//! ```

use std::path::Path;

use serde::Deserialize;

pub const REPO_CONFIG_FILE: &str = ".git-nest.toml";
pub const GLOBAL_CONFIG_DIR: &str = "git-nest";
pub const GLOBAL_CONFIG_FILE: &str = "git-nest.toml";
pub const GLOBAL_CONFIG_FILE_PATH: &str = "git-nest/git-nest.toml";

/// Raw deserialization target - one source file.
#[derive(Debug, Default, Deserialize)]
struct RawConfig {
    #[serde(default)]
    exclude: Vec<String>,
}

impl RawConfig {
    /// Parse one TOML config file into raw config values.
    ///
    /// - `path`: file path to read and parse.
    fn from_file(path: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        match toml::from_str(&text) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Warning: failed to parse {}: {e}", path.display());
                None
            }
        }
    }
}

/// Merged git-nest configuration for a single repository.
///
/// Built by combining the global config (XDG) with the repo-local `.git-nest.toml`.
/// Exclusions from both sources are unioned.
#[derive(Debug, Default)]
pub struct GitNestConfig {
    /// Submodule paths (relative to this repo's workdir) to exclude from all
    /// git-nest operations. Paths use forward slashes on all platforms.
    pub exclude: Vec<String>,
}

impl GitNestConfig {
    /// Load and merge the global config + `repo_workdir/.git-nest.toml`.
    pub fn load(repo_workdir: &Path) -> Self {
        Self::load_with_global(repo_workdir, global_config_path().as_deref())
    }

    /// Like [`load`] but with an explicit global config path, used in tests.
    fn load_with_global(repo_workdir: &Path, global_path: Option<&Path>) -> Self {
        let mut exclude = Vec::new();

        // 1. Global config
        if let Some(path) = global_path {
            if let Some(raw) = RawConfig::from_file(path) {
                exclude.extend(raw.exclude);
            }
        }

        // 2. Repo-local config (local aliases override global ones)
        let local_path = repo_workdir.join(REPO_CONFIG_FILE);
        if let Some(raw) = RawConfig::from_file(&local_path) {
            for entry in raw.exclude {
                if !exclude.contains(&entry) {
                    exclude.push(entry);
                }
            }
        }

        Self { exclude }
    }

    /// Returns `true` if `submodule_path` (relative to this repo's workdir)
    /// is excluded by this config.
    pub fn is_excluded(&self, submodule_path: &Path) -> bool {
        let path_str = submodule_path.to_slash_lossy();
        self.exclude.iter().any(|ex| ex == path_str.as_ref())
    }
}

/// Load aliases from the root repo that contains `cwd`.
///
/// Walks up from `cwd` using libgit2 to find the outermost git repository,
/// then loads its config (merged with the global config). Returns an empty
/// config if no git repo is found or any I/O error occurs.
pub fn load_root_config() -> GitNestConfig {
    // Find outermost git root by repeatedly discovering from the parent.
    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(_) => return GitNestConfig::default(),
    };
    let mut root = cwd.clone();
    let mut search = cwd.as_path();
    while let Ok(repo) = git2::Repository::discover(search) {
        if let Some(wd) = repo.workdir() {
            root = wd.to_path_buf();
        }
        match search.parent() {
            Some(p) => search = p,
            None => break,
        }
    }

    GitNestConfig::load(&root)
}

/// Resolve `$XDG_CONFIG_HOME/git-nest/git-nest.toml`, falling back to
/// `~/.config/git-nest/git-nest.toml` when `XDG_CONFIG_HOME` is not set.
pub fn global_config_path() -> Option<std::path::PathBuf> {
    xdg::BaseDirectories::with_prefix(GLOBAL_CONFIG_DIR)
        .get_config_home()
        .map(|p| p.join(GLOBAL_CONFIG_FILE))
}

// -- platform-neutral path-to-string helper ----------------------------------

trait ToSlashLossy<'a> {
    /// Convert a path to slash-separated text for config matching.
    fn to_slash_lossy(&'a self) -> std::borrow::Cow<'a, str>;
}

impl<'a> ToSlashLossy<'a> for Path {
    /// Return a slash-normalized string representation of `self`.
    fn to_slash_lossy(&'a self) -> std::borrow::Cow<'a, str> {
        #[cfg(windows)]
        {
            let s = self.to_string_lossy().replace('\\', "/");
            std::borrow::Cow::Owned(s)
        }
        #[cfg(not(windows))]
        {
            self.to_string_lossy()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Write a repo-local config file used by unit tests.
    ///
    /// - `dir`: repository root where `.git-nest.toml` is written.
    /// - `content`: TOML body written into the config file.
    fn write_config(dir: &Path, content: &str) {
        fs::write(dir.join(REPO_CONFIG_FILE), content).unwrap();
    }

    #[test]
    /// Ensures missing local/global files produce an empty merged config.
    fn missing_files_give_default() {
        let dir = tempdir().unwrap();
        let cfg = GitNestConfig::load_with_global(dir.path(), None);
        assert!(cfg.exclude.is_empty());
    }

    #[test]
    /// Ensures local config parsing populates the `exclude` list.
    fn parses_local_exclude_list() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"exclude = ["vendor/lib", "third_party/other"]"#,
        );
        let cfg = GitNestConfig::load_with_global(dir.path(), None);
        assert_eq!(cfg.exclude, vec!["vendor/lib", "third_party/other"]);
    }

    #[test]
    /// Ensures global and local excludes are both present after merge.
    fn global_and_local_are_merged() {
        let tmp = tempdir().unwrap();
        let global_file = tmp.path().join(GLOBAL_CONFIG_FILE);
        fs::write(&global_file, r#"exclude = ["global/dep"]"#).unwrap();

        let repo_dir = tempdir().unwrap();
        write_config(repo_dir.path(), r#"exclude = ["local/dep"]"#);

        let cfg = GitNestConfig::load_with_global(repo_dir.path(), Some(&global_file));
        assert!(cfg.exclude.contains(&"global/dep".to_string()));
        assert!(cfg.exclude.contains(&"local/dep".to_string()));
    }

    #[test]
    /// Ensures duplicate excludes from both sources are deduplicated.
    fn duplicates_are_not_repeated() {
        let tmp = tempdir().unwrap();
        let global_file = tmp.path().join(GLOBAL_CONFIG_FILE);
        fs::write(&global_file, r#"exclude = ["shared/dep"]"#).unwrap();

        let repo_dir = tempdir().unwrap();
        write_config(repo_dir.path(), r#"exclude = ["shared/dep"]"#);

        let cfg = GitNestConfig::load_with_global(repo_dir.path(), Some(&global_file));
        assert_eq!(cfg.exclude.iter().filter(|e| *e == "shared/dep").count(), 1);
    }

    #[test]
    /// Ensures exclusion matching compares normalized relative paths.
    fn is_excluded_matches_path() {
        let cfg = GitNestConfig {
            exclude: vec!["vendor/lib".to_string()],
            ..Default::default()
        };
        assert!(cfg.is_excluded(Path::new("vendor/lib")));
        assert!(!cfg.is_excluded(Path::new("vendor/other")));
    }
}
