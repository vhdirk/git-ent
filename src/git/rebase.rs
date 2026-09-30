use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

#[derive(Debug, Clone)]
pub struct RebaseCommand {
    pub branch: String,
}

impl RebaseCommand {
    pub fn new(branch: impl Into<String>) -> Self {
        Self {
            branch: branch.into(),
        }
    }
}

impl GitCommand for RebaseCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        vec![OsString::from("rebase"), OsString::from(&self.branch)]
    }

    fn parse_output(&self, _output: &Output) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct RebaseAbortCommand;

impl GitCommand for RebaseAbortCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        vec![OsString::from("rebase"), OsString::from("--abort")]
    }

    fn parse_output(&self, _output: &Output) -> Result<()> {
        Ok(())
    }
}
