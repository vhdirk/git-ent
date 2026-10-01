//! sgit configuration loading.
//!
//! Two config files are consulted for every repo in the tree, merged in order:
//!
//! 1. **Global** - `$XDG_CONFIG_HOME/sgit/sgit.toml` (typically
//!    `~/.config/sgit/sgit.toml`). Exclusions here apply to every repo.
//! 2. **Repo-local** - `.sgit.toml` at the root of each individual repo.
//!    Exclusions here apply only to that repo's direct submodules.
//!
//! Example `.sgit.toml` / `sgit.toml`:
//! ```toml
//! exclude = ["vendor/some-lib", "third_party/other"]
//! [alias]
//! deploy = "push -o env=production"
//! pushmr = "push -o merge_request.create -o merge_request.remove_source_branch
//! ```

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::Repo;

pub const REPO_CONFIG_FILE: &str = ".sgit.toml";
pub const GLOBAL_CONFIG_DIR: &str = "sgit";
pub const GLOBAL_CONFIG_FILE: &str = "sgit.toml";
pub const GLOBAL_CONFIG_FILE_PATH: &str = "sgit/sgit.toml";

/// Raw deserialization target - one source file.
#[derive(Debug, Default, Deserialize)]
struct RawConfig {
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    alias: HashMap<String, String>,
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

/// Merged sgit configuration for a single repository.
///
/// Built by combining the global config (XDG) with the repo-local `.sgit.toml`.
/// Exclusions from both sources are unioned.
#[derive(Debug)]
pub struct SgitConfig {
    /// Submodule paths (relative to this repo's workdir) to exclude from all
    /// sgit operations. Paths use forward slashes on all platforms.
    pub exclude: Vec<String>,
    /// Command aliases: maps an alias name to an argument string that is
    /// shell-word-split and prepended when the alias is invoked.
    ///
    /// Aliases from the global config and the **root** repo `.sgit.toml` are
    /// merged (local overrides global). Aliases from submodule configs are
    /// never loaded.
    pub aliases: HashMap<String, String>,
}

impl Default for SgitConfig {
    fn default() -> Self {
        Self {
            exclude: Vec::new(),
            aliases: HashMap::new(),
        }
    }
}

impl SgitConfig {
    /// Load and merge the global config + `repo_workdir/.sgit.toml`.
    pub fn merge(&mut self, path: &Path) {
        if let Some(raw) = RawConfig::from_file(path) {
            for entry in raw.exclude {
                if !self.exclude.contains(&entry) {
                    self.exclude.push(entry);
                }
            }
            self.aliases.extend(raw.alias);
        }
    }

    pub fn load(repo_dir: &Path) -> Self {
        Self::load_with_global(repo_dir, global_config_path().as_deref())
    }

    /// Like [`load`] but with an explicit global config path, used in tests.
    fn load_with_global(repo_dir: &Path, global_path: Option<&Path>) -> Self {
        let mut config = SgitConfig::default();

        // first load the global config
        if let Some(global_path) = global_path {
            config.merge(global_path);
        }

        // then override with
        let local_path = repo_dir.join(REPO_CONFIG_FILE);
        config.merge(&local_path);
        config
    }

    /// Returns `true` if `submodule_path` (relative to this repo's workdir)
    /// is excluded by this config.
    pub fn is_excluded(&self, submodule_path: &Path) -> bool {
        let path_str = submodule_path.to_slash_lossy();
        self.exclude.iter().any(|ex| ex == path_str.as_ref())
    }

    /// If `name` is a known alias, return the expansion split into words.
    /// Returns `None` if `name` is not an alias.
    pub fn expand_alias(&self, name: &str) -> Option<Vec<String>> {
        let expansion = self.aliases.get(name)?;
        shell_words::split(expansion).ok()
    }
}

/// Load aliases from the root repo that contains `cwd`.
///
/// Walks up from `cwd` using Git to find the outermost git repository,
/// then loads its config (merged with the global config). Returns an empty
/// config if no git repo is found or any I/O error occurs.
pub fn load_root_config() -> SgitConfig {
    // Find outermost git root by repeatedly discovering from the parent.
    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(_) => return SgitConfig::default(),
    };
    let mut root = cwd.clone();
    let mut search = cwd.as_path();
    while let Ok(repo) = Repo::discover(search) {
        root = repo.workdir.clone();
        match search.parent() {
            Some(p) => search = p,
            None => break,
        }
    }

    SgitConfig::load(&root)
}

