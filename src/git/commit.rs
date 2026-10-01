use crate::git::{ParseOutput, ToArgs};
use crate::{Result, SgitError};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

/// Structured representation of a successful commit output
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDetails {
    pub branch: String,
    pub hash: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitOutput {
    /// Successfully created a commit with parsed details
    Committed(CommitDetails),
    /// Nothing to commit (clean working tree, exit code 1 or specific message)
    NothingToCommit,
    /// Raw output fallback if format doesn't match standard patterns
    Raw(String),
}

#[derive(Default, Debug, Clone)]
pub struct CommitCmd {
    pub message: Option<String>,     // -m, --message=<msg>
    pub message_files: Vec<PathBuf>, // -F, --file=<file>
    pub all: bool,                   // -a, --all
    pub amend: bool,                 // --amend
    pub allow_empty: bool,           // --allow-empty
    pub no_verify: bool,             // -n, --no-verify (bypass hooks)
    pub signoff: bool,               // -s, --signoff
    pub author: Option<String>,      // --author=<author>
    pub only: Vec<String>,           // <pathspec>...
}

impl ToArgs for CommitCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("commit".into());

        if self.all {
            args.push("--all".into());
        }
        if self.amend {
            args.push("--amend".into());
        }
        if self.allow_empty {
            args.push("--allow-empty".into());
        }
        if self.no_verify {
            args.push("--no-verify".into());
        }
        if self.signoff {
            args.push("--signoff".into());
        }

        if let Some(msg) = &self.message {
            args.push("-m".into());
            args.push(msg.into());
        }

        for file in &self.message_files {
            args.push("-F".into());
            args.push(file.into());
        }

        if let Some(author) = &self.author {
            args.push(format!("--author={author}").into());
        }

        if !self.only.is_empty() {
            args.push("--".into());
            args.extend(self.only.iter().map(OsString::from));
        }
    }
}

impl ParseOutput for CommitCmd {
    type Output = Option<CommitDetails>;

    fn allow_failure(&self) -> bool {
        true
    }

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}\n{stderr}");

        // Git exits with non-zero or outputs "nothing to commit" when there are no changes
        if !output.status.success() || combined.contains("nothing to commit") {
            if combined.contains("nothing to commit") {
                return Ok(None);
            }
            return Err(SgitError::GitExit {
                workdir: PathBuf::from("."),
                code: output.status.code(),
                stderr: stderr.to_string(),
            });
        }

        // Git commit success format typically looks like:
        // [main abc1234] Commit message summary
        // or [detached HEAD def5678] Commit message summary
        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with('[') && line.contains(']') {
                if let Some(end_bracket) = line.find(']') {
                    let header = &line[1..end_bracket]; // e.g., "main abc1234" or "detached HEAD def5678"
                    let summary = line[end_bracket + 1..].trim().to_string();

                    let parts = header.split_whitespace();
                    let mut branch_tokens = Vec::new();
                    let mut hash = String::new();

                    // Extract tokens; the last token is usually the short hash
                    let tokens: Vec<&str> = parts.collect();
                    if tokens.len() >= 2 {
                        hash = tokens.last().unwrap().to_string();
                        branch_tokens = tokens[..tokens.len() - 1].to_vec();
                    }

                    return Ok(Some(CommitDetails {
                        branch: branch_tokens.join(" "),
                        hash,
                        summary,
                    }));
                }
            }
        }

        Err(SgitError::ParseError(format!(
            "unexpected commit output format: {}",
            stdout
        )))
    }
}
