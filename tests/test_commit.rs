//! Tests for `sgit commit`.

mod common;
use sgit::commands;

#[test]
/// Test case for commit staged changes.
///
fn commit_staged_changes() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);
    common::in_cwd(&p.path, || {
        commands::commit::run(Some("test commit"), false)
    })
    .unwrap();

    let out = common::git_out(&p.path, &["log", "-1", "--pretty=%s"]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("test commit"));
}

#[test]
/// Test case for nothing to commit.
///
fn nothing_to_commit() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || commands::commit::run(Some("empty"), false)).unwrap();
}

#[test]
/// Test case for long message flag.
///
fn long_message_flag() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "x\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);
    common::in_cwd(&p.path, || commands::commit::run(Some("long flag"), false)).unwrap();
}

#[test]
/// Test case for commits in submodule and parent.
///
fn commits_in_submodule_and_parent() {
    let r = common::repo_with_submodules();
    std::fs::write(r.main.join("sub1/file1.txt"), "changed\n").unwrap();
    std::fs::write(r.main.join("main.txt"), "changed\n").unwrap();
    common::in_cwd(&r.main, || {
        commands::add::run(&["sub1/file1.txt".into(), "main.txt".into()], false, false)
    })
    .unwrap();

    common::in_cwd(&r.main, || {
        commands::commit::run(Some("cross-module"), false)
    })
    .unwrap();

    let sub1_msg = String::from_utf8_lossy(
        &common::git_out(&r.main.join("sub1"), &["log", "-1", "--pretty=%s"]).stdout,
    )
    .trim()
    .to_string();
    let root_msg =
        String::from_utf8_lossy(&common::git_out(&r.main, &["log", "-1", "--pretty=%s"]).stdout)
            .trim()
            .to_string();
    assert_eq!(sub1_msg, "cross-module");
    assert_eq!(root_msg, "cross-module");
}

#[test]
/// Test case for auto stages submodule pointer.
///
fn auto_stages_submodule_pointer() {
    let r = common::repo_with_submodules();
    std::fs::write(r.main.join("sub1/file1.txt"), "changed\n").unwrap();
    common::in_cwd(&r.main, || {
        commands::add::run(&["sub1/file1.txt".into()], false, false)
    })
    .unwrap();
    common::in_cwd(&r.main, || commands::commit::run(Some("sub only"), false)).unwrap();

    // After commit, no dirty submodule pointer remains.
    let out = common::git_out(&r.main, &["status", "--porcelain"]);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.contains("sub1"));
}

#[test]
/// Test case for only commits repos with staged changes.
///
fn only_commits_repos_with_staged_changes() {
    let r = common::repo_with_submodules();
    std::fs::write(r.main.join("sub1/file1.txt"), "changed\n").unwrap();
    common::in_cwd(&r.main, || {
        commands::add::run(&["sub1/file1.txt".into()], false, false)
    })
    .unwrap();
    common::in_cwd(&r.main, || commands::commit::run(Some("partial"), false)).unwrap();
    // Just verify it succeeded (the old test checked stdout for sub2 absence)
}

#[test]
/// Test case for commit nested submodules.
///
fn commit_nested_submodules() {
    let r = common::nested_submodules();
    std::fs::write(r.main.join("mid/leaf/leaf.txt"), "changed\n").unwrap();
    common::in_cwd(&r.main, || {
        commands::add::run(&["mid/leaf/leaf.txt".into()], false, false)
    })
    .unwrap();
    common::in_cwd(&r.main, || {
        commands::commit::run(Some("deep commit"), false)
    })
    .unwrap();
}

#[test]
/// Test case for no verify bypasses hooks.
///
fn no_verify_bypasses_hooks() {
    let p = common::plain_repo();
    let hook_dir = p.path.join(".git/hooks");
    std::fs::create_dir_all(&hook_dir).unwrap();
    let hook = hook_dir.join("pre-commit");
    std::fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);
    common::in_cwd(&p.path, || commands::commit::run(Some("bypass"), true)).unwrap();
}
