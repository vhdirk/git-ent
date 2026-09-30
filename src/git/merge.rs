use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

#[derive(Debug, Clone)]
pub struct MergeCommand {
    pub branch: String,
}

impl MergeCommand {
    pub fn new(branch: impl Into<String>) -> Self {
        Self {
            branch: branch.into(),
        }
    }
}

impl GitCommand for MergeCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        vec![OsString::from("merge"), OsString::from(&self.branch)]
    }

    fn parse_output(&self, _output: &Output) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MergeAbortCommand;

impl GitCommand for MergeAbortCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        vec![OsString::from("merge"), OsString::from("--abort")]
    }

    fn parse_output(&self, _output: &Output) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MergeBaseCommand {
    pub rev1: String,
    pub rev2: String,
}

impl MergeBaseCommand {
    pub fn new(rev1: impl Into<String>, rev2: impl Into<String>) -> Self {
        Self {
            rev1: rev1.into(),
            rev2: rev2.into(),
        }
    }
}

impl GitCommand for MergeBaseCommand {
    type Output = String;

    fn to_args(&self) -> Vec<OsString> {
        vec![
            OsString::from("merge-base"),
            OsString::from(&self.rev1),
            OsString::from(&self.rev2),
        ]
    }

    fn parse_output(&self, output: &Output) -> Result<String> {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
