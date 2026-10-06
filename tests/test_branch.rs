mod common;
use git_nest::cli::branch::BranchCmd;

#[test]
/// Test case for creates branch in plain repo.
///
fn creates_branch_in_plain_repo() {
    let p = common::plain_repo();
    common::in_cwd(
        &p.path,
        BranchCmd {
            create: Some("feature-a".into()),
        },
    )
    .unwrap();

    let out = common::git_out(&p.path, &["branch"]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("feature-a"));
}

#[test]
/// Test case for creates branch across submodules.
///
fn creates_branch_across_submodules() {
    let r = common::repo_with_submodules();
    common::in_cwd(
        &r.main,
        BranchCmd {
            create: Some("feature-b".into()),
        },
    )
    .unwrap();

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
    common::in_cwd(
        &r.main,
        BranchCmd {
            create: Some("deep-branch".into()),
        },
    )
    .unwrap();

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
    common::in_cwd(
        &p.path,
        BranchCmd {
            create: Some("dup".into()),
        },
    )
    .unwrap();
    common::in_cwd(
        &p.path,
        BranchCmd {
            create: Some("dup".into()),
        },
    )
    .unwrap();
}

#[test]
/// Test case for lists branches plain.
///
fn lists_branches_plain() {
    let p = common::plain_repo();
    common::git(&p.path, &["branch", "other"]);
    common::in_cwd(&p.path, BranchCmd { create: None }).unwrap();
}

#[test]
/// Test case for lists branches with submodules.
///
fn lists_branches_with_submodules() {
    let r = common::repo_with_submodules();
    common::in_cwd(&r.main, BranchCmd { create: None }).unwrap();
}

#[test]
/// Test case for long create flag.
///
fn long_create_flag() {
    let p = common::plain_repo();
    common::in_cwd(
        &p.path,
        BranchCmd {
            create: Some("long-flag".into()),
        },
    )
    .unwrap();
}
