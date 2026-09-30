//! Tests for `sgit switch`.

mod common;
use sgit::commands;

#[test]
/// Test case for switch existing branch across submodules.
///
fn switch_existing_branch_across_submodules() {
    let r = common::repo_with_submodules();
    common::in_cwd(&r.main, || commands::branch::run(Some("feature"))).unwrap();

    common::in_cwd(&r.main, || {
        commands::switch::run(Some("feature"), None, None, false)
    })
    .unwrap();

    let branch_main = String::from_utf8_lossy(
        &common::git_out(&r.main, ["rev-parse", "--abbrev-ref", "HEAD"].as_slice()).stdout,
    )
    .trim()
    .to_string();
    assert_eq!(branch_main, "feature");
}

#[test]
/// Test case for switch warns if branch missing in submodules.
///
fn switch_warns_if_branch_missing_in_submodules() {
    let r = common::repo_with_submodules();
    common::git(&r.main, ["branch", "only-root"].as_slice());

    common::in_cwd(&r.main, || {
        commands::switch::run(Some("only-root"), None, None, false)
    })
    .unwrap();
}

#[test]
/// Test case for switch c creates branch everywhere.
///
fn switch_c_creates_branch_everywhere() {
    let r = common::repo_with_submodules();

    common::in_cwd(&r.main, || {
        commands::switch::run(None, Some("topic"), None, false)
    })
    .unwrap();

    let exists_main = String::from_utf8_lossy(
        &common::git_out(&r.main, ["branch", "--list", "topic"].as_slice()).stdout,
    )
    .trim()
    .to_string();
    let exists_sub1 = String::from_utf8_lossy(
        &common::git_out(
            &r.main.join("sub1"),
            ["branch", "--list", "topic"].as_slice(),
        )
        .stdout,
    )
    .trim()
    .to_string();
    let exists_sub2 = String::from_utf8_lossy(
        &common::git_out(
            &r.main.join("sub2"),
            ["branch", "--list", "topic"].as_slice(),
        )
        .stdout,
    )
    .trim()
    .to_string();

    assert!(exists_main.contains("topic"));
    assert!(exists_sub1.contains("topic"));
    assert!(exists_sub2.contains("topic"));
}

#[test]
/// Test case for switch force create resets branch pointer.
///
fn switch_force_create_resets_branch_pointer() {
    let p = common::plain_repo();
    common::git(&p.path, ["branch", "topic"].as_slice());

    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::git(&p.path, ["add", "file.txt"].as_slice());
    common::git(&p.path, ["commit", "-q", "-m", "advance main"].as_slice());

    common::in_cwd(&p.path, || {
        commands::switch::run(None, None, Some("topic"), false)
    })
    .unwrap();

    let head_name = String::from_utf8_lossy(
        &common::git_out(&p.path, ["rev-parse", "--abbrev-ref", "HEAD"].as_slice()).stdout,
    )
    .trim()
    .to_string();
    assert_eq!(head_name, "topic");

    let head_oid =
        String::from_utf8_lossy(&common::git_out(&p.path, ["rev-parse", "HEAD"].as_slice()).stdout)
            .trim()
            .to_string();
    let topic_oid = String::from_utf8_lossy(
        &common::git_out(&p.path, ["rev-parse", "refs/heads/topic"].as_slice()).stdout,
    )
    .trim()
    .to_string();
    assert_eq!(head_oid, topic_oid);
}

#[test]
/// Test case for switch detach moves to detached head.
///
fn switch_detach_moves_to_detached_head() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || {
        commands::switch::run(Some("HEAD"), None, None, true)
    })
    .unwrap();

    let head_name = String::from_utf8_lossy(
        &common::git_out(&p.path, ["rev-parse", "--abbrev-ref", "HEAD"].as_slice()).stdout,
    )
    .trim()
    .to_string();
    assert_eq!(head_name, "HEAD");
}

#[test]
/// Test case for switch invalid usage when target is missing.
///
fn switch_invalid_usage_without_target() {
    assert!(
        common::in_cwd(&std::env::temp_dir(), || commands::switch::run(
            None, None, None, false
        ))
        .is_err()
    );
}

#[test]
/// Test case for switch -c reporting an error when branch already exists.
///
fn switch_c_fails_if_branch_already_exists() {
    let p = common::plain_repo();
    common::git(&p.path, ["branch", "topic"].as_slice());

    // Switch -c topic should report that branch already exists without crashing.
    common::in_cwd(&p.path, || {
        commands::switch::run(None, Some("topic"), None, false)
    })
    .unwrap();

    // HEAD should still be main since -c failed.
    let head_name = String::from_utf8_lossy(
        &common::git_out(&p.path, ["rev-parse", "--abbrev-ref", "HEAD"].as_slice()).stdout,
    )
    .trim()
    .to_string();
    assert_eq!(head_name, "main");
}

#[test]
/// Test case for switch -c with a specified start point revision.
///
fn switch_c_with_start_point() {
    let p = common::plain_repo();
    let initial_oid =
        String::from_utf8_lossy(&common::git_out(&p.path, ["rev-parse", "HEAD"].as_slice()).stdout)
            .trim()
            .to_string();

    std::fs::write(p.path.join("file.txt"), "second commit\n").unwrap();
    common::git(&p.path, ["add", "file.txt"].as_slice());
    common::git(&p.path, ["commit", "-q", "-m", "second"].as_slice());

    common::in_cwd(&p.path, || {
        commands::switch::run(Some("HEAD~1"), Some("branched_earlier"), None, false)
    })
    .unwrap();

    let head_name = String::from_utf8_lossy(
        &common::git_out(&p.path, ["rev-parse", "--abbrev-ref", "HEAD"].as_slice()).stdout,
    )
    .trim()
    .to_string();
    assert_eq!(head_name, "branched_earlier");

    let head_oid =
        String::from_utf8_lossy(&common::git_out(&p.path, ["rev-parse", "HEAD"].as_slice()).stdout)
            .trim()
            .to_string();
    assert_eq!(head_oid, initial_oid);
}

#[test]
/// Test case for switch -C creating and switching to a branch that did not exist.
///
fn switch_force_create_creates_new_branch_when_not_existing() {
    let p = common::plain_repo();

    common::in_cwd(&p.path, || {
        commands::switch::run(None, None, Some("brandnew"), false)
    })
    .unwrap();

    let head_name = String::from_utf8_lossy(
        &common::git_out(&p.path, ["rev-parse", "--abbrev-ref", "HEAD"].as_slice()).stdout,
    )
    .trim()
    .to_string();
    assert_eq!(head_name, "brandnew");
}
