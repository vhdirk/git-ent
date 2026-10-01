use std::ffi::OsString;
use std::process::Output;
use std::str::FromStr;

use crate::SgitError;
use crate::error::Result;
use crate::git::{ParseOutput, ToArgs};

/// Flag describing the kind of ref update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefUpdateFlag {
    FastForward,
    ForcedUpdate,
    NewRef,
    Deleted,
    Pruned,
    Rejected,
    UpToDate,
    Other,
}

impl FromStr for RefUpdateFlag {
    type Err = SgitError;
    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "*" => RefUpdateFlag::NewRef,
            "+" => RefUpdateFlag::ForcedUpdate,
            "-" => RefUpdateFlag::Deleted,
            "!" => RefUpdateFlag::Rejected,
            "=" => RefUpdateFlag::UpToDate,
            _ => RefUpdateFlag::Other,
        })
    }
}

/// A parsed ref update line from `git push`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefUpdate {
    pub flag: RefUpdateFlag,
    pub summary: String,
    pub from: String,
    pub to: String,
    pub reason: Option<String>,
}

/// Structured representation of `git push` output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PushOutput {
    /// Remote destination (e.g., `git@github.com:...` or local bare path).
    pub destination: Option<String>,
    /// True if git push reported that everything is up to date.
    pub up_to_date: bool,
    /// Ref updates parsed from output.
    pub ref_updates: Vec<RefUpdate>,
    /// Informational notices from remote (e.g., server hooks, MR instructions).
    pub notices: Vec<String>,
    /// All HTTP/HTTPS URLs extracted from remote notices.
    pub urls: Vec<String>,
    /// URLs specifically referring to merge requests or pull requests.
    pub merge_request_urls: Vec<String>,
    /// Raw stdout.
    pub raw_stdout: String,
    /// Raw stderr.
    pub raw_stderr: String,
}

impl PushOutput {
    /// Parse `git push` standard output and standard error into structured data.
    pub fn parse(stdout: &str, stderr: &str) -> Self {
        let mut destination = None;
        let mut up_to_date = false;
        let mut ref_updates = Vec::new();
        let mut notices = Vec::new();
        let mut urls = Vec::new();
        let mut merge_request_urls = Vec::new();

        let combined = format!("{stdout}\n{stderr}");

        for line in combined.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.contains("Everything up-to-date") {
                up_to_date = true;
            }

            if let Some(dest) = trimmed.strip_prefix("To ") {
                destination = Some(dest.trim().to_string());
                continue;
            }

            // Check for ref updates
            if let Some(update) = parse_ref_update_line(trimmed) {
                if update.flag == RefUpdateFlag::UpToDate {
                    up_to_date = true;
                }
                ref_updates.push(update);
                continue;
            }

            // Check for remote notices and URLs
            let is_remote_line = trimmed.starts_with("remote:");
            let content_without_remote = if is_remote_line {
                trimmed.strip_prefix("remote:").unwrap().trim()
            } else {
                trimmed
            };

            let lower = content_without_remote.to_ascii_lowercase();

            // Extract URLs from line
            let extracted_urls = extract_urls(content_without_remote);
            for url in &extracted_urls {
                if !urls.contains(url) {
                    urls.push(url.clone());
                }
                if (is_mr_or_pr(content_without_remote) || is_mr_or_pr(url))
                    && !merge_request_urls.contains(url)
                {
                    merge_request_urls.push(url.clone());
                }
            }

            if is_remote_line {
                // Ignore standard Git transfer progress counters
                let is_progress = lower.starts_with("counting objects")
                    || lower.starts_with("compressing objects")
                    || lower.starts_with("writing objects")
                    || lower.starts_with("resolving deltas")
                    || lower.starts_with("enumerating objects")
                    || lower.starts_with("total ");

                if !is_progress && !content_without_remote.is_empty() {
                    notices.push(content_without_remote.to_string());
                }
            } else if is_mr_or_pr(trimmed) || !extracted_urls.is_empty() {
                notices.push(trimmed.to_string());
            }
        }

