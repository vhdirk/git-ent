//! Tests for `sgit restore`.

mod common;
use sgit::cmd::restore::RestoreCmd;

#[test]
/// Test case for restore single file.
///
fn restore_single_file() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::in_cwd(
        &p.path,
        RestoreCmd {
            paths: vec!["file.txt".into()],
            staged: false,
        },
    )
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
    common::in_cwd(
        &p.path,
        RestoreCmd {
            paths: vec![],
            staged: false,
        },
    )
    .unwrap();
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
    common::in_cwd(
        &p.path,
        RestoreCmd {
            paths: vec![],
            staged: false,
        },
    )
    .unwrap();
}

#[test]
/// Test case for unstage single file.
///
fn unstage_single_file() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "staged\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);
    common::in_cwd(
        &p.path,
        RestoreCmd {
            paths: vec!["file.txt".into()],
            staged: true,
        },
    )
    .unwrap();
}

#[test]
/// Test case for unstage all.
///
fn unstage_all() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "staged\n").unwrap();
    common::git(&p.path, &["add", "file.txt"]);
    common::in_cwd(
        &p.path,
        RestoreCmd {
            paths: vec![],
            staged: true,
        },
    )
    .unwrap();
}

#[test]
/// Test case for restore file in submodule.
///
fn restore_file_in_submodule() {
    let r = common::repo_with_submodules();
    std::fs::write(r.main.join("sub1/file1.txt"), "changed\n").unwrap();
    common::in_cwd(
        &r.main,
        RestoreCmd {
            paths: vec!["sub1/file1.txt".into()],
            staged: false,
        },
    )
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
    common::in_cwd(
        &p.path,
        RestoreCmd {
            paths: vec!["/nonexistent/file.txt".into()],
            staged: false,
        },
    )
    .unwrap();
}
