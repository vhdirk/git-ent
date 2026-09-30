//! Tests for `sgit add`.

mod common;
use sgit::commands;

#[test]
/// Test case for add single file.
///
fn add_single_file() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::in_cwd(&p.path, || {
        commands::add::run(&["file.txt".into()], false, false)
    })
    .unwrap();

    let diff = common::git_out(&p.path, &["diff", "--cached", "--name-only"]);
    assert!(String::from_utf8_lossy(&diff.stdout).contains("file.txt"));
}

#[test]
/// Test case for add new file.
///
fn add_new_file() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("new.txt"), "new\n").unwrap();
    common::in_cwd(&p.path, || {
        commands::add::run(&["new.txt".into()], false, false)
    })
    .unwrap();
}

#[test]
/// Test case for add multiple files.
///
fn add_multiple_files() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("a.txt"), "a\n").unwrap();
    std::fs::write(p.path.join("b.txt"), "b\n").unwrap();
    common::in_cwd(&p.path, || {
        commands::add::run(&["a.txt".into(), "b.txt".into()], false, false)
    })
    .unwrap();

    let diff = common::git_out(&p.path, &["diff", "--cached", "--name-only"]);
    let staged = String::from_utf8_lossy(&diff.stdout);
    assert!(staged.contains("a.txt"));
    assert!(staged.contains("b.txt"));
}

#[test]
/// Test case for add nonexistent file reports error.
///
fn add_nonexistent_file_reports_error() {
    let p = common::plain_repo();
    // Should still succeed at library layer (reports to stderr, continues).
    common::in_cwd(&p.path, || {
        commands::add::run(&["/nonexistent/path/file.txt".into()], false, false)
    })
    .unwrap();
}

#[test]
/// Test case for add file in submodule.
///
fn add_file_in_submodule() {
    let r = common::repo_with_submodules();
    std::fs::write(r.main.join("sub1/file1.txt"), "changed\n").unwrap();
    common::in_cwd(&r.main, || {
        commands::add::run(&["sub1/file1.txt".into()], false, false)
    })
    .unwrap();

    let diff = common::git_out(
        &r.sub1.parent().unwrap().join("main-repo/sub1"),
        &["diff", "--cached", "--name-only"],
    );
    assert!(String::from_utf8_lossy(&diff.stdout).contains("file1.txt"));
}

#[test]
/// Test case for add all stages everything.
///
fn add_all_stages_everything() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    std::fs::write(p.path.join("new.txt"), "new\n").unwrap();
    common::in_cwd(&p.path, || commands::add::run(&[], true, false)).unwrap();
}

#[test]
/// Test case for add all across submodules.
///
fn add_all_across_submodules() {
    let r = common::repo_with_submodules();
    std::fs::write(r.main.join("main.txt"), "changed\n").unwrap();
    std::fs::write(r.main.join("sub1/file1.txt"), "changed\n").unwrap();
    common::in_cwd(&r.main, || commands::add::run(&[], true, false)).unwrap();

    let diff_main = common::git_out(&r.main, &["diff", "--cached", "--name-only"]);
    let main_staged = String::from_utf8_lossy(&diff_main.stdout);
    assert!(main_staged.contains("main.txt"));

    let diff_sub1 = common::git_out(&r.main.join("sub1"), &["diff", "--cached", "--name-only"]);
    let sub1_staged = String::from_utf8_lossy(&diff_sub1.stdout);
    assert!(sub1_staged.contains("file1.txt"));
}

#[test]
/// Test case for add update stages modified not untracked.
///
fn add_update_stages_modified_not_untracked() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    std::fs::write(p.path.join("untracked.txt"), "new\n").unwrap();
    common::in_cwd(&p.path, || commands::add::run(&[], false, true)).unwrap();

    let diff = common::git_out(&p.path, &["diff", "--cached", "--name-only"]);
    let s = String::from_utf8_lossy(&diff.stdout);
    assert!(s.contains("file.txt"));
    assert!(!s.contains("untracked.txt"));
}

#[test]
/// Test case for all and update mutually exclusive.
///
fn all_and_update_mutually_exclusive() {
    let p = common::plain_repo();
    assert!(common::in_cwd(&p.path, || commands::add::run(&[], true, true)).is_err());
}

#[test]
fn add_path_with_spaces() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file with spaces.txt"), "spaces\n").unwrap();
    common::in_cwd(&p.path, || {
        commands::add::run(&["file with spaces.txt".into()], false, false)
    })
    .unwrap();

    let diff = common::git_out(&p.path, &["diff", "--cached", "--name-only"]);
    assert!(String::from_utf8_lossy(&diff.stdout).contains("file with spaces.txt"));
}

#[test]
fn add_path_with_leading_dash() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("-leading-dash.txt"), "dash\n").unwrap();
    common::in_cwd(&p.path, || {
        commands::add::run(&["-leading-dash.txt".into()], false, false)
    })
    .unwrap();

    let diff = common::git_out(&p.path, &["diff", "--cached", "--name-only"]);
    assert!(String::from_utf8_lossy(&diff.stdout).contains("-leading-dash.txt"));
}
