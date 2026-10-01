use crate::error::Result;
use crate::git::{ParseOutput, ToArgs};
use std::ffi::OsString;
use std::process::Output;

/// Typed output variants for `git symbolic-ref`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolicRefOutput {
    /// The target reference name returned by a read operation (e.g., "refs/heads/main")
    Ref(Option<String>),

    /// Confirmation that an update or delete operation succeeded
    Success,
}

#[derive(Debug, Clone, Default)]
pub struct Read {
    pub name: String,
    pub quiet: bool,
    pub short: bool,
    pub no_recurse: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Update {
    pub name: String,
    pub target: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Delete {
    pub name: String,
    pub quiet: bool,
}

#[derive(Debug, Clone)]
pub enum SymbolicRefCmd {
    /// Read a symbolic reference (`git symbolic-ref [-q] [--short] [--no-recurse] <name>`)
    Read(Read),
    /// Create or update a symbolic reference (`git symbolic-ref [-m <reason>] <name> <ref>`)
    Update(Update),
    /// Delete a symbolic reference (`git symbolic-ref --delete [-q] <name>`)
    Delete(Delete),
}

impl ToArgs for SymbolicRefCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("symbolic-ref".into());

        match self {
            Self::Read(Read {
                name,
                quiet,
                short,
                no_recurse,
            }) => {
                if *quiet {
                    args.push("--quiet".into());
                }
                if *short {
                    args.push("--short".into());
                }
                if *no_recurse {
                    args.push("--no-recurse".into());
                }
                args.push(name.into());
            }
            Self::Update(Update {
                name,
                target,
                reason,
            }) => {
                if let Some(r) = reason {
                    args.push("-m".into());
                    args.push(r.into());
                }
                args.push(name.into());
                args.push(target.into());
            }
            Self::Delete(Delete { name, quiet }) => {
                args.push("--delete".into());
                if *quiet {
                    args.push("--quiet".into());
                }
                args.push(name.into());
            }
        }
    }
}

impl ParseOutput for SymbolicRefCmd {
    type Output = SymbolicRefOutput;

    fn allow_failure(&self) -> bool {
        matches!(self, Self::Read(_))
    }

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        match self {
            Self::Read(_) => {
                // not sure how to deal with 'quit' here
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    Ok(SymbolicRefOutput::Ref(Some(text)))
                } else {
                    Ok(SymbolicRefOutput::Ref(None))
                }
            }
            Self::Update(_) | Self::Delete(_) => {
                // Update and delete commands do not output text on success
                Ok(SymbolicRefOutput::Success)
            }
        }
    }
}
