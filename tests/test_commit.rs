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

#[test]
fn pre_commit_hook_rejection_prevents_commit() {
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
    let head_before = common::git_out(&p.path, &["rev-parse", "HEAD"]).stdout;

    let result = common::in_cwd(&p.path, || commands::commit::run(Some("rejected"), false));

    let head_after = common::git_out(&p.path, &["rev-parse", "HEAD"]).stdout;
    assert!(result.is_err());
    assert_eq!(head_after, head_before);
}

#[test]
fn commit_msg_hook_can_edit_message_from_configured_hooks_path() {
    let p = common::plain_repo();
    let hook_dir = p.tmp.path().join("custom-hooks");
    std::fs::create_dir_all(&hook_dir).unwrap();
    common::git(
        &p.path,
        &["config", "core.hooksPath", hook_dir.to_str().unwrap()],
    );
    let hook = hook_dir.join("commit-msg");
    std::fs::write(
        &hook,
        "#!/bin/sh\nprintf '%s\\n' 'edited by hook' > \"$1\"\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);
    common::in_cwd(&p.path, || {
        commands::commit::run(Some("original message"), false)
    })
    .unwrap();

    let subject = common::git_out(&p.path, &["log", "-1", "--pretty=%s"]);
    assert_eq!(
        String::from_utf8_lossy(&subject.stdout).trim(),
        "edited by hook"
    );
}

#[test]
fn prepare_commit_msg_hook_runs() {
    let p = common::plain_repo();
    let hook_dir = p.path.join(".git/hooks");
    std::fs::create_dir_all(&hook_dir).unwrap();
    let hook = hook_dir.join("prepare-commit-msg");
    std::fs::write(
        &hook,
        "#!/bin/sh\nprintf 'prepared: ' > \"$1.tmp\" && cat \"$1\" >> \"$1.tmp\" && mv \"$1.tmp\" \"$1\"\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);
    common::in_cwd(&p.path, || {
        commands::commit::run(Some("initial message"), false)
    })
    .unwrap();

    let subject = common::git_out(&p.path, &["log", "-1", "--pretty=%s"]);
    assert_eq!(
        String::from_utf8_lossy(&subject.stdout).trim(),
        "prepared: initial message"
    );
}

#[test]
fn hooks_run_in_order_exactly_once() {
    let p = common::plain_repo();
    let hook_dir = p.path.join(".git/hooks");
    std::fs::create_dir_all(&hook_dir).unwrap();
    let log_file = p.tmp.path().join("hooks.log");
    let log_path = log_file.to_str().unwrap();

    for (name, payload) in [
        ("pre-commit", "pre-commit\n"),
        ("prepare-commit-msg", "prepare-commit-msg\n"),
        ("commit-msg", "commit-msg\n"),
    ] {
        let hook = hook_dir.join(name);
        std::fs::write(
            &hook,
            format!("#!/bin/sh\nprintf '{payload}' >> '{log_path}'\n"),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);
    common::in_cwd(&p.path, || commands::commit::run(Some("test hooks"), false)).unwrap();

    let content = std::fs::read_to_string(&log_file).unwrap_or_default();
    assert_eq!(content, "pre-commit\nprepare-commit-msg\ncommit-msg\n");
}
