//! `sgit switch` - recursive equivalent of git switch.

use crate::RepoTree;
use crate::error::{Result, SgitError};
use crate::git::{Repo, SwitchCommand};

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
            match detach_to(r, target) {
                Ok(()) => println!("[{label}] Switched to detached HEAD at '{target}'"),
                Err(e) => eprintln!("[{label}] Error switching to detached HEAD: {e}"),
            }
        }
        return Ok(());
    }

    if let Some(name) = create {
        for r in tree.all() {
            let label = r.label();
            match create_and_switch(r, name, target, false) {
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
            match create_and_switch(r, name, target, true) {
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
        match switch_existing(r, branch) {
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
fn switch_existing(repo: &Repo, branch: &str) -> Result<SwitchOutcome> {
    if !repo.branch_exists(branch) {
        return Ok(SwitchOutcome::Missing);
    }
    repo.git(&SwitchCommand::branch(branch))?;
    Ok(SwitchOutcome::Switched)
}

/// Create (or optionally reset) a branch and switch to it.
///
/// - `repo`: repository to modify.
/// - `name`: branch name to create/switch.
/// - `start_point`: optional revision used as new branch start.
/// - `force`: when `true`, reset existing branch pointer.
fn create_and_switch(
    repo: &Repo,
    name: &str,
    start_point: Option<&str>,
    force: bool,
) -> Result<SwitchOutcome> {
    let existed = repo.branch_exists(name);
    if !force && existed {
        return Err(SgitError::Other(format!("branch '{name}' already exists")));
    }

    let cmd = if force {
        SwitchCommand::force_create(name, start_point)
    } else {
        SwitchCommand::create(name, start_point)
    };

    repo.git(&cmd)?;

    if existed {
        Ok(SwitchOutcome::Switched)
    } else {
        Ok(SwitchOutcome::CreatedAndSwitched)
    }
}

/// Detach HEAD at `target` in one repository.
///
/// - `repo`: repository to detach.
/// - `target`: commit-ish resolved to a commit object.
fn detach_to(repo: &Repo, target: &str) -> Result<()> {
    repo.git(&SwitchCommand::detach(target))
}
