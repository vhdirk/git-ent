use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

use crate::git::{ParseOutput, ToArgs};

// TODO: map full command-line interface for `git checkout`

pub enum CheckoutCmd {
    Switch {
        target: Option<String>,
        create: Option<String>,
    },
    Restore {
        target: Option<String>,
        paths: Vec<PathBuf>,
    },
}

impl ToArgs for CheckoutCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("checkout".into());

        match self {
            CheckoutCmd::Switch { target, create } => {
                if let Some(create) = create {
                    args.push("-b".into());
                    args.push(create.into());
                }
                if let Some(target) = target {
                    args.push(target.into());
                }
            }
            CheckoutCmd::Restore { target, paths } => {
                if let Some(target) = target {
                    args.push(target.into());
                }
                if !paths.is_empty() {
                    args.push("--".into());
                    for path in paths {
                        args.push(path.as_os_str().to_os_string());
                    }
                }
            }
        }
    }
}

impl ParseOutput for CheckoutCmd {
    type Output = ();

    fn parse_output(&self, _output: &Output) -> crate::Result<Self::Output> {
        Ok(())
    }
}
