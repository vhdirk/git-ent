use crate::RepoTree;
use crate::cli::command::{Cmd, Context};
use crate::error::Result;
use crate::git::first_remote;
use crate::repo::Repo;
use clap::Args;
use git2::{BranchType, Cred, CredentialType, PushOptions, RemoteCallbacks, Repository};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Push to remote across all repos that have commits to push.
#[derive(Default, Debug, Args)]
pub struct PushCmd {
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    #[arg(short = 'd', long = "delete")]
    pub delete: bool,

    #[arg(long = "prune")]
    pub prune: bool,

    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    #[arg(short = 'u', long = "set-upstream")]
    pub set_upstream: bool,

    /// Push tags as well.
    #[arg(long = "tags")]
    pub tags: bool,

    /// Push options (`-o`). Can be specified multiple times.
    #[arg(short = 'o', long = "push-option")]
    pub option: Vec<String>,

    #[arg(long = "no-verify")]
    pub no_verify: bool,
}

impl Cmd for PushCmd {
    /// Iterate the repo tree and push each eligible repo.
    ///
    /// - `push_options`: push options forwarded to each remote push call.
    fn run(&self, _ctx: &Context) -> Result<()> {
        let opts: Vec<&str> = self.option.iter().map(String::as_str).collect();

        let tree = RepoTree::discover(None)?;
        for r in tree.all() {
            let label = r.label();
            match push_one(r, &opts, self) {
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
}

struct PushResult {
    ahead: usize,
    notices: Vec<String>,
}

/// Push the current branch. Returns `Some(n)` if `n > 0` commits were
/// pushed, `None` if nothing needed pushing / no remote / detached HEAD.
fn push_one(r: &Repo, push_options: &[&str], args: &PushCmd) -> Result<Option<PushResult>> {
    let repo = &r.repo;

    // Detached HEAD --> nothing to push.
    if repo.head_detached().unwrap_or(false) {
        return Ok(None);
    }
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(None),
    };
    let branch_name = match head.shorthand() {
        Ok(branch) => branch.to_string(),
        Err(_) => return Ok(None),
    };

    let remote_name = match first_remote(repo) {
        Some(n) => n,
        None => return Ok(None),
    };

    // How many commits are ahead of upstream?
    let ahead = commits_ahead(repo, &branch_name, &remote_name).unwrap_or(0);
    if ahead == 0 {
        println!("[{}] Everything up-to-date", r.label());
        return Ok(None);
    }

    let mut remote = repo.find_remote(&remote_name)?;

    let sideband_lines = Arc::new(Mutex::new(Vec::<String>::new()));
    let sideband_capture = Arc::clone(&sideband_lines);

    let mut callbacks = RemoteCallbacks::new();

    // TODO: first collect all sideband lines, THEN split into lines
    callbacks.sideband_progress(move |data| {
        if let Ok(text) = std::str::from_utf8(data) {
            if let Ok(mut lines) = sideband_capture.lock() {
                for line in text.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        lines.push(trimmed.to_string());
                    }
                }
            }
        }
        true
    });

    callbacks.credentials(|url, username_from_url, allowed| {
        dbg!(
            "credentials callback called with url: {}, username_from_url: {:?}, allowed: {:?}",
            url,
            username_from_url,
            allowed
        );
        // SSH: auth via agent or standard key files (~/.ssh/id_*)
        // libgit2 requires a callback for SSH; this callback follows standard SSH behavior.
        if allowed.contains(CredentialType::SSH_KEY) {
            let user = username_from_url.unwrap_or("git");
            // Try SSH agent first
            if let Ok(cred) = Cred::ssh_key_from_agent(user) {
                return Ok(cred);
            }
            // Fallback: try standard SSH key locations (matches OpenSSH's default search)
            for key_name in &["id_ed25519", "id_rsa", "id_ecdsa"] {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let key_path = PathBuf::from(format!("{}/.ssh/{}", home, key_name));
                if key_path.exists() {
                    if let Ok(cred) = Cred::ssh_key(user, None, &key_path, None) {
                        return Ok(cred);
                    }
                }
            }
        }
        // HTTPS: use credential.helper
        if allowed.contains(CredentialType::USER_PASS_PLAINTEXT) {
            if let Ok(cfg) = repo.config() {
                if let Ok(cred) = Cred::credential_helper(&cfg, url, username_from_url) {
                    return Ok(cred);
                }
            }
        }
        // Username-only
        if allowed.contains(CredentialType::USERNAME) {
            return Cred::username(username_from_url.unwrap_or("git"));
        }
        Cred::default()
    });

    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(callbacks);
    if !push_options.is_empty() {
        push_opts.remote_push_options(push_options);
    }

