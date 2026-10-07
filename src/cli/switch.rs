use crate::RepoTree;
use crate::cli::command::{Cmd, Context};
use crate::error::{GitEntError, Result};
use clap::Args;
use git2::{BranchType, ErrorCode, Repository, build::CheckoutBuilder};

/// Switch branches recursively across all repos.
#[derive(Default, Debug, Args)]
pub struct SwitchCmd {
    /// Create and switch to a new branch in every repo.
    #[arg(short = 'c', long = "create", conflicts_with_all = ["force_create", "detach"])]
    pub create: Option<String>,
    /// Reset or create the branch, then switch to it in every repo.
    #[arg(short = 'C', long = "force-create", conflicts_with_all = ["create", "detach"])]
    pub force_create: Option<String>,
    /// Switch to a detached HEAD at the target commit-ish.
    #[arg(long = "detach", conflicts_with_all = ["create", "force_create"])]
    pub detach: bool,
    /// Existing branch to switch to, or start-point for `-c`/`-C`.
    pub target: Option<String>,
}

impl Cmd for SwitchCmd {
    fn run(&self, _ctx: &Context) -> Result<()> {
        let tree = RepoTree::discover(None)?;

        if self.detach {
            let Some(target) = self.target.as_deref() else {
                return Err(GitEntError::InvalidUsage(
                    "usage: git-ent switch --detach <commit-ish>".into(),
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

        if let Some(name) = self.create.as_deref() {
            for r in tree.all() {
                let label = r.label();
                match create_and_switch(&r.repo, name, self.target.as_deref(), false) {
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

        if let Some(name) = self.force_create.as_deref() {
            for r in tree.all() {
                let label = r.label();
                match create_and_switch(&r.repo, name, self.target.as_deref(), true) {
                    Ok(SwitchOutcome::CreatedAndSwitched) => {
                        println!("[{label}] Created and switched to '{name}'")
                    }
                    Ok(SwitchOutcome::Switched) => {
                        println!("[{label}] Reset and switched to '{name}'")
                    }
                    Ok(SwitchOutcome::Missing) => unreachable!(),
                    Err(e) => eprintln!("[{label}] Error resetting/switching to '{name}': {e}"),
                }
            }
            return Ok(());
        }

        let Some(branch) = self.target.as_deref() else {
            return Err(GitEntError::InvalidUsage(
            "usage: git-ent switch <branch> | git-ent switch -c <new-branch> [<start-point>] | git-ent switch -C <branch> [<start-point>] | git-ent switch --detach <commit-ish>".into(),
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

    let refname = br.get().name().map_err(GitEntError::from)?;
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
        Err(e) if !force && e.code() == ErrorCode::Exists => Err(GitEntError::Other(format!(
            "branch '{name}' already exists"
        ))),
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
