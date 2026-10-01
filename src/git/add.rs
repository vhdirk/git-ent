use std::{ffi::OsString, path::PathBuf};

use crate::git::{ParseOutput, ToArgs};

#[derive(Debug, Clone)]
pub enum AddCmd {
    All,
    Update,
    Paths(Vec<PathBuf>),
}

impl ToArgs for AddCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        match self {
            AddCmd::All => {
                args.push("add".into());
                args.push("-A".into());
            }
            AddCmd::Update => {
                args.push("add".into());
                args.push("-u".into());
            }
            AddCmd::Paths(paths) => {
                args.push("add".into());
                args.push("--".into());
                for path in paths {
                    args.push(path.as_os_str().to_os_string());
                }
            }
        }
    }
}

impl ParseOutput for AddCmd {
    type Output = ();

    fn parse_output(&self, _output: &std::process::Output) -> crate::Result<Self::Output> {
        Ok(())
    }
}
