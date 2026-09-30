use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Output};

use crate::error::{Result, SgitError};
use crate::git::Git;


/// A command that can be serialized into argument-safe Git CLI arguments.
pub trait GitCommand {
    type Output;

    /// Serialize into argument-safe CLI arguments.
    fn to_args(&self) -> Vec<OsString>;

    /// Parse the process output into the typed response.
    fn parse_output(&self, output: &Output) -> Result<Self::Output>;

    /// Run the command using the specified Git `executable` in `cwd`,
    /// failing if Git exits with a non-zero status.
    fn run(&self, git: &Git, cwd: impl AsRef<Path>) -> Result<Self::Output> {
        let output = git.run(cwd.as_ref(), &self.to_args())?;
        self.parse_output(&output)
    }

    /// Run the command using the specified Git `executable` in `cwd`,
    /// allowing non-zero exit status.
    fn try_run(&self, git: &Git, cwd: impl AsRef<Path>) -> Result<Self::Output> {
        let output = git.run(cwd.as_ref(), &self.to_args())?;
        self.parse_output(&output)
    }
}
