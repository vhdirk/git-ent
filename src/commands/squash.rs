//! `sgit squash <branch>` - collapse commits since the merge-base, depth-first.
//!
//! Mirrors GitHub/GitLab's "squash and merge" but nests naturally across
//! submodules: each submodule is squashed first, then the parent picks up
//! the moved submodule pointer as part of its own squash commit.

use std::path::PathBuf;

use crate::RepoTree;
use crate::error::Result;
use crate::git::{
    AddCommand, CommitCommand, CommitSubjectsCommand, MergeBaseCommand, Repo, ResetMode,
    RevListCountCommand,
};
use crate::repo_tree::changed_submodule_paths;

/// Squash commits since merge-base against `branch` in every repo.
///
/// - `branch`: target branch used to compute merge-base.
/// - `message`: optional explicit squash commit message.
pub fn run(branch: &str, message: Option<&str>) -> Result<()> {
    let tree = RepoTree::discover(None)?;

    for r in tree.all() {
        let label = r.label();
        match squash_one(r, branch, message) {
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
    // Current branch name
    let Some(current) = r.branch_name() else {
        return Ok(SquashOutcome::Skipped("detached HEAD".into()));
    };

    // Target branch must exist.
    if !r.branch_exists(branch) {
        return Ok(SquashOutcome::Skipped(format!(
            "branch '{branch}' does not exist"
        )));
    }

    if current == branch {
        return Ok(SquashOutcome::Skipped(format!("already on '{branch}'")));
    }

    // Find merge-base.
    let base = match r.git(&MergeBaseCommand::new("HEAD", branch)) {
        Ok(b) if !b.is_empty() => b,
        _ => {
            return Ok(SquashOutcome::Skipped(format!(
                "no common ancestor with '{branch}'"
            )));
        }
    };

    let range = format!("{base}..HEAD");
    let commits_count = r.git(&RevListCountCommand::new(&range))?;

    // Auto-stage any dirty submodule pointers *after* the soft-reset, but
    // first we need the list for this path - query pre-reset too.
    let has_sub_changes_pre = !changed_submodule_paths(r)?.is_empty();

    if commits_count == 0 {
        if has_sub_changes_pre {
            record_pointer_commit(r)?;
            return Ok(SquashOutcome::PointerOnly);
        }
        return Ok(SquashOutcome::Nothing);
    }

    // Build the squash message before moving HEAD.
    let msg = match message {
        Some(m) => m.to_string(),
        None => {
            let subjects = r.git(&CommitSubjectsCommand::new(&range))?;
            format!("Squashed commits:\n\n{}\n", subjects.join("\n"))
        }
    };

    // Soft-reset to merge-base, keeping the index + worktree as-is.
    r.reset(ResetMode::Soft, Some(&base))?;

    // Stage any submodule pointer changes so they land in the squash commit.
    let subs = changed_submodule_paths(r)?;
    if !subs.is_empty() {
        r.git(&AddCommand::Paths(subs.iter().map(PathBuf::from).collect()))?;
    }

    // Create the squash commit.
    r.git(&CommitCommand {
        message: msg,
        no_verify: false,
    })?;

    Ok(SquashOutcome::Squashed(commits_count))
}

/// Create a pointer-only commit when only submodule refs changed.
///
/// - `repo`: parent repo receiving the pointer update commit.
fn record_pointer_commit(r: &Repo) -> Result<()> {
    let changed = changed_submodule_paths(r)?;
    if changed.is_empty() {
        return Ok(());
    }
    r.git(&AddCommand::Paths(
        changed.iter().map(PathBuf::from).collect(),
    ))?;
    r.git(&CommitCommand {
        message: "Update submodule refs".to_string(),
        no_verify: false,
    })?;
    Ok(())
}
