//! Tests for `sgit restore`.

mod common;
use sgit::commands;

#[test]
/// Test case for restore single file.
///
fn restore_single_file() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::in_cwd(&p.path, || {
        commands::restore::run(&["file.txt".into()], false)
    })
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(p.path.join("file.txt")).unwrap(),
        "content\n"
    );
}

#[test]
/// Test case for restore all unstaged.
///
fn restore_all_unstaged() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::in_cwd(&p.path, || commands::restore::run(&[], false)).unwrap();
    assert_eq!(
        std::fs::read_to_string(p.path.join("file.txt")).unwrap(),
        "content\n"
    );
}

#[test]
/// Test case for restore noop on clean repo.
///
fn restore_noop_on_clean_repo() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || commands::restore::run(&[], false)).unwrap();
}

#[test]
/// Test case for unstage single file.
///
fn unstage_single_file() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "staged\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);
    common::in_cwd(&p.path, || {
        commands::restore::run(&["file.txt".into()], true)
    })
    .unwrap();
}

#[test]
/// Test case for unstage all.
///
fn unstage_all() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "staged\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);
    common::in_cwd(&p.path, || commands::restore::run(&[], true)).unwrap();
}

#[test]
/// Test case for restore file in submodule.
///
fn restore_file_in_submodule() {
    let r = common::repo_with_submodules();
    std::fs::write(r.main.join("sub1/file1.txt"), "changed\n").unwrap();
    common::in_cwd(&r.main, || {
        commands::restore::run(&["sub1/file1.txt".into()], false)
    })
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(r.main.join("sub1/file1.txt")).unwrap(),
        "sub1\n"
    );
}

#[test]
/// Test case for restore nonexistent reports.
///
fn restore_nonexistent_reports() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, || {
        commands::restore::run(&["/nonexistent/file.txt".into()], false)
    })
    .unwrap();
}
