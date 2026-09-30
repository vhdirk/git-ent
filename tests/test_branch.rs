//! Tests for `sgit branch`.

mod common;
use sgit::commands;

#[test]
/// Test case for creates branch in plain repo.
///
fn creates_branch_in_plain_repo() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || commands::branch::run(Some("feature-a"))).unwrap();

    let out = common::git_out(&p.path, &["branch"]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("feature-a"));
}

#[test]
/// Test case for creates branch across submodules.
///
fn creates_branch_across_submodules() {
    let r = common::repo_with_submodules();
    common::in_cwd(&r.main, || commands::branch::run(Some("feature-b"))).unwrap();

    let root_branches =
        String::from_utf8_lossy(&common::git_out(&r.main, &["branch"]).stdout).to_string();
    let sub1_branches =
        String::from_utf8_lossy(&common::git_out(&r.main.join("sub1"), &["branch"]).stdout)
            .to_string();
    let sub2_branches =
        String::from_utf8_lossy(&common::git_out(&r.main.join("sub2"), &["branch"]).stdout)
            .to_string();
    assert!(root_branches.contains("feature-b"));
    assert!(sub1_branches.contains("feature-b"));
    assert!(sub2_branches.contains("feature-b"));
}

#[test]
/// Test case for creates branch nested.
///
fn creates_branch_nested() {
    let r = common::nested_submodules();
    common::in_cwd(&r.main, || commands::branch::run(Some("deep-branch"))).unwrap();

    let root_branches =
        String::from_utf8_lossy(&common::git_out(&r.main, &["branch"]).stdout).to_string();
    let mid_branches =
        String::from_utf8_lossy(&common::git_out(&r.main.join("mid"), &["branch"]).stdout)
            .to_string();
    let leaf_branches =
        String::from_utf8_lossy(&common::git_out(&r.main.join("mid/leaf"), &["branch"]).stdout)
            .to_string();
    assert!(root_branches.contains("deep-branch"));
    assert!(mid_branches.contains("deep-branch"));
    assert!(leaf_branches.contains("deep-branch"));
}

#[test]
/// Test case for duplicate branch is idempotent.
///
fn duplicate_branch_is_idempotent() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || commands::branch::run(Some("dup"))).unwrap();
    common::in_cwd(&p.path, || commands::branch::run(Some("dup"))).unwrap();
}

#[test]
/// Test case for lists branches plain.
///
fn lists_branches_plain() {
    let p = common::plain_repo();
    common::git(&p.path, &["branch", "other"]);
    common::in_cwd(&p.path, || commands::branch::run(None)).unwrap();
}

#[test]
/// Test case for lists branches with submodules.
///
fn lists_branches_with_submodules() {
    let r = common::repo_with_submodules();
    common::in_cwd(&r.main, || commands::branch::run(None)).unwrap();
}

#[test]
/// Test case for long create flag.
fn long_create_flag() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || commands::branch::run(Some("long-flag"))).unwrap();
}

#[test]
/// Test case for branch name with slash like feature/topic.
fn creates_and_lists_branch_with_slashes() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || commands::branch::run(Some("feature/topic"))).unwrap();

    let out = common::git_out(&p.path, &["branch"]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("feature/topic"));
    common::in_cwd(&p.path, || commands::branch::run(None)).unwrap();
}
