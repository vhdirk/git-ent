mod common;

use sgit::Repo;

#[test]
fn discover_from_repository_root_returns_root_repo() {
    let repo = common::plain_repo();

    let discovered = Repo::discover(&repo.path).unwrap();

    assert_eq!(
        discovered.workdir,
        std::fs::canonicalize(repo.path).unwrap()
    );
    assert_eq!(discovered.prefix, std::path::Path::new("."));
}

#[test]
fn discover_from_nested_directory_returns_repository_root() {
    let repo = common::plain_repo();
    let nested = repo.path.join("nested");
    std::fs::create_dir(&nested).unwrap();

    let discovered = Repo::discover(&nested).unwrap();

    assert_eq!(
        discovered.workdir,
        std::fs::canonicalize(repo.path).unwrap()
    );
    assert_eq!(discovered.prefix, std::path::Path::new("."));
}

#[test]
fn discover_from_submodule_handles_gitfile() {
    let repos = common::repo_with_submodules();

    let discovered = Repo::discover(repos.main.join("sub1")).unwrap();

    assert_eq!(
        discovered.workdir,
        std::fs::canonicalize(repos.main.join("sub1")).unwrap()
    );
    assert_eq!(discovered.prefix, std::path::Path::new("."));
}

#[test]
fn discover_outside_repository_returns_error() {
    let outside = tempfile::tempdir().unwrap();

    assert!(Repo::discover(outside.path()).is_err());
}
