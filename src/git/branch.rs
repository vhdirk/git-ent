use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

/// Create a new branch with `git branch`.
#[derive(Debug, Clone)]
pub struct BranchCommand {
    pub name: String,
}

impl BranchCommand {
    pub fn create(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl GitCommand for BranchCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        vec![
            OsString::from("branch"),
            OsString::from("--"),
            OsString::from(&self.name),
        ]
    }

    fn parse_output(&self, _output: &Output) -> Result<()> {
        Ok(())
    }
}

/// Query local branch names with `git branch --list`.
#[derive(Debug, Clone, Default)]
pub struct ListBranchesCommand;

impl GitCommand for ListBranchesCommand {
    type Output = Vec<String>;

    fn to_args(&self) -> Vec<OsString> {
        vec![
            OsString::from("branch"),
            OsString::from("--list"),
            OsString::from("--format=%(refname:short)"),
        ]
    }

    fn parse_output(&self, output: &Output) -> Result<Vec<String>> {
        let text = String::from_utf8_lossy(&output.stdout);
        let branches = text
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        Ok(branches)
    }
}

/// Check if a local branch exists with `git show-ref --verify --quiet refs/heads/<name>`.
#[derive(Debug, Clone)]
pub struct BranchExistsCommand {
    pub name: String,
}

impl BranchExistsCommand {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl GitCommand for BranchExistsCommand {
    type Output = bool;

    fn to_args(&self) -> Vec<OsString> {
        vec![
            OsString::from("show-ref"),
            OsString::from("--verify"),
            OsString::from("--quiet"),
            OsString::from(format!("refs/heads/{}", self.name)),
        ]
    }

    fn parse_output(&self, output: &Output) -> Result<bool> {
        Ok(output.status.success())
    }
}
