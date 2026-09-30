use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

/// Options for restoring working tree files or unstaging index entries with `git restore`.
#[derive(Debug, Clone, Default)]
pub struct RestoreCommand {
    pub staged: bool,
    pub paths: Vec<PathBuf>,
}

impl GitCommand for RestoreCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        let mut args = vec![OsString::from("restore")];
        if self.staged {
            args.push(OsString::from("--staged"));
        }
        if !self.paths.is_empty() {
            args.push(OsString::from("--"));
            for path in &self.paths {
                args.push(path.as_os_str().to_os_string());
            }
        }
        args
    }

    fn parse_output(&self, _output: &Output) -> Result<()> {
        Ok(())
    }
}