    let refspec = format!("refs/heads/{branch_name}:refs/heads/{branch_name}");
    remote.push(&[&refspec], Some(&mut push_opts))?;

    let notices = sideband_lines
        .lock()
        .map(|lines| extract_push_notices(&lines))
        .unwrap_or_default();

    // Ensure upstream tracking is set.
    if args.set_upstream {
        if let Ok(mut br) = repo.find_branch(&branch_name, BranchType::Local) {
            let upstream = format!("{remote_name}/{branch_name}");
            let _ = br.set_upstream(Some(&upstream));
        }
    }

    Ok(Some(PushResult { ahead, notices }))
}

/// Keep sideband lines that are likely useful user notices.
///
/// - `lines`: raw sideband lines captured from remote output.
fn extract_push_notices(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            lower.contains("http://")
                || lower.contains("https://")
                || lower.contains("merge request")
        })
        .cloned()
        .collect()
}

/// Count local commits ahead of `<remote>/<branch>`; returns 0 if the
/// upstream doesn't exist yet.
fn commits_ahead(repo: &Repository, branch: &str, remote: &str) -> Result<usize> {
    let local = match repo.refname_to_id(&format!("refs/heads/{branch}")) {
        Ok(o) => o,
        Err(_) => return Ok(0),
    };
    let upstream = match repo.refname_to_id(&format!("refs/remotes/{remote}/{branch}")) {
        Ok(o) => o,
        Err(_) => {
            // No upstream yet --> count commits unique to local via revwalk from local.
            let mut walk = repo.revwalk()?;
            walk.push(local)?;
            return Ok(walk.count());
        }
    };
    let (ahead, _behind) = repo.graph_ahead_behind(local, upstream)?;
    Ok(ahead)
}

/// Shared credentials callback factory for clone/update.
/// libgit2 requires a callback for SSH to work; this implementation follows standard SSH behavior.
pub fn remote_callbacks<'a>() -> RemoteCallbacks<'a> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(|_url, username_from_url, allowed| {
        // SSH: try agent then standard key locations
        if allowed.contains(CredentialType::SSH_KEY) {
            let user = username_from_url.unwrap_or("git");
            if let Ok(cred) = Cred::ssh_key_from_agent(user) {
                return Ok(cred);
            }
            for key_name in &["id_ed25519", "id_rsa", "id_ecdsa"] {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let key_path = PathBuf::from(format!("{}/.ssh/{}", home, key_name));
                if key_path.exists() {
                    if let Ok(cred) = Cred::ssh_key(user, None, &key_path, None) {
                        return Ok(cred);
                    }
                }
            }
        }
        // HTTPS: use credential.helper
        if allowed.contains(CredentialType::USER_PASS_PLAINTEXT) {
            if let Ok(repo) = Repository::open(".") {
                if let Ok(cfg) = repo.config() {
                    if let Ok(cred) = Cred::credential_helper(&cfg, _url, username_from_url) {
                        return Ok(cred);
                    }
                }
            }
        }
        if allowed.contains(CredentialType::USERNAME) {
            return Cred::username(username_from_url.unwrap_or("git"));
        }
        Cred::default()
    });
    cb
}
