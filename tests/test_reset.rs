//! Tests for `sgit reset`.

mod common;
use sgit::commands;

#[test]
/// Test case for reset unstages staged changes.
///
fn reset_unstages_staged_changes() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);

    common::in_cwd(&p.path, || commands::reset::run(None, false)).unwrap();

    let out = common::git_out(&p.path, &["diff", "--cached", "--name-only"]);
    assert!(String::from_utf8_lossy(&out.stdout).trim().is_empty());
}

#[test]
/// Test case for hard reset discards all changes.
///
fn hard_reset_discards_all_changes() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);

    common::in_cwd(&p.path, || commands::reset::run(None, true)).unwrap();

    assert_eq!(
        std::fs::read_to_string(p.path.join("file.txt")).unwrap(),
        "content\n"
    );
}

#[test]
/// Test case for hard reset to previous commit.
///
fn hard_reset_to_previous_commit() {
    let p = common::plain_repo();
    let first = String::from_utf8_lossy(&common::git_out(&p.path, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_string();
    std::fs::write(p.path.join("file.txt"), "v2\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);
    common::git(&p.path, &["commit", "-q", "-m", "second"]);

    common::in_cwd(&p.path, || commands::reset::run(Some("HEAD~1"), true)).unwrap();

    let now = String::from_utf8_lossy(&common::git_out(&p.path, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_string();
    assert_eq!(now, first);
    assert_eq!(
        std::fs::read_to_string(p.path.join("file.txt")).unwrap(),
        "content\n"
    );
}

#[test]
/// Test case for hard reset across submodules.
///
fn hard_reset_across_submodules() {
    let r = common::repo_with_submodules();
    std::fs::write(r.main.join("main.txt"), "changed\n").unwrap();
    std::fs::write(r.main.join("sub1/file1.txt"), "changed\n").unwrap();

    common::in_cwd(&r.main, || commands::reset::run(None, true)).unwrap();
    assert_eq!(
        std::fs::read_to_string(r.main.join("main.txt")).unwrap(),
        "main\n"
    );
    assert_eq!(
        std::fs::read_to_string(r.main.join("sub1/file1.txt")).unwrap(),
        "sub1\n"
    );
}