        Self {
            destination,
            up_to_date,
            ref_updates,
            notices,
            urls,
            merge_request_urls,
            raw_stdout: stdout.to_string(),
            raw_stderr: stderr.to_string(),
        }
    }
}

fn extract_urls(text: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for word in text.split_whitespace() {
        let trimmed =
            word.trim_matches(&['(', ')', '[', ']', '{', '}', '<', '>', '\'', '"', ',', ';'][..]);
        let trimmed = trimmed.trim_end_matches('.');
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            urls.push(trimmed.to_string());
        }
    }
    urls
}

fn is_mr_or_pr(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("merge request")
        || lower.contains("merge_request")
        || lower.contains("merge_requests")
        || lower.contains("pull request")
        || lower.contains("/pull/")
        || lower.contains("/pulls/")
        || lower.contains("/pull-requests/")
}

fn parse_ref_update_line(line: &str) -> Option<RefUpdate> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check porcelain format: <flag>\t<from>:<to>\t<summary>
    if line.contains('\t') {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let flag_char = parts[0].trim();
            let refs = parts[1].trim();
            let summary = parts[2].trim();

            let (from, to) = if let Some(colon) = refs.find(':') {
                (&refs[..colon], &refs[colon + 1..])
            } else {
                (refs, refs)
            };

            let flag = match flag_char {
                "*" => RefUpdateFlag::NewRef,
                "+" => RefUpdateFlag::ForcedUpdate,
                "-" => RefUpdateFlag::Deleted,
                "!" => RefUpdateFlag::Rejected,
                "=" => RefUpdateFlag::UpToDate,
                _ => {
                    if summary.contains("..") {
                        RefUpdateFlag::FastForward
                    } else {
                        RefUpdateFlag::Other
                    }
                }
            };

            return Some(RefUpdate {
                flag,
                summary: summary.to_string(),
                from: from.to_string(),
                to: to.to_string(),
                reason: None,
            });
        }
    }

    // Check standard human-readable format:
    // e.g. " * [new branch]      main -> main"
    //      "   c82f2ae..f5fd671  main -> main"
    //      " + 1234567...89abcde feature -> feature (forced update)"
    //      " ! [rejected]        main -> main (non-fast-forward)"
    //      " = [up to date]      main -> main"
    if trimmed.contains("->") {
        let (left, right) = trimmed.split_once("->")?;
        let left = left.trim();
        let right = right.trim();

        let (flag, rest_left) = match left.chars().next() {
            Some(c @ ('*' | '+' | '-' | '!' | '=')) => {
                let f = RefUpdateFlag::from_str(&c.to_string()).unwrap();
                (f, left[c.len_utf8()..].trim())
            }
            _ => (RefUpdateFlag::FastForward, left),
        };

        let (summary, from) = if rest_left.starts_with('[') {
            let end = rest_left.find(']')?;
            let summary = &rest_left[..=end];
            let from = rest_left[end + 1..].trim();
            (summary, from)
        } else {
            let mut parts = rest_left.split_whitespace();
            let summary = parts.next()?;
            let from = parts.next().unwrap_or(summary);
            (summary, from)
        };

        let (to, reason) = if let Some(start_paren) = right.find('(') {
            let to = right[..start_paren].trim();
            let reason = right[start_paren + 1..].trim_end_matches(')').trim();
            (to, Some(reason.to_string()))
        } else {
            let to = right.split_whitespace().next().unwrap_or(right);
            (to, None)
        };

        return Some(RefUpdate {
            flag,
            summary: summary.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            reason,
        });
    }

    // Check deleted ref without "->": " - [deleted]         feature"
    if trimmed.contains("[deleted]") {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if let Some(&name) = parts.last() {
            if name != "[deleted]" {
                return Some(RefUpdate {
                    flag: RefUpdateFlag::Deleted,
                    summary: "[deleted]".to_string(),
                    from: name.to_string(),
                    to: name.to_string(),
                    reason: None,
                });
            }
        }
    }

    None
}

/// Recurse submodules mode for git push (`--recurse-submodules=check|on-demand|only|no`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecurseSubmodules {
    Check,
    OnDemand,
    Only,
    No,
}

