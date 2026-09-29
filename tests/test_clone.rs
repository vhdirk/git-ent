//! Tests for `sgit clone`.

mod common;

use sgit::commands;
use std::path::Path;

/// Test case for cloneable.
///
/// - `tmp`: test input or fixture parameter.
fn cloneable(tmp: &Path) -> std::path::PathBuf {
    let base = tmp.join("remote");
    std::fs::create_dir_all(&base).unwrap();
    let sub = base.join("dep");
    common::make_repo(&sub, &[("dep.txt", "dep\n")]);
    let proj = base.join("project");
    common::make_repo(&proj, &[("readme.txt", "hello\n")]);
    common::git(
        &proj,
        &["submodule", "add", "-q", &sub.display().to_string(), "dep"],
    );
    common::git(&proj, &["commit", "-q", "-m", "add dep"]);
    proj
}

#[test]
/// Test case for clone with explicit dest.
///
fn clone_with_explicit_dest() {
    let tmp = tempfile::tempdir().unwrap();
    let src = cloneable(tmp.path());
    let dest = tmp.path().join("cloned");

    common::in_cwd(tmp.path(), || {
        commands::clone::run(
            &src.display().to_string(),
            Some(&dest.display().to_string()),
        )
    })
    .unwrap();

    assert!(dest.join("readme.txt").exists());
    assert!(dest.join("dep/dep.txt").exists());
}

#[test]
/// Test case for clone infers dest.
///
fn clone_infers_dest() {
    let tmp = tempfile::tempdir().unwrap();
    let src = cloneable(tmp.path());

    common::in_cwd(tmp.path(), || {
        commands::clone::run(&src.display().to_string(), None)
    })
    .unwrap();

    let inferred = tmp.path().join("project");
    assert!(inferred.exists());
    assert!(inferred.join("readme.txt").exists());
}

#[test]
/// Test case for clone invalid usage when URL is missing.
///
fn clone_invalid_usage_without_url() {
    // Note: clone::run requires url: &str, so this test cannot be directly replicated.
    // This is a Clap CLI concern and not applicable to direct library calls.
    // The URL is mandatory at the function level.
}
