//! Shared test helpers. Every test works inside a `tempfile::TempDir`
//! so tests never touch the real filesystem.

#![allow(dead_code)]

use git2::DiffLineType::Context;
use sgit::SgitError;
use sgit::cmd::command::{Cmd as SgitCommand, Context as SgitContext};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use tempfile::TempDir;

/// Run `git <args>` inside `cwd`, allowing the `file://` protocol (needed
/// to `submodule add` a local repo).
pub fn git(cwd: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(["-c", "protocol.file.allow=always"])
        .args(args)
        .current_dir(cwd)
        .status()
        .expect("git failed to spawn");
    assert!(status.success(), "git {args:?} failed in {cwd:?}");
}

/// Run `git <args>` capturing stderr, allowing the `file://` protocol.
pub fn git_out(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .args(["-c", "protocol.file.allow=always"])
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git failed to spawn")
}

/// Make a git repo at `path` (creating it) with one commit containing `files`.
pub fn make_repo(path: &Path, files: &[(&str, &str)]) {
    std::fs::create_dir_all(path).unwrap();
    git(path, ["init", "-q", "-b", "main"].as_slice());
    git(
        path,
        ["config", "user.email", "test@example.com"].as_slice(),
    );
    git(path, ["config", "user.name", "Test"].as_slice());
    git(path, ["config", "commit.gpgsign", "false"].as_slice());
    let files = if files.is_empty() {
        &[("dummy.txt", "hello\n")][..]
    } else {
        files
    };
    for (name, content) in files {
        std::fs::write(path.join(name), content).unwrap();
    }
    let names: Vec<&str> = files.iter().map(|(n, _)| *n).collect();
    let mut args = vec!["add", "--"];
    args.extend(names);
    git(path, &args);
    git(path, ["commit", "-q", "-m", "initial commit"].as_slice());
}

/// A single plain repo with `file.txt` containing `content\n`.
pub struct PlainRepo {
    pub tmp: TempDir,
    pub path: PathBuf,
}

/// Test helper for plain repo.
///
pub fn plain_repo() -> PlainRepo {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("repo");
    make_repo(&path, [("file.txt", "content\n")].as_slice());
    PlainRepo { tmp, path }
}

/// A main repo with two flat submodules (`sub1`, `sub2`).
pub struct RepoWithSubmodules {
    pub tmp: TempDir,
    pub main: PathBuf,
    pub sub1: PathBuf,
    pub sub2: PathBuf,
}

/// Test helper for repo with submodules.
///
pub fn repo_with_submodules() -> RepoWithSubmodules {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().join("repos");
    std::fs::create_dir_all(&base).unwrap();

    let sub1 = base.join("sub1");
    let sub2 = base.join("sub2");
    make_repo(&sub1, [("file1.txt", "sub1\n")].as_slice());
    make_repo(&sub2, [("file2.txt", "sub2\n")].as_slice());

    let main = base.join("main-repo");
    make_repo(&main, [("main.txt", "main\n")].as_slice());

    git(
        &main,
        [
            "submodule",
            "add",
            "-q",
            &sub1.display().to_string(),
            "sub1",
        ]
        .as_slice(),
    );
    git(
        &main,
        [
            "submodule",
            "add",
            "-q",
            &sub2.display().to_string(),
            "sub2",
        ]
        .as_slice(),
    );
    git(&main, ["commit", "-q", "-m", "add submodules"].as_slice());

    RepoWithSubmodules {
        tmp,
        main,
        sub1,
        sub2,
    }
}

/// main -> mid -> leaf nested submodules.
pub struct NestedSubmodules {
    pub tmp: TempDir,
    pub main: PathBuf,
    pub mid: PathBuf,
    pub leaf: PathBuf,
}

/// Test helper for nested submodules.
///
pub fn nested_submodules() -> NestedSubmodules {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().join("repos");
    std::fs::create_dir_all(&base).unwrap();

    let leaf = base.join("leaf");
    make_repo(&leaf, [("leaf.txt", "leaf\n")].as_slice());

    let mid = base.join("mid");
    make_repo(&mid, [("mid.txt", "mid\n")].as_slice());
    git(
        &mid,
        [
            "submodule",
            "add",
            "-q",
            &leaf.display().to_string(),
            "leaf",
        ]
        .as_slice(),
    );
    git(&mid, ["commit", "-q", "-m", "add leaf"].as_slice());

    let main = base.join("main");
    make_repo(&main, [("main.txt", "main\n")].as_slice());
    git(
        &main,
        ["submodule", "add", "-q", &mid.display().to_string(), "mid"].as_slice(),
    );
    git(&main, ["commit", "-q", "-m", "add mid"].as_slice());
    git(
        &main,
        ["submodule", "update", "-q", "--init", "--recursive"].as_slice(),
    );

    NestedSubmodules {
        tmp,
        main,
        mid,
        leaf,
    }
}

fn cwd_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Execute `c` with process cwd temporarily set to `cwd` under a global lock.
pub fn in_cwd(cwd: &Path, c: impl SgitCommand) -> Result<(), SgitError> {
    let _guard = cwd_lock().lock().expect("cwd lock poisoned");
    let prev_cwd = std::env::current_dir().expect("read cwd");
    std::env::set_current_dir(cwd).expect("set cwd");
    let out = c.run(&SgitContext::default());
    std::env::set_current_dir(prev_cwd).expect("restore cwd");
    out
}
