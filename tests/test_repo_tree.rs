//! Tests for the RepoTree domain model.

mod common;

use std::path::Path;

use sgit::RepoTree;

/// Discover from `path` by setting cwd. Tests don't run in parallel
/// within a binary by default, but each test-bin does - using explicit
/// `RepoTree::discover(Some(path))` avoids the race.
fn tree_at(path: &Path) -> RepoTree {
    RepoTree::discover(Some(path)).unwrap()
}

#[test]
/// Test case for finds repo from explicit path.
///
fn finds_repo_from_explicit_path() {
    let p = common::plain_repo();
    let tree = tree_at(&p.path);
    assert_eq!(
        std::fs::canonicalize(&tree.root.workdir).unwrap(),
        std::fs::canonicalize(&p.path).unwrap()
    );
}

#[test]
/// Test case for finds repo from subdir.
///
fn finds_repo_from_subdir() {
    let p = common::plain_repo();
    let subdir = p.path.join("subdir");
    std::fs::create_dir(&subdir).unwrap();
    let tree = tree_at(&subdir);
    assert_eq!(
        std::fs::canonicalize(&tree.root.workdir).unwrap(),
        std::fs::canonicalize(&p.path).unwrap()
    );
}

#[test]
/// Test case for no submodules on plain repo.
///
fn no_submodules_on_plain_repo() {
    let p = common::plain_repo();
    let tree = tree_at(&p.path);
    assert!(tree.submodules.is_empty());
}

