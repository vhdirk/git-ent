use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

/// Query symbolic-ref with `git symbolic-ref`.
#[derive(Debug, Clone, Default)]
pub struct SymbolicRefCommand {
    pub ref_name: String,
    pub quiet: bool,
    pub short: bool,
}

impl SymbolicRefCommand {
    /// Query symbolic-ref for HEAD.
    pub fn head() -> Self {
        Self {
            ref_name: "HEAD".into(),
            quiet: true,
            short: true,
        }
    }
}

impl GitCommand for SymbolicRefCommand {
    type Output = Option<String>;

    fn to_args(&self) -> Vec<OsString> {
        let mut args = vec![OsString::from("symbolic-ref")];
        if self.quiet {
            args.push(OsString::from("--quiet"));
        }
        if self.short {
            args.push(OsString::from("--short"));
        }
        args.push(OsString::from(&self.ref_name));
        args
    }

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if name.is_empty() {
                Ok(None)
            } else {
                Ok(Some(name))
            }
        } else {
            Ok(None)
        }
    }
}
