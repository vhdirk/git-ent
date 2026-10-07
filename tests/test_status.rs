mod common;
use git_ent::cli::status::StatusCmd;

#[test]
/// Test case for clean repo prints nothing.
///
fn clean_repo_prints_nothing() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, StatusCmd {}).unwrap();
}

#[test]
/// Test case for clean with submodules prints nothing.
///
fn clean_with_submodules_prints_nothing() {
    let r = common::repo_with_submodules();
    common::in_cwd(&r.main, StatusCmd {}).unwrap();
}

#[test]
/// Test case for modified file.
///
fn modified_file() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::in_cwd(&p.path, StatusCmd {}).unwrap();
}

#[test]
/// Test case for untracked file.
///
fn untracked_file() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("new.txt"), "new\n").unwrap();
    common::in_cwd(&p.path, StatusCmd {}).unwrap();
}

#[test]
/// Test case for staged file.
///
fn staged_file() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "staged\n").unwrap();
    common::git(&p.path, ["add", "file.txt"].as_slice());
    common::in_cwd(&p.path, StatusCmd {}).unwrap();
}

#[test]
/// Test case for staged new file.
///
fn staged_new_file() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("brand_new.txt"), "new\n").unwrap();
    common::git(&p.path, ["add", "brand_new.txt"].as_slice());
    common::in_cwd(&p.path, StatusCmd {}).unwrap();
}

#[test]
/// Test case for shows branch name.
///
fn shows_branch_name() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "changed\n").unwrap();
    common::in_cwd(&p.path, StatusCmd {}).unwrap();
}

#[test]
/// Test case for shows hint messages.
///
fn shows_hint_messages() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("file.txt"), "staged\n").unwrap();
    common::git(&p.path, ["add", "file.txt"].as_slice());
    std::fs::write(p.path.join("file.txt"), "changed again\n").unwrap();
    std::fs::write(p.path.join("new.txt"), "new\n").unwrap();
    common::in_cwd(&p.path, StatusCmd {}).unwrap();
}

#[test]
/// Test case for submodule paths relative to toplevel.
///
fn submodule_paths_relative_to_toplevel() {
    let r = common::repo_with_submodules();
    std::fs::write(r.main.join("sub1/file1.txt"), "changed\n").unwrap();
    common::in_cwd(&r.main, StatusCmd {}).unwrap();
}
