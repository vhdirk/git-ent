//! `sgit push`- push across the whole tree via git CLI.

use crate::RepoTree;
use crate::error::Result;
use crate::git::{ListRemotesCommand, PushCommand, Repo, RevListCountCommand};

struct PushResult {
    ahead: usize,
    notices: Vec<String>,
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
                for notice in res.notices {
                    println!("[{label}] {notice}");
                }
            }
            Ok(None) => {}
            Err(e) => eprintln!("[{label}] Error pushing: {e}"),
        }
    }
    Ok(())
}

/// Push the current branch. Returns `Some(n)` if `n > 0` commits were
/// pushed, `None` if nothing needed pushing / no remote / detached HEAD.
fn push_one(r: &Repo, push_options: &[String]) -> Result<Option<PushResult>> {
    // Detached HEAD → nothing to push.
    let Some(branch_name) = r.branch_name() else {
        return Ok(None);
    };

    let remotes = r.git(&ListRemotesCommand)?;
    let Some(remote_name) = remotes.first() else {
        return Ok(None);
    };

    // Check how many commits are ahead of upstream
    let range = format!("{remote_name}/{branch_name}..{branch_name}");
    let ahead = match r.git(&RevListCountCommand::new(&range)) {
        Ok(count) => count,
        Err(_) => {
            // Upstream doesn't exist yet, count commits on local branch
            r.git(&RevListCountCommand::new(&branch_name)).unwrap_or(0)
        }
    };

    if ahead == 0 {
        return Ok(None);
    }

    let cmd = PushCommand {
        remote: Some(remote_name.clone()),
        branch: Some(branch_name.clone()),
        set_upstream: true,
        push_options: push_options.to_vec(),
    };

    let (_stdout, stderr) = r.git(&cmd)?;

    let mut notices = Vec::new();
    for line in stderr.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let lower = trimmed.to_ascii_lowercase();
            if lower.contains("http://")
                || lower.contains("https://")
                || lower.contains("merge request")
            {
                notices.push(trimmed.to_string());
            }
        }
    }

    Ok(Some(PushResult { ahead, notices }))
}
