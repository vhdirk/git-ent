use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Output};

use crate::error::{Result, SgitError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Git {
    pub executable: OsString,
}

impl Default for Git {
    fn default() -> Self {
        Self {
            executable: OsString::from("git"),
        }
    }
}

impl Git {
    /// Create a new `Git` instance with the specified executable.
    pub fn new(executable: OsString) -> Self {
        Self { executable }
    }

    /// Internal helper to execute a Git binary with arguments in a given working directory.
    fn execute(&self, cwd: &Path, args: &[OsString]) -> Result<Output> {
        Command::new(&self.executable)
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(|source| SgitError::GitLaunch {
                workdir: cwd.to_path_buf(),
                source,
            })
    }

    /// Run Git using the specified `executable` in `cwd` with `args`.
    ///
    /// Succeeds only if Git exits with status code zero.
    pub fn run_with(&self, cwd: &Path, args: &[OsString]) -> Result<Output> {
        let output = self.execute(cwd, args)?;
        if !output.status.success() {
            let stderr_raw = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stderr = if stderr_raw.is_empty() {
                match output.status.code() {
                    Some(code) => format!("command exited with code {code}"),
                    None => "command terminated by signal".to_string(),
                }
            } else {
                stderr_raw
            };
            return Err(SgitError::GitExit {
                workdir: cwd.to_path_buf(),
                code: output.status.code(),
                stderr,
            });
        }
        Ok(output)
    }

    /// Run Git using the specified `executable` in `cwd` with `args`, allowing non-zero exit status.
    ///
    /// Callers must inspect `output.status` to determine success.
    pub fn run_with_allow_failure(&self, cwd: &Path, args: &[OsString]) -> Result<Output> {
        self.execute(cwd, args)
    }
}

/// Run Git in `cwd` with `args`, failing if Git exits with a non-zero status.
pub fn run_git(cwd: &Path, args: &[OsString]) -> Result<Output> {
    Git::default().run_with(cwd, args)
}

/// Run Git using the specified `executable` in `cwd` with `args`.
pub fn run_git_with(executable: &OsStr, cwd: &Path, args: &[OsString]) -> Result<Output> {
    Git::new(executable.to_os_string()).run_with(cwd, args)
}

/// Run Git in `cwd` with `args`, allowing non-zero exit status.
pub fn run_git_allow_failure(cwd: &Path, args: &[OsString]) -> Result<Output> {
    Git::default().run_with_allow_failure(cwd, args)
}

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
        let output = git.run_with(cwd.as_ref(), &self.to_args())?;
        self.parse_output(&output)
    }

    /// Run the command using the specified Git `executable` in `cwd`,
    /// allowing non-zero exit status.
    fn try_run(&self, git: &Git, cwd: impl AsRef<Path>) -> Result<Self::Output> {
        let output = git.run_with_allow_failure(cwd.as_ref(), &self.to_args())?;
        self.parse_output(&output)
    }
}
