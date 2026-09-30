use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

/// Query all remote names (`git remote`).
#[derive(Debug, Clone, Default)]
pub struct ListRemotesCommand;

impl GitCommand for ListRemotesCommand {
    type Output = Vec<String>;

    fn to_args(&self) -> Vec<OsString> {
        vec![OsString::from("remote")]
    }

    fn parse_output(&self, output: &Output) -> Result<Vec<String>> {
        let text = String::from_utf8_lossy(&output.stdout);
        let remotes = text
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        Ok(remotes)
    }
}

/// Count commits in a revision range (e.g. `base..head` or `branch`).
#[derive(Debug, Clone)]
pub struct RevListCountCommand {
    pub rev: String,
}

impl RevListCountCommand {
    pub fn new(rev: impl Into<String>) -> Self {
        Self { rev: rev.into() }
    }
}

impl GitCommand for RevListCountCommand {
    type Output = usize;

    fn to_args(&self) -> Vec<OsString> {
        vec![
            OsString::from("rev-list"),
            OsString::from("--count"),
            OsString::from(&self.rev),
        ]
    }

    fn parse_output(&self, output: &Output) -> Result<usize> {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(text.parse().unwrap_or(0))
    }
}

/// Query commit subjects in a range (e.g. `base..head`).
#[derive(Debug, Clone)]
pub struct CommitSubjectsCommand {
    pub range: String,
}

impl CommitSubjectsCommand {
    pub fn new(range: impl Into<String>) -> Self {
        Self {
            range: range.into(),
        }
    }
}

impl GitCommand for CommitSubjectsCommand {
    type Output = Vec<String>;

    fn to_args(&self) -> Vec<OsString> {
        vec![
            OsString::from("log"),
            OsString::from("--pretty=format:* %s"),
            OsString::from("--reverse"),
            OsString::from(&self.range),
        ]
    }

    fn parse_output(&self, output: &Output) -> Result<Vec<String>> {
        let text = String::from_utf8_lossy(&output.stdout);
        let subjects = text
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        Ok(subjects)
    }
}
