//! Tests for `sgit merge`.

mod common;
use sgit::commands;

#[test]
/// Test case for merge branch.
///
fn merge_branch() {
    let p = common::plain_repo();
    common::git(&p.path, ["checkout", "-q", "-b", "feature"].as_slice());
    std::fs::write(p.path.join("feature.txt"), "feature\n").unwrap();
    common::git(&p.path, ["add", "feature.txt"].as_slice());
    common::git(&p.path, ["commit", "-q", "-m", "feat"].as_slice());
    common::git(&p.path, ["checkout", "-q", "main"].as_slice());

    common::in_cwd(&p.path, || commands::merge::run("feature")).unwrap();
    assert!(p.path.join("feature.txt").exists());
}

#[test]
/// Test case for merge missing branch skips.
///
fn merge_missing_branch_skips() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || commands::merge::run("nonexistent")).unwrap();
}

#[test]
/// Test case for merge across submodules.
///
fn merge_across_submodules() {
    let r = common::repo_with_submodules();
    common::in_cwd(&r.main, || commands::branch::run(Some("feature"))).unwrap();
    common::in_cwd(&r.main, || commands::merge::run("feature")).unwrap();

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

    assert_eq!(b_main, "main");
    assert_eq!(b_sub1, "main");
    assert_eq!(b_sub2, "main");
}

#[test]
/// Test case for merge conflict exits.
///
fn merge_conflict_exits() {
    let p = common::plain_repo();
    common::git(&p.path, ["checkout", "-q", "-b", "other"].as_slice());
    std::fs::write(p.path.join("file.txt"), "other\n").unwrap();
    common::git(&p.path, ["add", "file.txt"].as_slice());
    common::git(&p.path, ["commit", "-q", "-m", "o"].as_slice());
    common::git(&p.path, ["checkout", "-q", "main"].as_slice());
    std::fs::write(p.path.join("file.txt"), "main\n").unwrap();
    common::git(&p.path, ["add", "file.txt"].as_slice());
    common::git(&p.path, ["commit", "-q", "-m", "m"].as_slice());

    assert!(common::in_cwd(&p.path, || commands::merge::run("other")).is_err());
}
