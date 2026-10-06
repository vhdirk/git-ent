//! `sgit squash <branch>` - collapse commits since the merge-base, depth-first.
//!
//! Mirrors GitHub/GitLab's "squash and merge" but nests naturally across
//! submodules: each submodule is squashed first, then the parent picks up
//! the moved submodule pointer as part of its own squash commit.

use git2::{BranchType, Oid, Repository, ResetType};

use crate::RepoTree;
use crate::cmd::command::{Command, Context};
use crate::cmd::reset::reset_to;
use crate::error::{Result, SgitError};
use crate::git::signature;
use crate::repo::Repo;
use crate::repo_tree::changed_submodule_paths;
use clap::Args;

/// Squash all commits since the common ancestor with <branch>.
///
/// Mirrors GitHub/GitLab's "squash and merge" semantics but depth-first
/// across submodules: each submodule is squashed first, then the
/// updated submodule pointer is rolled into the parent's squash commit.
#[derive(Default, Debug, Args)]
pub struct SquashCmd {
    /// The base branch whose merge-base defines the squash range.
    pub branch: String,
    /// Commit message for the squash. Defaults to the concatenated
    /// subjects of the squashed commits.
    #[arg(short = 'm', long = "message")]
    pub message: Option<String>,
}

impl Command for SquashCmd {
    /// Squash commits since merge-base against `branch` in every repo.
    ///
    /// - `branch`: target branch used to compute merge-base.
    /// - `message`: optional explicit squash commit message.
    fn run(&self, ctx: &Context) -> Result<()> {
        let tree = RepoTree::discover(ctx.workdir.as_deref())?;

        for r in tree.all() {
            let label = r.label();
            match squash_one(r, &self.branch, self.message.as_deref()) {
                Ok(SquashOutcome::Skipped(reason)) => {
                    println!("[{label}] Skipping: {reason}");
                }
                Ok(SquashOutcome::Nothing) => {
                    println!("[{label}] Nothing to squash");
                }
                Ok(SquashOutcome::Squashed(n)) => {
                    println!("[{label}] Squashed {n} commit(s)");
                }
                Ok(SquashOutcome::PointerOnly) => {
                    println!("[{label}] Committed updated submodule ref(s)");
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

enum SquashOutcome {
    Skipped(String),
    Nothing,
    Squashed(usize),
    PointerOnly,
}

/// Squash one repo's commits since merge-base against `branch`.
///
/// - `r`: repository handle to squash.
/// - `branch`: target branch used to compute merge-base.
/// - `message`: optional explicit squash commit message.
fn squash_one(r: &Repo, branch: &str, message: Option<&str>) -> Result<SquashOutcome> {
    let repo = &r.repo;

    // Detached --> skip.
    if repo.head_detached().unwrap_or(false) {
        return Ok(SquashOutcome::Skipped("detached HEAD".into()));
    }

    // Target branch must exist.
    if repo.find_branch(branch, BranchType::Local).is_err() {
        return Ok(SquashOutcome::Skipped(format!(
            "branch '{branch}' does not exist"
        )));
    }

    // Current branch.
    let head_ref = repo.head()?;
    let current = head_ref.shorthand().unwrap_or("").to_string();
    if current == branch {
        return Ok(SquashOutcome::Skipped(format!("already on '{branch}'")));
    }

    let head_oid = head_ref
        .target()
        .ok_or_else(|| SgitError::Other("HEAD has no target".into()))?;
    let branch_oid = repo
        .find_branch(branch, BranchType::Local)?
        .get()
        .target()
        .ok_or_else(|| SgitError::Other(format!("branch '{branch}' has no target")))?;

    // Find merge-base.
    let base: Oid = match repo.merge_base(head_oid, branch_oid) {
        Ok(o) => o,
        Err(_) => {
            return Ok(SquashOutcome::Skipped(format!(
                "no common ancestor with '{branch}'"
            )));
        }
    };

    // Commits to squash = base..HEAD
    let commits = commits_between(repo, base, head_oid)?;

    // Auto-stage any dirty submodule pointers *after* the soft-reset, but
    // first we need the list for this path - query pre-reset too.
    let has_sub_changes_pre = !changed_submodule_paths(repo)?.is_empty();

    if commits.is_empty() {
        if has_sub_changes_pre {
            // No own commits to squash, but submodule pointer moved --> create
            // a pointer-only commit on the current branch.
            record_pointer_commit(repo)?;
            return Ok(SquashOutcome::PointerOnly);
        }
        return Ok(SquashOutcome::Nothing);
    }

    // Soft-reset to merge-base, keeping the index + worktree as-is.
    reset_to(repo, &base.to_string(), ResetType::Soft)?;

    // Stage any submodule pointer changes so they land in the squash commit.
    let subs = changed_submodule_paths(repo)?;
    if !subs.is_empty() {
        let mut index = repo.index()?;
        for p in &subs {
            index.add_path(std::path::Path::new(p))?;
        }
        index.write()?;
    }

    // Build the squash message.
    let msg = match message {
        Some(m) => m.to_string(),
        None => default_message(repo, &commits)?,
    };

    // Create the squash commit.
    let sig = signature(repo)?;
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let parent = repo.find_commit(base)?;
    repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&parent])?;
    Ok(SquashOutcome::Squashed(commits.len()))
}

/// Return commit OIDs in `base..head` for one repository.
///
/// - `repo`: repository used for revision walk.
/// - `base`: lower bound (excluded).
/// - `head`: upper bound (included).
fn commits_between(repo: &Repository, base: Oid, head: Oid) -> Result<Vec<Oid>> {
    if base == head {
        return Ok(Vec::new());
    }
    let mut walk = repo.revwalk()?;
    walk.push(head)?;
    walk.hide(base)?;
    let mut out = Vec::new();
    for oid in walk {
        out.push(oid?);
    }
    Ok(out)
}

/// Build a default squash message from commit subjects.
///
/// - `repo`: repository from which commit metadata is loaded.
/// - `commits`: commit IDs included in the squash.
fn default_message(repo: &Repository, commits: &[Oid]) -> Result<String> {
    let mut lines = Vec::with_capacity(commits.len());
    for oid in commits {
        let c = repo.find_commit(*oid)?;
        let subj = c.summary().ok().flatten().unwrap_or("").to_string();
        lines.push(format!("* {subj}"));
    }
    Ok(format!("Squashed commits:\n\n{}\n", lines.join("\n")))
}

/// Create a pointer-only commit when only submodule refs changed.
///
/// - `repo`: parent repo receiving the pointer update commit.
fn record_pointer_commit(repo: &Repository) -> Result<()> {
    let changed = changed_submodule_paths(repo)?;
    if changed.is_empty() {
        return Ok(());
    }
    let mut index = repo.index()?;
    for p in &changed {
        index.add_path(std::path::Path::new(p))?;
    }
    index.write()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let sig = signature(repo)?;
    let parent = repo.head()?.peel_to_commit()?;
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Update submodule refs",
        &tree,
        &[&parent],
    )?;
    Ok(())
}
