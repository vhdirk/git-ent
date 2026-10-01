use crate::error::{Result, SgitError};
use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Output},
};

pub mod add;
pub mod branch;
pub mod checkout;
pub mod clone;
pub mod command;
pub mod commit;
pub mod config;
pub mod rev_parse;
pub mod show_ref;
pub mod status;
pub mod submodule;
pub mod symbolic_ref;
pub use command::{GitCmd, ParseOutput, ToArgs};

const GIT: &str = "git";

/// Internal helper to execute a Git binary with arguments in a given working directory.
pub(crate) fn run<I, S>(cwd: &Path, args: I, allow_failure: bool) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(GIT)
        .args(args)
        .current_dir(cwd)
        .output()
        // return error when the Git command itself could not be launched
        .map_err(|source| SgitError::GitLaunch {
            workdir: cwd.to_path_buf(),
            source,
        })?;

    if output.status.success() || allow_failure {
        return Ok(output);
    }

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
