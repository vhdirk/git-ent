//! Tests for `sgit rebase`.

mod common;
use sgit::commands;

#[test]
/// Test case for rebase missing branch skips.
///
fn rebase_missing_branch_skips() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || commands::rebase::run("nonexistent")).unwrap();
}

#[test]
/// Test case for rebase onto branch.
///
fn rebase_onto_branch() {
    let p = common::plain_repo();
    common::git(&p.path, &["branch", "target"]);

    common::in_cwd(&p.path, || commands::rebase::run("target")).unwrap();
}

#[test]
/// Test case for rebase across submodules.
///
fn rebase_across_submodules() {
    let r = common::repo_with_submodules();
    common::in_cwd(&r.main, || commands::branch::run(Some("target"))).unwrap();
    common::in_cwd(&r.main, || commands::rebase::run("target")).unwrap();

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

    assert!(b_main == "main" || b_main == "HEAD");
    assert!(b_sub1 == "main" || b_sub1 == "HEAD");
    assert!(b_sub2 == "main" || b_sub2 == "HEAD");
}

#[test]
/// Test case for rebase conflict aborts cleanly and reports conflict error.
fn rebase_conflict_aborts_and_reports_error() {
    let p = common::plain_repo();
    common::git(&p.path, &["checkout", "-b", "feature"]);
    std::fs::write(p.path.join("file.txt"), "feature content\n").unwrap();
    common::git(&p.path, &["commit", "-am", "feature commit"]);

    common::git(&p.path, &["checkout", "main"]);
    std::fs::write(p.path.join("file.txt"), "conflicting main content\n").unwrap();
    common::git(&p.path, &["commit", "-am", "main conflicting commit"]);

    common::git(&p.path, &["checkout", "feature"]);

    let err = common::in_cwd(&p.path, || commands::rebase::run("main")).unwrap_err();
    match err {
        sgit::SgitError::Conflict { hint, .. } => {
            assert!(hint.contains("git rebase main"));
        }
        other => panic!("Expected conflict error, got: {other:?}"),
    }

    // Assert that rebase was cleanly aborted
    assert!(!p.path.join(".git/rebase-apply").exists());
    assert!(!p.path.join(".git/rebase-merge").exists());
}

#[test]
/// Test case for direct RebaseCmd execution.
fn rebase_cmd_direct_execution() {
    let p = common::plain_repo();
    common::git(&p.path, &["branch", "target"]);

    let r = sgit::Repo::discover(&p.path).unwrap();
    let cmd = sgit::git::rebase::RebaseCmd::new("target");
    let output = r.git(&cmd).unwrap();

    assert!(output.is_up_to_date() || output.is_successful());
}