/// GPG signing mode for git push (`--signed[=(yes|no|if-asked)]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignedMode {
    Yes,
    No,
    IfAsked,
}

/// Git push command options conforming to https://git-scm.com/docs/git-push.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PushCmd {
    /// Remote repository name or URL to push to.
    pub repository: Option<String>,
    /// Refspecs to push (e.g. `main`, `HEAD:refs/heads/feature`, `:old-branch`).
    pub refspecs: Vec<String>,
    /// Backward-compatibility alias for `repository`.
    pub remote: Option<String>,
    /// Backward-compatibility alias for `refspecs`.
    pub branch: Option<String>,
    /// Push all branches (`--all`).
    pub all: bool,
    /// Alias of `--all` (`--branches`).
    pub branches: bool,
    /// Mirror all refs (`--mirror`).
    pub mirror: bool,
    /// Delete all listed refs (`-d`, `--delete`).
    pub delete: bool,
    /// Push all tags (`--tags`).
    pub tags: bool,
    /// Push missing but relevant tags (`--follow-tags` / `--no-follow-tags`).
    pub follow_tags: Option<bool>,
    /// Request atomic transaction on remote side (`--atomic` / `--no-atomic`).
    pub atomic: Option<bool>,
    /// Dry run (`-n`, `--dry-run`).
    pub dry_run: bool,
    /// Machine-readable porcelain output (`--porcelain`).
    pub porcelain: bool,
    /// Force updates (`-f`, `--force`).
    pub force: bool,
    /// Require old value of ref to match (`--force-with-lease[=<refname>[:<expect>]]`).
    pub force_with_lease: Option<Option<String>>,
    /// Require remote updates to be integrated locally (`--force-if-includes`).
    pub force_if_includes: bool,
    /// Control recursive pushing of submodules (`--recurse-submodules=check|on-demand|only|no`).
    pub recurse_submodules: Option<RecurseSubmodules>,
    /// Use thin pack (`--thin` / `--no-thin`).
    pub thin: Option<bool>,
    /// Receive pack program on the remote (`--receive-pack=<git-receive-pack>` / `--exec=<git-receive-pack>`).
    pub receive_pack: Option<String>,
    /// Set upstream for git pull/status (`-u`, `--set-upstream`).
    pub set_upstream: bool,
    /// Force progress reporting (`--progress` / `--no-progress`).
    pub progress: Option<bool>,
    /// Prune locally removed refs on remote (`--prune`).
    pub prune: bool,
    /// Bypass pre-push hook (`--no-verify`).
    pub no_verify: bool,
    /// Opposite of `--no-verify` (`--verify`).
    pub verify: bool,
    /// GPG sign the push (`--signed[=(yes|no|if-asked)]` / `--no-signed`).
    pub signed: Option<Option<SignedMode>>,
    /// Option to transmit (`-o`, `--push-option=<option>`).
    pub push_options: Vec<String>,
    /// Suppress non-error messages (`-q`, `--quiet`).
    pub quiet: bool,
    /// Be verbose (`-v`, `--verbose`).
    pub verbose: bool,
    /// Default repository (`--repo=<repository>`).
    pub repo: Option<String>,
    /// Use IPv4 addresses only (`-4`, `--ipv4`).
    pub ipv4: bool,
    /// Use IPv6 addresses only (`-6`, `--ipv6`).
    pub ipv6: bool,
}

impl PushCmd {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upstream(remote: impl Into<String>, branch: impl Into<String>) -> Self {
        let r = remote.into();
        let b = branch.into();
        Self {
            repository: Some(r.clone()),
            remote: Some(r),
            refspecs: vec![b.clone()],
            branch: Some(b),
            set_upstream: true,
            ..Default::default()
        }
    }

    pub fn delete(remote: impl Into<String>, refspec: impl Into<String>) -> Self {
        let r = remote.into();
        let s = refspec.into();
        Self {
            repository: Some(r.clone()),
            remote: Some(r),
            refspecs: vec![s.clone()],
            branch: Some(s),
            delete: true,
            ..Default::default()
        }
    }
}

