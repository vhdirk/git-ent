use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

/// Options for `git checkout`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckoutCommand {
    pub target: Option<String>,
    pub create_branch: Option<String>,
    pub paths: Vec<PathBuf>,
}

impl CheckoutCommand {
    /// Checkout an existing branch or commit-ish (`git checkout <target>`).
    pub fn branch(target: impl Into<String>) -> Self {
        Self {
            target: Some(target.into()),
            create_branch: None,
            paths: Vec::new(),
        }
    }

    /// Create and switch to a new branch (`git checkout -b <name>`).
    pub fn create_branch(name: impl Into<String>) -> Self {
        Self {
            target: None,
            create_branch: Some(name.into()),
            paths: Vec::new(),
        }
    }

    /// Checkout paths from a branch or index (`git checkout [<tree-ish>] -- <paths...>`).
    pub fn paths(treeish: Option<impl Into<String>>, paths: Vec<PathBuf>) -> Self {
        Self {
            target: treeish.map(Into::into),
            create_branch: None,
            paths,
        }
    }
}

impl GitCommand for CheckoutCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        let mut args = vec![OsString::from("checkout")];

        if let Some(create) = &self.create_branch {
            args.push(OsString::from("-b"));
            args.push(OsString::from(create));
            if let Some(target) = &self.target {
                args.push(OsString::from(target));
            }
            return args;
        }

        if !self.paths.is_empty() {
            if let Some(target) = &self.target {
                args.push(OsString::from(target));
            }
            args.push(OsString::from("--"));
            for path in &self.paths {
                args.push(path.as_os_str().to_os_string());
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
