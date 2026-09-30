use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

/// Options for `git commit`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitCommand {
    pub message: String,
    pub no_verify: bool,
}

impl CommitCommand {
    /// Create a new commit command with a commit message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            no_verify: false,
        }
    }

    /// Set whether hooks should be bypassed (`--no-verify`).
    pub fn no_verify(mut self, no_verify: bool) -> Self {
        self.no_verify = no_verify;
        self
    }
}

impl GitCommand for CommitCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        let mut args = vec![OsString::from("commit")];
        if self.no_verify {
            args.push(OsString::from("--no-verify"));
        }
        args.push(OsString::from("-m"));
        args.push(OsString::from(&self.message));
        args
    }

    fn parse_output(&self, _output: &Output) -> Result<()> {
        Ok(())
    }
}
