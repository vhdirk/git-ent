//! Tests for `sgit push`.

mod common;

use sgit::commands;
use std::path::Path;

#[test]
/// Test case for no remote does nothing.
///
fn no_remote_does_nothing() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || commands::push::run(&[])).unwrap();
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
    common::in_cwd(&repo, || commands::push::run(&[])).unwrap();
}

#[test]
/// Test case for nothing to push when up to date.
///
fn nothing_to_push_when_up_to_date() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = repo_with_remote(tmp.path());
    common::git(&repo, &["push", "-q"]);
    common::in_cwd(&repo, || commands::push::run(&[])).unwrap();
}

#[test]
/// Test case for push with options forwarding.
fn pushes_with_options() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = repo_with_remote(tmp.path());
    common::in_cwd(&repo, || commands::push::run(&["ci.skip".to_string()])).unwrap();
}

#[test]
/// Test case for push capturing merge request notices and URLs from remote hook.
fn push_captures_merge_request_notices() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().unwrap();
    let bare = tmp.path().join("bare.git");
    std::process::Command::new("git")
        .args(["init", "--bare", "-q", &bare.display().to_string()])
        .status()
        .unwrap();

    let hook_path = bare.join("hooks").join("pre-receive");
    std::fs::create_dir_all(bare.join("hooks")).unwrap();
    std::fs::write(
        &hook_path,
        "#!/bin/sh\n\
         echo \"To create a merge request for branch, visit:\"\n\
         echo \"  https://gitlab.example.com/org/repo/-/merge_requests/new?branch=main\"\n\
         exit 0\n",
    )
    .unwrap();
    let mut perms = std::fs::metadata(&hook_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&hook_path, perms).unwrap();

    let repo = tmp.path().join("work");
    common::make_repo(&repo, &[("f.txt", "initial\n")]);
    common::git(
        &repo,
        &["remote", "add", "origin", &bare.display().to_string()],
    );
    std::fs::write(repo.join("f.txt"), "changed\n").unwrap();
    common::git(&repo, &["add", "f.txt"]);
    common::git(&repo, &["commit", "-q", "-m", "second"]);

    let r = sgit::Repo::discover(&repo).unwrap();
    let cmd = sgit::git::push::PushCmd {
        remote: Some("origin".to_string()),
        branch: Some("main".to_string()),
        set_upstream: true,
        ..Default::default()
    };
    let output = r.git(&cmd).unwrap();

    assert_eq!(
        output.merge_request_urls,
        vec!["https://gitlab.example.com/org/repo/-/merge_requests/new?branch=main"]
    );
    assert!(
        output
            .notices
            .iter()
            .any(|n| n.contains("To create a merge request"))
    );
    assert!(!output.ref_updates.is_empty());
}
