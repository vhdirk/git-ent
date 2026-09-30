use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

#[derive(Debug, Clone, Default)]
pub struct PushCommand {
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub set_upstream: bool,
    pub push_options: Vec<String>,
}

impl PushCommand {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upstream(remote: impl Into<String>, branch: impl Into<String>) -> Self {
        Self {
            remote: Some(remote.into()),
            branch: Some(branch.into()),
            set_upstream: true,
            push_options: Vec::new(),
        }
    }
}

impl GitCommand for PushCommand {
    type Output = (String, String);

    fn to_args(&self) -> Vec<OsString> {
        let mut args = vec![OsString::from("push")];
        for opt in &self.push_options {
            args.push(OsString::from("-o"));
            args.push(OsString::from(opt));
        }
        if self.set_upstream {
            args.push(OsString::from("--set-upstream"));
        }
        if let Some(remote) = &self.remote {
            args.push(OsString::from(remote));
        }
        if let Some(branch) = &self.branch {
            args.push(OsString::from(branch));
        }
        args
    }

    fn parse_output(&self, output: &Output) -> Result<(String, String)> {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((stdout, stderr))
    }
}
