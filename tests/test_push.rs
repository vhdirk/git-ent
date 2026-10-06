//! Tests for `sgit push`.

mod common;

use sgit::cmd::push::PushCmd;
use std::path::Path;

#[test]
/// Test case for no remote does nothing.
///
fn no_remote_does_nothing() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, PushCmd::default()).unwrap();
}

/// Test case for repo with remote.
///
/// - `base`: test input or fixture parameter.
fn repo_with_remote(base: &Path) -> std::path::PathBuf {
    let bare = base.join("bare.git");
    std::process::Command::new("git")
        .args(["init", "--bare", "-q", &bare.display().to_string()])
        .status()
        .unwrap();

    let repo = base.join("work");
    common::make_repo(&repo, &[("f.txt", "initial\n")]);
    common::git(
        &repo,
        &["remote", "add", "origin", &bare.display().to_string()],
    );
    common::git(&repo, &["push", "-q", "--set-upstream", "origin", "main"]);
    std::fs::write(repo.join("f.txt"), "changed\n").unwrap();
    common::git(&repo, &["add", "f.txt"]);
    common::git(&repo, &["commit", "-q", "-m", "second"]);
    repo
}

#[test]
/// Test case for pushes ahead commits.
///
fn pushes_ahead_commits() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = repo_with_remote(tmp.path());
    common::in_cwd(&repo, PushCmd::default()).unwrap();
}

#[test]
/// Test case for nothing to push when up to date.
///
fn nothing_to_push_when_up_to_date() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = repo_with_remote(tmp.path());
    common::git(&repo, &["push", "-q"]);
    common::in_cwd(&repo, PushCmd::default()).unwrap();
}
