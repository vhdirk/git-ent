mod common;

use git_nest::cli::update::UpdateCmd;
use std::path::Path;

#[test]
/// Test case for update noop without submodules.
///
fn update_noop_without_submodules() {
    let p = common::plain_repo();
    common::in_cwd(&p.path, UpdateCmd {}).unwrap();
}

#[test]
/// Test case for update initializes submodules.
///
fn update_initializes_submodules() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path();

    let sub = base.join("sub");
    common::make_repo(&sub, &[("sub.txt", "sub\n")]);

    let parent = base.join("parent");
    common::make_repo(&parent, &[("main.txt", "main\n")]);
    common::git(
        &parent,
        &[
            "submodule",
            "add",
            "-q",
            &sub.display().to_string(),
            "mysub",
        ],
    );
    common::git(&parent, &["commit", "-q", "-m", "add submodule"]);

    let clone = base.join("clone");
    common::git(
        Path::new("."),
        &[
            "clone",
            "-q",
            &parent.display().to_string(),
            &clone.display().to_string(),
        ],
    );

    assert!(clone.join("mysub").exists());
    assert!(!clone.join("mysub/sub.txt").exists());

    common::in_cwd(&clone, UpdateCmd {}).unwrap();
    assert!(clone.join("mysub/sub.txt").exists());
}
