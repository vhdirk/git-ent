use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

/// Revision parsing and repository path queries with `git rev-parse`.
#[derive(Debug, Clone)]
pub struct RevParseCommand {
    pub args: Vec<OsString>,
}

impl RevParseCommand {
    /// Query the top-level directory of the working tree.
    pub fn toplevel() -> Self {
        Self {
            args: vec![
                OsString::from("rev-parse"),
                OsString::from("--show-toplevel"),
            ],
        }
    }

    /// Check if a path is a valid git repository or gitfile pointer.
    pub fn resolve_git_dir(path: impl Into<PathBuf>) -> Self {
        Self {
            args: vec![
                OsString::from("rev-parse"),
                OsString::from("--resolve-git-dir"),
                path.into().into_os_string(),
            ],
        }
    }
}

impl GitCommand for RevParseCommand {
    type Output = PathBuf;

    fn to_args(&self) -> Vec<OsString> {
        self.args.clone()
    }

    fn parse_output(&self, output: &Output) -> Result<PathBuf> {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(text))
    }
}
