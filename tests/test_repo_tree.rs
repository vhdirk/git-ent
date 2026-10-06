//! Tests for the RepoTree domain model.

mod common;

use std::path::{Path, PathBuf};

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
    let (repo, rel) = tree
        .resolve_file_from(&r.main, &PathBuf::from("sub1/new.txt"))
        .unwrap();
    assert_eq!(repo.workdir.file_name().unwrap().to_string_lossy(), "sub1");
    assert_eq!(rel, std::path::PathBuf::from("new.txt"));
}

#[test]
/// Test case for resolve file routes to toplevel.
///
fn resolve_file_routes_to_toplevel() {
    let r = common::repo_with_submodules();
    let tree = tree_at(&r.main);
    let (repo, _rel) = tree
        .resolve_file_from(&r.main, &PathBuf::from("main.txt"))
        .unwrap();
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
    let status = tree.root.status().unwrap();
    assert!(!status.unstaged.is_empty());
    assert!(tree.root.has_changes().unwrap());
}
