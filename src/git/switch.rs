use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

/// Options for `git switch`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwitchCommand {
    pub target: Option<String>,
    pub create: Option<String>,
    pub force_create: Option<String>,
    pub detach: bool,
    pub start_point: Option<String>,
}

impl SwitchCommand {
    /// Switch to an existing branch (`git switch <branch>`).
    pub fn branch(target: impl Into<String>) -> Self {
        Self {
            target: Some(target.into()),
            create: None,
            force_create: None,
            detach: false,
            start_point: None,
        }
    }

    /// Create and switch to a new branch (`git switch -c <name> [<start-point>]`).
    pub fn create(name: impl Into<String>, start_point: Option<impl Into<String>>) -> Self {
        Self {
            target: None,
            create: Some(name.into()),
            force_create: None,
            detach: false,
            start_point: start_point.map(Into::into),
        }
    }

    /// Force create/reset and switch to a branch (`git switch -C <name> [<start-point>]`).
    pub fn force_create(name: impl Into<String>, start_point: Option<impl Into<String>>) -> Self {
        Self {
            target: None,
            create: None,
            force_create: Some(name.into()),
            detach: false,
            start_point: start_point.map(Into::into),
        }
    }

    /// Detach HEAD at target (`git switch --detach <target>`).
    pub fn detach(target: impl Into<String>) -> Self {
        Self {
            target: Some(target.into()),
            create: None,
            force_create: None,
            detach: true,
            start_point: None,
        }
    }
}

impl GitCommand for SwitchCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        let mut args = vec![OsString::from("switch")];

        if self.detach {
            args.push(OsString::from("--detach"));
            if let Some(target) = &self.target {
                args.push(OsString::from(target));
            }
            return args;
        }

        if let Some(create) = &self.create {
            args.push(OsString::from("-c"));
            args.push(OsString::from(create));
            if let Some(sp) = &self.start_point {
                args.push(OsString::from(sp));
            }
            return args;
        }

        if let Some(force_create) = &self.force_create {
            args.push(OsString::from("-C"));
            args.push(OsString::from(force_create));
            if let Some(sp) = &self.start_point {
                args.push(OsString::from(sp));
            }
            return args;
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
