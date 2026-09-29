//! Tests for `sgit checkout`.

mod common;
use sgit::commands;

#[test]
/// Test case for checkout existing branch across submodules.
///
fn checkout_existing_branch_across_submodules() {
    let r = common::repo_with_submodules();
    common::in_cwd(&r.main, || commands::branch::run(Some("feature"))).unwrap();

    common::in_cwd(&r.main, || {
        commands::checkout::run(Some("feature"), None, &[])
    })
    .unwrap();

    let b_main = String::from_utf8_lossy(
        &common::git_out(&r.main, ["rev-parse", "--abbrev-ref", "HEAD"].as_slice()).stdout,
    )
    .trim()
    .to_string();
    let b_sub1 = String::from_utf8_lossy(
        &common::git_out(
            &r.main.join("sub1"),
            ["rev-parse", "--abbrev-ref", "HEAD"].as_slice(),
        )
        .stdout,
    )
    .trim()
    .to_string();
    let b_sub2 = String::from_utf8_lossy(
        &common::git_out(
            &r.main.join("sub2"),
            ["rev-parse", "--abbrev-ref", "HEAD"].as_slice(),
        )
        .stdout,
    )
    .trim()
    .to_string();

    assert_eq!(b_main, "feature");
    assert_eq!(b_sub1, "feature");
    assert_eq!(b_sub2, "feature");
}

#[test]
/// Test case for checkout warns if branch missing in submodules.
///
fn checkout_warns_if_branch_missing_in_submodules() {
    let r = common::repo_with_submodules();
    common::git(&r.main, ["branch", "only-root"].as_slice());

    common::in_cwd(&r.main, || {
        commands::checkout::run(Some("only-root"), None, &[])
    })
    .unwrap();

    let b_main = String::from_utf8_lossy(
        &common::git_out(&r.main, ["rev-parse", "--abbrev-ref", "HEAD"].as_slice()).stdout,
    )
    .trim()
    .to_string();
    assert_eq!(b_main, "only-root");
}

#[test]
/// Test case for checkout b creates branch everywhere.
///
fn checkout_b_creates_branch_everywhere() {
    let r = common::repo_with_submodules();

    common::in_cwd(&r.main, || {
        commands::checkout::run(None, Some("topic"), &[])
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
/// Test case for checkout invalid usage when no branch is provided.
///
fn checkout_invalid_usage_without_branch() {
    assert!(
        common::in_cwd(&std::env::temp_dir(), || commands::checkout::run(
            None,
            None,
            &[]
        ))
        .is_err()
    );
}

#[test]
/// Test case for checkout branch files routes to submodule.
///
fn checkout_branch_files_routes_to_submodule() {
    let r = common::repo_with_submodules();
    common::in_cwd(&r.main, || commands::branch::run(Some("feature"))).unwrap();

    common::git(
        &r.main.join("sub1"),
        ["checkout", "-q", "feature"].as_slice(),
    );
    std::fs::write(r.main.join("sub1/file1.txt"), "feature version\n").unwrap();
    common::git(&r.main.join("sub1"), ["add", "file1.txt"].as_slice());
    common::git(
        &r.main.join("sub1"),
        ["commit", "-q", "-m", "sub1 feature"].as_slice(),
    );
    common::git(&r.main.join("sub1"), ["checkout", "-q", "main"].as_slice());

    common::in_cwd(&r.main, || {
        commands::checkout::run(Some("feature"), None, &["sub1/file1.txt".into()])
    })
    .unwrap();

    assert_eq!(
        std::fs::read_to_string(r.main.join("sub1/file1.txt")).unwrap(),
        "feature version\n"
    );
    let sub1_branch = String::from_utf8_lossy(
        &common::git_out(
            &r.main.join("sub1"),
            ["rev-parse", "--abbrev-ref", "HEAD"].as_slice(),
        )
        .stdout,
    )
    .trim()
    .to_string();
    assert_eq!(sub1_branch, "main");
}

#[test]
/// Test case for checkout branch files warns for missing branch in submodule.
///
fn checkout_branch_files_warns_for_missing_branch_in_submodule() {
    let r = common::repo_with_submodules();
    common::git(&r.main, ["branch", "only-root"].as_slice());

    common::in_cwd(&r.main, || {
        commands::checkout::run(Some("only-root"), None, &["sub1/file1.txt".into()])
    })
    .unwrap();
}
