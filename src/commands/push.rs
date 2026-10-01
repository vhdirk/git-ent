//! `sgit push` - push across the whole tree via git CLI.

use crate::RepoTree;
use crate::error::Result;
use crate::git::push::{PushCmd, PushOutput};
use crate::repo::Repo;

struct PushResult {
    ahead: usize,
    output: PushOutput,
}

/// Push recursively with user-provided push options.
///
/// - `push_option`: repeatable options passed through to remote push.
pub fn run(push_option: &[String]) -> Result<()> {
    let opts: Vec<String> = push_option.to_vec();
    push_tree(&opts)
}

/// Iterate the repo tree and push each eligible repo.
///
/// - `push_options`: push options forwarded to each remote push call.
fn push_tree(push_options: &[String]) -> Result<()> {
    let tree = RepoTree::discover(None)?;
    for r in tree.all() {
        let label = r.label();
        match push_one(r, push_options) {
            Ok(Some(res)) => {
                println!("[{label}] Pushed {} commit(s)", res.ahead);
                for notice in res.output.notices {
                    println!("[{label}] {notice}");
                }
            }
            Ok(None) => {}
            Err(e) => eprintln!("[{label}] Error pushing: {e}"),
        }
    }
    Ok(())
}

/// Push the current branch. Returns `Some(PushResult)` if `ahead > 0` commits were
/// pushed, `None` if nothing needed pushing / no remote / detached HEAD.
fn push_one(r: &Repo, push_options: &[String]) -> Result<Option<PushResult>> {
    // Detached HEAD -> nothing to push.
    let Some(branch_name) = r.branch_name() else {
        return Ok(None);
    };

    let remotes = r.remotes()?;
    let Some(remote_name) = remotes.first() else {
        return Ok(None);
    };

    let ahead = r.commits_ahead(&branch_name, remote_name).unwrap_or(0);
    if ahead == 0 {
        return Ok(None);
    }

    let cmd = PushCmd {
        remote: Some(remote_name.clone()),
        branch: Some(branch_name.clone()),
        set_upstream: true,
        push_options: push_options.to_vec(),
        ..Default::default()
    };

    let output = r.git(&cmd)?;

    Ok(Some(PushResult { ahead, output }))
}
