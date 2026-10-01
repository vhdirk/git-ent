use crate::error::Result;
use std::{ffi::OsString, path::Path, process::Output};

pub trait ToArgs {
    /// Serialize into argument-safe CLI arguments.
    fn to_args(&self, args: &mut Vec<OsString>);
}

pub trait ParseOutput {
    type Output;

    /// Parse the process output into the typed response.
    fn parse_output(&self, output: &Output) -> Result<Self::Output>;

    /// Whether the command is allowed to fail without causing an error.
    /// If set to false, `parse_output` will not be called if the command fails.
    fn allow_failure(&self) -> bool {
        false
    }
}

pub trait GitCmd: ToArgs + ParseOutput {
    /// Run the command in `cwd`,
    fn run(&self, cwd: impl AsRef<Path>) -> Result<Self::Output> {
        let mut args = Vec::new();
        self.to_args(&mut args);

        let output = crate::git::run(cwd.as_ref(), &args, self.allow_failure())?;
        self.parse_output(&output)
    }
}

impl<T: ToArgs + ParseOutput> GitCmd for T {}