/// Resolve `$XDG_CONFIG_HOME/sgit/sgit.toml`, falling back to
/// `~/.config/sgit/sgit.toml` when `XDG_CONFIG_HOME` is not set.
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
    /// - `dir`: repository root where `.sgit.toml` is written.
    /// - `content`: TOML body written into the config file.
    fn write_config(dir: &Path, content: &str) {
        fs::write(dir.join(REPO_CONFIG_FILE), content).unwrap();
    }

    #[test]
    /// Ensures missing local/global files produce an empty merged config.
    fn missing_files_give_default() {
        let dir = tempdir().unwrap();
        let cfg = SgitConfig::load_with_global(dir.path(), None);
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
        let cfg = SgitConfig::load_with_global(dir.path(), None);
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

        let cfg = SgitConfig::load_with_global(repo_dir.path(), Some(&global_file));
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

        let cfg = SgitConfig::load_with_global(repo_dir.path(), Some(&global_file));
        assert_eq!(cfg.exclude.iter().filter(|e| *e == "shared/dep").count(), 1);
    }

    #[test]
    /// Ensures exclusion matching compares normalized relative paths.
    fn is_excluded_matches_path() {
        let cfg = SgitConfig {
            exclude: vec!["vendor/lib".to_string()],
            ..Default::default()
        };
        assert!(cfg.is_excluded(Path::new("vendor/lib")));
        assert!(!cfg.is_excluded(Path::new("vendor/other")));
    }

    #[test]
    /// Alias defined in local config is expanded correctly.
    fn alias_from_local_config() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"[alias]
pushmr = "push -o merge_request.create -o merge_request.remove_source_branch -o merge_request.merge_when_pipeline_succeeds"
"#,
        );
        let cfg = SgitConfig::load_with_global(dir.path(), None);
        assert_eq!(
            cfg.expand_alias("pushmr"),
            Some(vec![
                "push".to_string(),
                "-o".to_string(),
                "merge_request.create".to_string(),
                "-o".to_string(),
                "merge_request.remove_source_branch".to_string(),
                "-o".to_string(),
                "merge_request.merge_when_pipeline_succeeds".to_string()
            ])
        );
        assert_eq!(cfg.expand_alias("push"), None);
    }

    #[test]
    /// Alias with multiple words is split into a vec of arguments.
    fn alias_multi_word_is_split() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"[alias]
deploy = "push -o env=production"
"#,
        );
        let cfg = SgitConfig::load_with_global(dir.path(), None);
        assert_eq!(
            cfg.expand_alias("deploy"),
            Some(vec![
                "push".to_string(),
                "-o".to_string(),
                "env=production".to_string()
            ])
        );
    }

    #[test]
    /// Local alias overrides a global alias of the same name.
    fn local_alias_overrides_global() {
        let tmp = tempdir().unwrap();
        let global_file = tmp.path().join(GLOBAL_CONFIG_FILE);
        fs::write(
            &global_file,
            r#"[alias]
pushmr = "push -o merge_request.create"
"#,
        )
        .unwrap();

        let repo_dir = tempdir().unwrap();
        write_config(
            repo_dir.path(),
            r#"[alias]
pushmr = "push -o merge_request.remove_source_branch -o merge_request.merge_when_pipeline_succeeds"
"#,
        );

        let cfg = SgitConfig::load_with_global(repo_dir.path(), Some(&global_file));
        assert_eq!(
            cfg.expand_alias("pushmr"),
            Some(vec![
                "push".to_string(),
                "-o".to_string(),
                "merge_request.remove_source_branch".to_string(),
                "-o".to_string(),
                "merge_request.merge_when_pipeline_succeeds".to_string()
            ])
        );
    }

    #[test]
    /// Global alias is visible when no local alias overrides it.
    fn global_alias_available_without_local_override() {
        let tmp = tempdir().unwrap();
        let global_file = tmp.path().join(GLOBAL_CONFIG_FILE);
        fs::write(
            &global_file,
            r#"[alias]
pushmr = "push -o merge_request.create -o merge_request.remove_source_branch -o merge_request.merge_when_pipeline_succeeds"
"#,
        )
        .unwrap();

        let repo_dir = tempdir().unwrap();
        // No local config at all.
        let cfg = SgitConfig::load_with_global(repo_dir.path(), Some(&global_file));
        assert_eq!(
            cfg.expand_alias("pushmr"),
            Some(vec![
                "push".to_string(),
                "-o".to_string(),
                "merge_request.create".to_string(),
                "-o".to_string(),
                "merge_request.remove_source_branch".to_string(),
                "-o".to_string(),
                "merge_request.merge_when_pipeline_succeeds".to_string()
            ])
        );
    }
}
