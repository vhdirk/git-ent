//! `sgit switch` - recursive equivalent of git switch.

use git2::{BranchType, ErrorCode, Repository, build::CheckoutBuilder};

use crate::RepoTree;
use crate::error::{Result, SgitError};

/// Run recursive `switch` behavior across the full repo tree.
///
/// - `target`: branch/commit-ish to switch to or detach to.
/// - `create`: optional new branch name (`-c`).
/// - `force_create`: optional branch name for force-create (`-C`).
/// - `detach`: when `true`, detach HEAD at `target`.
pub fn run(
    target: Option<&str>,
    create: Option<&str>,
    force_create: Option<&str>,
    detach: bool,
) -> Result<()> {
    let tree = RepoTree::discover(None)?;

    if detach {
        let Some(target) = target else {
            return Err(SgitError::InvalidUsage(
                "usage: sgit switch --detach <commit-ish>".into(),
            ));
        };
        for r in tree.all() {
            let label = r.label();
            match detach_to(&r.repo, target) {
                Ok(()) => println!("[{label}] Switched to detached HEAD at '{target}'"),
                Err(e) => eprintln!("[{label}] Error switching to detached HEAD: {e}"),
            }
        }
        return Ok(());
    }

    if let Some(name) = create {
        for r in tree.all() {
            let label = r.label();
            match create_and_switch(&r.repo, name, target, false) {
                Ok(SwitchOutcome::CreatedAndSwitched) => {
                    println!("[{label}] Created and switched to '{name}'")
                }
                Ok(SwitchOutcome::Switched) => println!("[{label}] Switched to '{name}'"),
                Ok(SwitchOutcome::Missing) => unreachable!(),
                Err(e) => eprintln!("[{label}] Error creating/switching to '{name}': {e}"),
            }
        }
        return Ok(());
    }

    if let Some(name) = force_create {
        for r in tree.all() {
            let label = r.label();
            match create_and_switch(&r.repo, name, target, true) {
                Ok(SwitchOutcome::CreatedAndSwitched) => {
                    println!("[{label}] Created and switched to '{name}'")
                }
                Ok(SwitchOutcome::Switched) => println!("[{label}] Reset and switched to '{name}'"),
                Ok(SwitchOutcome::Missing) => unreachable!(),
                Err(e) => eprintln!("[{label}] Error resetting/switching to '{name}': {e}"),
            }
        }
        return Ok(());
    }

    let Some(branch) = target else {
        return Err(SgitError::InvalidUsage(
            "usage: sgit switch <branch> | sgit switch -c <new-branch> [<start-point>] | sgit switch -C <branch> [<start-point>] | sgit switch --detach <commit-ish>".into(),
        ));
    };

    for r in tree.all() {
        let label = r.label();
        match switch_existing(&r.repo, branch) {
            Ok(SwitchOutcome::Switched) => println!("[{label}] Switched to '{branch}'"),
            Ok(SwitchOutcome::Missing) => {
                eprintln!("[{label}] Warning: branch '{branch}' does not exist")
            }
            Ok(SwitchOutcome::CreatedAndSwitched) => unreachable!(),
            Err(e) => eprintln!("[{label}] Error switching to '{branch}': {e}"),
        }
    }

    Ok(())
}

enum SwitchOutcome {
    Switched,
    CreatedAndSwitched,
    Missing,
}

/// Switch to an existing local branch in one repository.
///
/// - `repo`: repository whose HEAD should move.
/// - `branch`: target local branch name.
fn switch_existing(repo: &Repository, branch: &str) -> Result<SwitchOutcome> {
    let br = match repo.find_branch(branch, BranchType::Local) {
        Ok(b) => b,
        Err(e) if e.code() == ErrorCode::NotFound => return Ok(SwitchOutcome::Missing),
        Err(e) => return Err(e.into()),
    };

    let refname = br.get().name().map_err(SgitError::from)?;
    repo.set_head(refname)?;
    let mut co = CheckoutBuilder::new();
    repo.checkout_head(Some(&mut co))?;
    Ok(SwitchOutcome::Switched)
}

/// Create (or optionally reset) a branch and switch to it.
///
/// - `repo`: repository to modify.
/// - `name`: branch name to create/switch.
/// - `start_point`: optional revision used as new branch start.
/// - `force`: when `true`, reset existing branch pointer.
fn create_and_switch(
    repo: &Repository,
    name: &str,
    start_point: Option<&str>,
    force: bool,
) -> Result<SwitchOutcome> {
    let start_commit = match start_point {
        Some(sp) => repo.revparse_single(sp)?.peel_to_commit()?,
        None => repo.head()?.peel_to_commit()?,
    };

    match repo.branch(name, &start_commit, force) {
        Ok(_) => {
            let refname = format!("refs/heads/{name}");
            repo.set_head(&refname)?;
            let mut co = CheckoutBuilder::new();
            repo.checkout_head(Some(&mut co))?;
            Ok(SwitchOutcome::CreatedAndSwitched)
        }
        Err(e) if !force && e.code() == ErrorCode::Exists => {
            Err(SgitError::Other(format!("branch '{name}' already exists")))
        }
        Err(e) => Err(e.into()),
    }
}

/// Detach HEAD at `target` in one repository.
///
/// - `repo`: repository to detach.
/// - `target`: commit-ish resolved to a commit object.
fn detach_to(repo: &Repository, target: &str) -> Result<()> {
    let obj = repo.revparse_single(target)?;
    let commit = obj.peel_to_commit()?;
    repo.set_head_detached(commit.id())?;
    let mut co = CheckoutBuilder::new();
    repo.checkout_head(Some(&mut co))?;
    Ok(())
}
