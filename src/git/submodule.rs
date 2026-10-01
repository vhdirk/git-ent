use std::{ffi::OsString, path::PathBuf, process::Output};

use crate::error::Result;
use crate::git::{ParseOutput, ToArgs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmoduleState {
    /// Uninitialized (`-`): The submodule has not been initialized yet.
    Uninitialized,
    /// UpToDate (` `): The submodule is checked out at the correct commit.
    UpToDate,
    /// Modified (`+`): The submodule's checked out commit does not match the superproject's recorded commit.
    Modified,
    /// Conflict (`U`): The submodule has merge conflicts.
    Conflict,
    /// Unknown fallback for unexpected characters
    Unknown(char),
}

impl From<char> for SubmoduleState {
    fn from(c: char) -> Self {
        match c {
            '-' => Self::Uninitialized,
            ' ' => Self::UpToDate,
            '+' => Self::Modified,
            'u' | 'U' => Self::Conflict,
            other => Self::Unknown(other),
        }
    }
}

/// Parsed status of an individual submodule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmoduleStatusInfo {
    pub state: SubmoduleState, // ' ', '-', '+', 'U'
    pub commit_hash: String,
    pub path: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmoduleOutput {
    /// Status list returned by `git submodule status`
    Status(Vec<SubmoduleStatusInfo>),
    /// General text output (e.g. `foreach` script execution or raw logs)
    Text(String),
    /// Confirmation that an operation succeeded
    Success,
}

// --- Subcommand Configuration Structs ---

#[derive(Default, Debug, Clone)]
pub struct Add {
    pub repository: String,
    pub path: Option<PathBuf>,
    pub branch: Option<String>,
    pub force: bool,
    pub shallow_depth: Option<usize>,
}

impl Add {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("add".into());
        if self.force {
            args.push("--force".into());
        }
        if let Some(b) = &self.branch {
            args.push("--branch".into());
            args.push(b.into());
        }
        if let Some(d) = self.shallow_depth {
            args.push("--depth".into());
            args.push(d.to_string().into());
        }
        args.push(self.repository.as_str().into());
        if let Some(p) = &self.path {
            args.push(p.into());
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Status {
    pub cached: bool,
    pub recursive: bool,
    pub paths: Vec<String>,
}

impl Status {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("status".into());
        if self.cached {
            args.push("--cached".into());
        }
        if self.recursive {
            args.push("--recursive".into());
        }
        if !self.paths.is_empty() {
            args.push("--".into());
            args.extend(self.paths.iter().map(OsString::from));
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Update {
    pub init: bool,
    pub remote: bool,
    pub recursive: bool,
    pub no_fetch: bool,
    pub rebase: bool,
    pub merge: bool,
    pub depth: Option<usize>,
    pub paths: Vec<String>,
}

impl Update {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("update".into());
        if self.init {
            args.push("--init".into());
        }
        if self.remote {
            args.push("--remote".into());
        }
        if self.recursive {
            args.push("--recursive".into());
        }
        if self.no_fetch {
            args.push("--no-fetch".into());
        }
        if self.rebase {
            args.push("--rebase".into());
        }
        if self.merge {
            args.push("--merge".into());
        }
        if let Some(d) = self.depth {
            args.push("--depth".into());
            args.push(d.to_string().into());
        }
        if !self.paths.is_empty() {
            args.push("--".into());
            args.extend(self.paths.iter().map(OsString::from));
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Foreach {
    pub recursive: bool,
    pub command: String,
}

impl Foreach {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("foreach".into());
        if self.recursive {
            args.push("--recursive".into());
        }
        args.push(self.command.as_str().into());
    }
}

#[derive(Default, Debug, Clone)]
pub struct SyncCmd {
    pub recursive: bool,
    pub paths: Vec<String>,
}

impl SyncCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("sync".into());
        if self.recursive {
            args.push("--recursive".into());
        }
        if !self.paths.is_empty() {
            args.push("--".into());
            args.extend(self.paths.iter().map(OsString::from));
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct AbsorbGitDirs {
    pub paths: Vec<String>,
}

impl ToArgs for AbsorbGitDirs {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("absorbgitdirs".into());
        if !self.paths.is_empty() {
            args.push("--".into());
            args.extend(self.paths.iter().map(OsString::from));
        }
    }
}

// --- Main Submodule Enum ---

#[derive(Debug, Clone)]
pub enum SubmoduleCmd {
    Add(Add),
    Status(Status),
    Update(Update),
    Foreach(Foreach),
    Sync(SyncCmd),
    AbsorbGitDirs(AbsorbGitDirs),
}

impl ToArgs for SubmoduleCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("submodule".into());
        match self {
            Self::Add(c) => c.to_args(args),
            Self::Status(c) => c.to_args(args),
            Self::Update(c) => c.to_args(args),
            Self::Foreach(c) => c.to_args(args),
            Self::Sync(c) => c.to_args(args),
            Self::AbsorbGitDirs(c) => c.to_args(args),
        }
    }
}

impl ParseOutput for SubmoduleCmd {
    type Output = SubmoduleOutput;

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        let stdout = String::from_utf8_lossy(&output.stdout);

        match self {
            Self::Status(_) => {
                let mut statuses = Vec::new();
                for line in stdout.lines() {
                    if line.is_empty() {
                        continue;
                    }

                    // Format: `[-] <commit> <path> (description)`
                    let mut chars = line.chars();
                    let state_char = chars.next().unwrap_or(' ');
                    let rest = &line[1..].trim();

                    let mut parts = rest.split_whitespace();
                    let commit_hash = parts.next().unwrap_or_default().to_string();
                    let path = parts.next().unwrap_or_default().to_string();

                    // Extract optional description in parentheses if present
                    let description = if let Some(open_idx) = line.find('(') {
                        if let Some(close_idx) = line.find(')') {
                            if close_idx > open_idx {
                                Some(line[open_idx + 1..close_idx].to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    statuses.push(SubmoduleStatusInfo {
                        state: SubmoduleState::from(state_char),
                        commit_hash,
                        path,
                        description,
                    });
                }
                Ok(SubmoduleOutput::Status(statuses))
            }
            Self::Foreach(_) => Ok(SubmoduleOutput::Text(stdout.to_string())),
            _ => Ok(SubmoduleOutput::Success),
        }
    }
}
