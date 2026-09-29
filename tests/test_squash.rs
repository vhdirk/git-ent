//! Tests for `sgit squash <branch>` - the new squash-onto-merge-base
//! command. Mirrors GitHub/GitLab's "squash and merge" behaviour,
//! depth-first across submodules.

mod common;
use sgit::commands;

#[test]
/// Test case for squash missing branch skips.
///
fn squash_missing_branch_skips() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || commands::squash::run("nonexistent", None)).unwrap();
}

#[test]
/// Test case for squash collapses multiple commits into one.
///
fn squash_collapses_multiple_commits_into_one() {
    let p = common::plain_repo();
    // Baseline commit is on main. Branch off a new feature and make two
    // commits. Squash onto main → a single commit replaces the two.
    common::git(&p.path, &["checkout", "-q", "-b", "feature"]);
    std::fs::write(p.path.join("a.txt"), "a\n").unwrap();
    common::git(&p.path, &["add", "a.txt"]);
    common::git(&p.path, &["commit", "-q", "-m", "commit 1"]);
    std::fs::write(p.path.join("b.txt"), "b\n").unwrap();
    common::git(&p.path, &["add", "b.txt"]);
    common::git(&p.path, &["commit", "-q", "-m", "commit 2"]);

    let before =
        String::from_utf8_lossy(&common::git_out(&p.path, &["rev-list", "--count", "HEAD"]).stdout)
            .trim()
            .to_string();
    assert_eq!(before, "3"); // initial + 2

    common::in_cwd(&p.path, || commands::squash::run("main", None)).unwrap();

    let after =
        String::from_utf8_lossy(&common::git_out(&p.path, &["rev-list", "--count", "HEAD"]).stdout)
            .trim()
            .to_string();
    assert_eq!(after, "2"); // initial + 1 squash

    // Both files still present.
    assert!(p.path.join("a.txt").exists());
    assert!(p.path.join("b.txt").exists());
}

#[test]
/// Test case for squash custom message.
///
fn squash_custom_message() {
    let p = common::plain_repo();
    common::git(&p.path, &["checkout", "-q", "-b", "feature"]);
    std::fs::write(p.path.join("a.txt"), "a\n").unwrap();
    common::git(&p.path, &["add", "a.txt"]);
    common::git(&p.path, &["commit", "-q", "-m", "commit 1"]);
    std::fs::write(p.path.join("b.txt"), "b\n").unwrap();
    common::git(&p.path, &["add", "b.txt"]);
    common::git(&p.path, &["commit", "-q", "-m", "commit 2"]);

    common::in_cwd(&p.path, || commands::squash::run("main", Some("my squash"))).unwrap();

    let msg =
        String::from_utf8_lossy(&common::git_out(&p.path, &["log", "-1", "--pretty=%s"]).stdout)
            .trim()
            .to_string();
    assert_eq!(msg, "my squash");
}

#[test]
/// Test case for squash default message lists subjects.
///
fn squash_default_message_lists_subjects() {
    let p = common::plain_repo();
    common::git(&p.path, &["checkout", "-q", "-b", "feature"]);
    std::fs::write(p.path.join("a.txt"), "a\n").unwrap();
    common::git(&p.path, &["add", "a.txt"]);
    common::git(&p.path, &["commit", "-q", "-m", "first change"]);
    std::fs::write(p.path.join("b.txt"), "b\n").unwrap();
    common::git(&p.path, &["add", "b.txt"]);
    common::git(&p.path, &["commit", "-q", "-m", "second change"]);

    common::in_cwd(&p.path, || commands::squash::run("main", None)).unwrap();

    let body =
        String::from_utf8_lossy(&common::git_out(&p.path, &["log", "-1", "--pretty=%B"]).stdout)
            .to_string();
    assert!(body.contains("first change"));
    assert!(body.contains("second change"));
}

#[test]
/// Test case for squash skips when on target branch.
///
fn squash_skips_when_on_target_branch() {
    let p = common::plain_repo();
    // We are on main and trying to squash onto main.
    common::in_cwd(&p.path, || commands::squash::run("main", None)).unwrap();
}

#[test]
/// Test case for squash nothing to squash when already at merge base.
///
fn squash_nothing_to_squash_when_already_at_merge_base() {
    let p = common::plain_repo();
    common::git(&p.path, &["checkout", "-q", "-b", "feature"]);
    // No new commits; feature == main.
    common::in_cwd(&p.path, || commands::squash::run("main", None)).unwrap();
}

