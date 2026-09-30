use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

#[derive(Debug, Clone)]
pub struct CloneCommand {
    pub url: String,
    pub dest: Option<PathBuf>,
    pub recurse_submodules: bool,
}

impl CloneCommand {
    pub fn recursive(url: impl Into<String>, dest: Option<PathBuf>) -> Self {
        Self {
            url: url.into(),
            dest,
            recurse_submodules: true,
        }
    }
}

impl GitCommand for CloneCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        let mut args = vec![
            OsString::from("-c"),
            OsString::from("protocol.file.allow=always"),
            OsString::from("clone"),
        ];
        if self.recurse_submodules {
            args.push(OsString::from("--recurse-submodules"));
        }
        args.push(OsString::from(&self.url));
        if let Some(dest) = &self.dest {
            args.push(dest.as_os_str().to_os_string());
        }
        args
    }

    fn parse_output(&self, _output: &Output) -> Result<()> {
        Ok(())
    }
}
