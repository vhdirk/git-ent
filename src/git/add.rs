use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

#[derive(Debug, Clone)]
pub enum AddCommand {
    All,
    Update,
    Paths(Vec<PathBuf>),
}

impl GitCommand for AddCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        match self {
            AddCommand::All => vec![OsString::from("add"), OsString::from("-A")],
            AddCommand::Update => vec![OsString::from("add"), OsString::from("-u")],
            AddCommand::Paths(paths) => {
                let mut args = vec![OsString::from("add"), OsString::from("--")];
                for path in paths {
                    args.push(path.as_os_str().to_os_string());
                }
                args
            }
        }
    }

    fn parse_output(&self, _output: &Output) -> Result<()> {
        Ok(())
    }
}
