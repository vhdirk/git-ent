use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

/// Reset mode matching `git reset` options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResetMode {
    #[default]
    Mixed,
    Hard,
    Soft,
}

/// Reset repository state with `git reset`.
#[derive(Debug, Clone, Default)]
pub struct ResetCommand<'a> {
    pub mode: ResetMode,
    pub target: Option<&'a str>,
}

impl<'a> GitCommand for ResetCommand<'a> {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        let mut args = vec![OsString::from("reset")];
        match self.mode {
            ResetMode::Mixed => args.push(OsString::from("--mixed")),
            ResetMode::Hard => args.push(OsString::from("--hard")),
            ResetMode::Soft => args.push(OsString::from("--soft")),
        }
        if let Some(target) = &self.target {
            args.push(OsString::from(target));
        }
        args
    }

    fn parse_output(&self, _output: &Output) -> Result<()> {
        Ok(())
    }
}