#[test]
/// Test case for flat submodules.
///
fn flat_submodules() {
    let r = common::repo_with_submodules();
    let tree = tree_at(&r.main);
    let names: std::collections::HashSet<String> = tree
        .submodules
        .iter()
        .map(|s| {
            s.workdir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert!(names.contains("sub1"));
    assert!(names.contains("sub2"));
}

#[test]
/// Test case for nested submodules depth first.
///
fn nested_submodules_depth_first() {
    let r = common::nested_submodules();
    let tree = tree_at(&r.main);
    let names: Vec<String> = tree
        .submodules
        .iter()
        .map(|s| {
            s.workdir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(names, vec!["leaf", "mid"]);
}

#[test]
/// Root config can exclude a nested submodule via full path from root.
fn root_config_excludes_nested_submodule_by_full_path() {
    let r = common::nested_submodules();
    std::fs::write(r.main.join(".sgit.toml"), "exclude = [\"mid/leaf\"]\n").unwrap();

    let tree = tree_at(&r.main);
    let names: Vec<String> = tree
        .submodules
        .iter()
        .map(|s| {
            s.workdir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    assert_eq!(names, vec!["mid"]);
}

#[test]
/// Submodule config can exclude nested submodules via paths from submodule root.
fn submodule_config_excludes_nested_submodule_by_full_path() {
    let r = common::nested_submodules();
    std::fs::write(r.main.join("mid/.sgit.toml"), "exclude = [\"leaf\"]\n").unwrap();

    let tree = tree_at(&r.main);
    let names: Vec<String> = tree
        .submodules
        .iter()
        .map(|s| {
            s.workdir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    assert_eq!(names, vec!["mid"]);
}

#[test]
/// Test case for all includes root last.
///
fn all_includes_root_last() {
    let r = common::repo_with_submodules();
    let tree = tree_at(&r.main);
    let all = tree.all();
    assert_eq!(all.len(), 3);
    assert_eq!(
        std::fs::canonicalize(&all.last().unwrap().workdir).unwrap(),
        std::fs::canonicalize(&tree.root.workdir).unwrap(),
    );
}

#[test]
/// Test case for label top level is dot.
///
fn label_top_level_is_dot() {
    let p = common::plain_repo();
    let tree = tree_at(&p.path);
    assert_eq!(tree.root.label(), ".");
}

#[test]
/// Test case for resolve file routes to submodule.
///
fn resolve_file_routes_to_submodule() {
    let r = common::repo_with_submodules();
    std::fs::write(r.main.join("sub1/new.txt"), "x\n").unwrap();
    let tree = tree_at(&r.main);
    let (repo, rel) = tree.resolve_file_from(&r.main, "sub1/new.txt").unwrap();
    assert_eq!(repo.workdir.file_name().unwrap().to_string_lossy(), "sub1");
    assert_eq!(rel, std::path::PathBuf::from("new.txt"));
}

#[test]
/// Test case for resolve file routes to toplevel.
///
fn resolve_file_routes_to_toplevel() {
    let r = common::repo_with_submodules();
    let tree = tree_at(&r.main);
    let (repo, _rel) = tree.resolve_file_from(&r.main, "main.txt").unwrap();
    assert_eq!(
        std::fs::canonicalize(&repo.workdir).unwrap(),
        std::fs::canonicalize(&tree.root.workdir).unwrap(),
    );
}

#[test]
/// Test case for list status detects changes.
///
fn list_status_detects_changes() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    let tree = tree_at(&p.path);
    let (_s, u, _ut) = tree.root.list_status().unwrap();
    assert!(!u.is_empty());
    assert!(tree.root.has_changes().unwrap());
}

#[test]
/// Git executable configuration is global and cannot be set per repo.
fn git_executable_is_global_not_per_repo() {
    let r = common::repo_with_submodules();
    let tmp = tempfile::tempdir().unwrap();
    let global_git_bin = tmp.path().join("custom-global-git");
    let local_git_bin = tmp.path().join("custom-local-git");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(&global_git_bin, "#!/bin/sh\nexec git \"$@\"\n").unwrap();
        std::fs::set_permissions(&global_git_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(&local_git_bin, "#!/bin/sh\nexec git \"$@\"\n").unwrap();
        std::fs::set_permissions(&local_git_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    // Attempting to set git in repo-local .sgit.toml files must be ignored.
    std::fs::write(
        r.main.join(".sgit.toml"),
        format!("git = \"{}\"\n", local_git_bin.display()),
    )
    .unwrap();
    std::fs::write(
        r.main.join("sub2/.sgit.toml"),
        format!("git = \"{}\"\n", local_git_bin.display()),
    )
    .unwrap();

    let global_file = tmp.path().join("sgit.toml");
    std::fs::write(
        &global_file,
        format!("git = \"{}\"\n", global_git_bin.display()),
    )
    .unwrap();

    let tree = RepoTree::discover_with_global(Some(&r.main), Some(&global_file)).unwrap();
    assert_eq!(tree.root.git.executable, global_git_bin.as_os_str());

    let sub1_path = std::fs::canonicalize(r.main.join("sub1")).unwrap();
    let sub2_path = std::fs::canonicalize(r.main.join("sub2")).unwrap();

    let sub1 = tree
        .submodules
        .iter()
        .find(|s| s.workdir == sub1_path)
        .unwrap();
    assert_eq!(sub1.git.executable, global_git_bin.as_os_str());

    let sub2 = tree
        .submodules
        .iter()
        .find(|s| s.workdir == sub2_path)
        .unwrap();
    // Sub2 must also use global_git_bin, ignoring the local .sgit.toml attempt
    assert_eq!(sub2.git.executable, global_git_bin.as_os_str());
}

#[test]
/// Uninitialized submodules are omitted from discovered tree.
fn uninitialized_submodule_is_omitted() {
    let r = common::repo_with_submodules();
    // De-initialize sub1 so its worktree .git is removed.
    common::git(&r.main, &["submodule", "deinit", "-f", "sub1"]);

    let tree = tree_at(&r.main);
    let names: Vec<String> = tree
        .submodules
        .iter()
        .map(|s| {
            s.workdir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    assert!(!names.contains(&"sub1".to_string()));
    assert!(names.contains(&"sub2".to_string()));
}

#[test]
/// load_root_config_from discovers the outermost root repo even from deep submodules.
fn load_root_config_from_discovers_outermost_root() {
    let r = common::nested_submodules();
    std::fs::write(
        r.main.join(".sgit.toml"),
        "[alias]\ncustom-alias = \"status -s\"\n",
    )
    .unwrap();

    let deep_leaf = r.main.join("mid/leaf");
    let cfg = sgit::config::load_root_config_from(&deep_leaf);
    assert_eq!(
        cfg.expand_alias("custom-alias"),
        Some(vec!["status".into(), "-s".into()])
    );
}