impl ToArgs for PushCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("push".into());
        if self.all {
            args.push("--all".into());
        }
        if self.branches {
            args.push("--branches".into());
        }
        if self.mirror {
            args.push("--mirror".into());
        }
        if self.delete {
            args.push("--delete".into());
        }
        if self.tags {
            args.push("--tags".into());
        }
        match self.follow_tags {
            Some(true) => args.push("--follow-tags".into()),
            Some(false) => args.push("--no-follow-tags".into()),
            None => {}
        }
        match self.atomic {
            Some(true) => args.push("--atomic".into()),
            Some(false) => args.push("--no-atomic".into()),
            None => {}
        }
        if self.dry_run {
            args.push("--dry-run".into());
        }
        if self.porcelain {
            args.push("--porcelain".into());
        }
        if self.force {
            args.push("--force".into());
        }
        if let Some(lease) = &self.force_with_lease {
            match lease {
                Some(spec) => args.push(format!("--force-with-lease={spec}").into()),
                None => args.push("--force-with-lease".into()),
            }
        }
        if self.force_if_includes {
            args.push("--force-if-includes".into());
        }
        if let Some(recurse) = self.recurse_submodules {
            let val = match recurse {
                RecurseSubmodules::Check => "check",
                RecurseSubmodules::OnDemand => "on-demand",
                RecurseSubmodules::Only => "only",
                RecurseSubmodules::No => "no",
            };
            args.push(format!("--recurse-submodules={val}").into());
        }
        match self.thin {
            Some(true) => args.push("--thin".into()),
            Some(false) => args.push("--no-thin".into()),
            None => {}
        }
        if let Some(rp) = &self.receive_pack {
            args.push(format!("--receive-pack={rp}").into());
        }
        if self.set_upstream {
            args.push("--set-upstream".into());
        }
        match self.progress {
            Some(true) => args.push("--progress".into()),
            Some(false) => args.push("--no-progress".into()),
            None => {}
        }
        if self.prune {
            args.push("--prune".into());
        }
        if self.no_verify {
            args.push("--no-verify".into());
        }
        if self.verify {
            args.push("--verify".into());
        }
        if let Some(signed) = self.signed {
            match signed {
                Some(SignedMode::Yes) => args.push("--signed=yes".into()),
                Some(SignedMode::No) => args.push("--signed=no".into()),
                Some(SignedMode::IfAsked) => args.push("--signed=if-asked".into()),
                None => args.push("--signed".into()),
            }
        }
        for opt in &self.push_options {
            args.push("-o".into());
            args.push(opt.into());
        }
        if self.quiet {
            args.push("--quiet".into());
        }
        if self.verbose {
            args.push("--verbose".into());
        }
        if let Some(repo) = &self.repo {
            args.push(format!("--repo={repo}").into());
        }
        if self.ipv4 {
            args.push("-4".into());
        }
        if self.ipv6 {
            args.push("-6".into());
        }

        let repo_target = self.repository.as_ref().or(self.remote.as_ref());
        if let Some(target) = repo_target {
            args.push(target.into());
        }

        if !self.refspecs.is_empty() {
            for spec in &self.refspecs {
                args.push(spec.into());
            }
        } else if let Some(branch) = &self.branch {
            args.push(branch.into());
        }
    }
}

