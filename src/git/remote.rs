use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::{ParseOutput, ToArgs};

/// Git remote command.
#[derive(Debug, Clone, Default)]
pub struct RemoteCmd;

impl ToArgs for RemoteCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("remote".into());
    }
}

impl ParseOutput for RemoteCmd {
    type Output = Vec<String>;

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        let text = String::from_utf8_lossy(&output.stdout);
        let remotes = text
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        Ok(remotes)
    }
}