#[test]
/// Test case for squash across submodules depth first.
///
fn squash_across_submodules_depth_first() {
    let r = common::repo_with_submodules();

    // Create `feature` branches everywhere.
    common::in_cwd(&r.main, || commands::branch::run(Some("feature"))).unwrap();

    // Check out `feature` in every repo (recursively).
    common::git(&r.main, &["checkout", "-q", "feature"]);
    common::git(&r.main.join("sub1"), &["checkout", "-q", "feature"]);
    common::git(&r.main.join("sub2"), &["checkout", "-q", "feature"]);

    // Two commits on sub1's feature branch.
    std::fs::write(r.main.join("sub1/x.txt"), "x\n").unwrap();
    common::git(&r.main.join("sub1"), &["add", "x.txt"]);
    common::git(&r.main.join("sub1"), &["commit", "-q", "-m", "sub1 c1"]);
    std::fs::write(r.main.join("sub1/y.txt"), "y\n").unwrap();
    common::git(&r.main.join("sub1"), &["add", "y.txt"]);
    common::git(&r.main.join("sub1"), &["commit", "-q", "-m", "sub1 c2"]);

    // Two commits on the parent's feature branch.
    std::fs::write(r.main.join("main.txt"), "changed1\n").unwrap();
    common::git(&r.main, &["add", "main.txt"]);
    common::git(&r.main, &["commit", "-q", "-m", "parent c1"]);
    std::fs::write(r.main.join("main.txt"), "changed2\n").unwrap();
    common::git(&r.main, &["add", "main.txt"]);
    common::git(&r.main, &["commit", "-q", "-m", "parent c2"]);

    // Record baseline counts.
    let parent_before =
        String::from_utf8_lossy(&common::git_out(&r.main, &["rev-list", "--count", "HEAD"]).stdout)
            .trim()
            .to_string();
    let sub1_before = String::from_utf8_lossy(
        &common::git_out(&r.main.join("sub1"), &["rev-list", "--count", "HEAD"]).stdout,
    )
    .trim()
    .to_string();

    common::in_cwd(&r.main, || commands::squash::run("main", None)).unwrap();

    // sub1 should collapse 2 commits into 1.
    let sub1_after: i32 = String::from_utf8_lossy(
        &common::git_out(&r.main.join("sub1"), &["rev-list", "--count", "HEAD"]).stdout,
    )
    .trim()
    .parse()
    .unwrap();
    let sub1_before_n: i32 = sub1_before.parse().unwrap();
    assert_eq!(sub1_after, sub1_before_n - 1);

    // Parent should also collapse its 2 commits + pick up the new submodule ref.
    let parent_after: i32 =
        String::from_utf8_lossy(&common::git_out(&r.main, &["rev-list", "--count", "HEAD"]).stdout)
            .trim()
            .parse()
            .unwrap();
    let parent_before_n: i32 = parent_before.parse().unwrap();
    assert_eq!(parent_after, parent_before_n - 1);
}

#[test]
/// Test case for squash submodule only creates parent pointer commit.
///
fn squash_submodule_only_creates_parent_pointer_commit() {
    // When a deeper submodule gets squashed but the parent itself has no
    // commits to squash, the parent still needs a new commit recording the
    // moved submodule pointer.
    let r = common::repo_with_submodules();
    common::in_cwd(&r.main, || commands::branch::run(Some("feature"))).unwrap();

    // Only sub1 checks out `feature` and makes commits.
    common::git(&r.main.join("sub1"), &["checkout", "-q", "feature"]);
    std::fs::write(r.main.join("sub1/x.txt"), "x\n").unwrap();
    common::git(&r.main.join("sub1"), &["add", "x.txt"]);
    common::git(&r.main.join("sub1"), &["commit", "-q", "-m", "sub1 c1"]);
    std::fs::write(r.main.join("sub1/y.txt"), "y\n").unwrap();
    common::git(&r.main.join("sub1"), &["add", "y.txt"]);
    common::git(&r.main.join("sub1"), &["commit", "-q", "-m", "sub1 c2"]);

    // Parent stays on main (not feature); it has no commits to squash
    // but the submodule pointer is dirty.
    common::git(&r.main, &["checkout", "-q", "feature"]);

    common::in_cwd(&r.main, || commands::squash::run("main", None)).unwrap();

    // The sub1 ref change should have been committed into the parent feature branch.
    let status =
        String::from_utf8_lossy(&common::git_out(&r.main, &["status", "--porcelain"]).stdout)
            .to_string();
    assert!(
        !status.contains("sub1"),
        "parent should have no dirty sub pointer"
    );
}