impl ParseOutput for PushCmd {
    type Output = PushOutput;

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(PushOutput::parse(&stdout, &stderr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_new_branch_output() {
        let stdout = "branch 'main' set up to track 'origin/main'.\n";
        let stderr = "To /tmp/bare.git\n * [new branch]      main -> main\n";
        let output = PushOutput::parse(stdout, stderr);

        assert_eq!(output.destination, Some("/tmp/bare.git".to_string()));
        assert_eq!(output.ref_updates.len(), 1);
        assert_eq!(output.ref_updates[0].flag, RefUpdateFlag::NewRef);
        assert_eq!(output.ref_updates[0].summary, "[new branch]");
        assert_eq!(output.ref_updates[0].from, "main");
        assert_eq!(output.ref_updates[0].to, "main");
        assert_eq!(output.ref_updates[0].reason, None);
    }

    #[test]
    fn parse_fast_forward_output() {
        let stderr = "To /tmp/bare.git\n   c82f2ae..f5fd671  main -> main\n";
        let output = PushOutput::parse("", stderr);

        assert_eq!(output.ref_updates.len(), 1);
        assert_eq!(output.ref_updates[0].flag, RefUpdateFlag::FastForward);
        assert_eq!(output.ref_updates[0].summary, "c82f2ae..f5fd671");
        assert_eq!(output.ref_updates[0].from, "main");
        assert_eq!(output.ref_updates[0].to, "main");
    }

    #[test]
    fn parse_forced_update_output() {
        let stderr = "To /tmp/bare.git\n + 1234567...89abcde feature -> feature (forced update)\n";
        let output = PushOutput::parse("", stderr);

        assert_eq!(output.ref_updates.len(), 1);
        assert_eq!(output.ref_updates[0].flag, RefUpdateFlag::ForcedUpdate);
        assert_eq!(output.ref_updates[0].summary, "1234567...89abcde");
        assert_eq!(
            output.ref_updates[0].reason,
            Some("forced update".to_string())
        );
    }

    #[test]
    fn parse_rejected_output() {
        let stderr = "To /tmp/bare.git\n ! [rejected]        main -> main (fetch first)\n";
        let output = PushOutput::parse("", stderr);

        assert_eq!(output.ref_updates.len(), 1);
        assert_eq!(output.ref_updates[0].flag, RefUpdateFlag::Rejected);
        assert_eq!(output.ref_updates[0].summary, "[rejected]");
        assert_eq!(
            output.ref_updates[0].reason,
            Some("fetch first".to_string())
        );
    }

    #[test]
    fn parse_deleted_ref() {
        let stderr = "To /tmp/bare.git\n - [deleted]         old-feature\n";
        let output = PushOutput::parse("", stderr);

        assert_eq!(output.ref_updates.len(), 1);
        assert_eq!(output.ref_updates[0].flag, RefUpdateFlag::Deleted);
        assert_eq!(output.ref_updates[0].from, "old-feature");
        assert_eq!(output.ref_updates[0].to, "old-feature");
    }

    #[test]
    fn parse_up_to_date() {
        let stderr = "Everything up-to-date\n";
        let output = PushOutput::parse("", stderr);

        assert!(output.up_to_date);
        assert!(output.ref_updates.is_empty());
    }

    #[test]
    fn parse_gitlab_merge_request_notices_and_urls() {
        let stderr = r#"
remote:
remote: To create a merge request for feature, visit:
remote:   https://gitlab.example.com/org/repo/-/merge_requests/new?merge_request%5Bsource_branch%5D=feature
remote:
To gitlab.example.com:org/repo.git
 * [new branch]      feature -> feature
"#;
        let output = PushOutput::parse("", stderr);

        assert_eq!(
            output.destination,
            Some("gitlab.example.com:org/repo.git".to_string())
        );
        assert_eq!(
            output.urls,
            vec![
                "https://gitlab.example.com/org/repo/-/merge_requests/new?merge_request%5Bsource_branch%5D=feature"
            ]
        );
        assert_eq!(
            output.merge_request_urls,
            vec![
                "https://gitlab.example.com/org/repo/-/merge_requests/new?merge_request%5Bsource_branch%5D=feature"
            ]
        );
        assert_eq!(
            output.notices,
            vec![
                "To create a merge request for feature, visit:",
                "https://gitlab.example.com/org/repo/-/merge_requests/new?merge_request%5Bsource_branch%5D=feature",
            ]
        );
    }

    #[test]
    fn parse_github_pull_request_notices_and_urls() {
        let stderr = r#"
remote: Resolving deltas: 100% (1/1), completed with 1 local object.
remote:
remote: Create a pull request for 'feat' on GitHub by visiting:
remote:      https://github.com/org/repo/pull/new/feat
remote:
To github.com:org/repo.git
 * [new branch]      feat -> feat
"#;
        let output = PushOutput::parse("", stderr);

        assert_eq!(
            output.urls,
            vec!["https://github.com/org/repo/pull/new/feat"]
        );
        assert_eq!(
            output.merge_request_urls,
            vec!["https://github.com/org/repo/pull/new/feat"]
        );
        assert_eq!(
            output.notices,
            vec![
                "Create a pull request for 'feat' on GitHub by visiting:",
                "https://github.com/org/repo/pull/new/feat",
            ]
        );
    }

    fn to_args_vec(cmd: &PushCmd) -> Vec<String> {
        let mut args = Vec::new();
        cmd.to_args(&mut args);
        args.into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn push_default_args() {
        let cmd = PushCmd::default();
        assert_eq!(to_args_vec(&cmd), vec!["push"]);
    }

    #[test]
    fn push_upstream_args() {
        let cmd = PushCmd::upstream("origin", "main");
        assert_eq!(
            to_args_vec(&cmd),
            vec!["push", "--set-upstream", "origin", "main"]
        );
    }

    #[test]
    fn push_delete_args() {
        let cmd = PushCmd::delete("origin", "feature");
        assert_eq!(
            to_args_vec(&cmd),
            vec!["push", "--delete", "origin", "feature"]
        );
    }

    #[test]
    fn push_comprehensive_options_args() {
        let cmd = PushCmd {
            repository: Some("origin".into()),
            refspecs: vec!["main".into(), "feature:feature".into()],
            all: true,
            branches: true,
            mirror: true,
            tags: true,
            follow_tags: Some(true),
            atomic: Some(true),
            dry_run: true,
            porcelain: true,
            force: true,
            force_with_lease: Some(Some("main:1234abcd".into())),
            force_if_includes: true,
            recurse_submodules: Some(RecurseSubmodules::OnDemand),
            thin: Some(true),
            receive_pack: Some("git-receive-pack".into()),
            set_upstream: true,
            progress: Some(false),
            prune: true,
            no_verify: true,
            verify: false,
            signed: Some(Some(SignedMode::Yes)),
            push_options: vec!["mr.create".into(), "ci.skip".into()],
            quiet: false,
            verbose: true,
            repo: Some("custom-repo".into()),
            ipv4: true,
            ipv6: false,
            ..Default::default()
        };

        let args = to_args_vec(&cmd);
        assert_eq!(args[0], "push");
        assert!(args.contains(&"--all".to_string()));
        assert!(args.contains(&"--branches".to_string()));
        assert!(args.contains(&"--mirror".to_string()));
        assert!(args.contains(&"--tags".to_string()));
        assert!(args.contains(&"--follow-tags".to_string()));
        assert!(args.contains(&"--atomic".to_string()));
        assert!(args.contains(&"--dry-run".to_string()));
        assert!(args.contains(&"--porcelain".to_string()));
        assert!(args.contains(&"--force".to_string()));
        assert!(args.contains(&"--force-with-lease=main:1234abcd".to_string()));
        assert!(args.contains(&"--force-if-includes".to_string()));
        assert!(args.contains(&"--recurse-submodules=on-demand".to_string()));
        assert!(args.contains(&"--thin".to_string()));
        assert!(args.contains(&"--receive-pack=git-receive-pack".to_string()));
        assert!(args.contains(&"--set-upstream".to_string()));
        assert!(args.contains(&"--no-progress".to_string()));
        assert!(args.contains(&"--prune".to_string()));
        assert!(args.contains(&"--no-verify".to_string()));
        assert!(args.contains(&"--signed=yes".to_string()));
        assert!(args.contains(&"-o".to_string()));
        assert!(args.contains(&"mr.create".to_string()));
        assert!(args.contains(&"ci.skip".to_string()));
        assert!(args.contains(&"--verbose".to_string()));
        assert!(args.contains(&"--repo=custom-repo".to_string()));
        assert!(args.contains(&"-4".to_string()));
        assert_eq!(args.iter().rev().nth(2), Some(&"origin".to_string()));
        assert_eq!(args.iter().rev().nth(1), Some(&"main".to_string()));
        assert_eq!(
            args.iter().rev().next(),
            Some(&"feature:feature".to_string())
        );
    }
}
