use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::{ParseOutput, ToArgs};

/// Git rev-list command.
#[derive(Debug, Clone)]
pub struct RevListCmd {
    pub rev: String,
    pub count: bool,
}

impl RevListCmd {
    pub fn count(rev: impl Into<String>) -> Self {
        Self {
            rev: rev.into(),
            count: true,
        }
    }
}

impl ToArgs for RevListCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("rev-list".into());
        if self.count {
            args.push("--count".into());
        }
        args.push(self.rev.as_str().into());
    }
}

impl ParseOutput for RevListCmd {
    type Output = usize;

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(text.parse().unwrap_or(0))
    }
}
