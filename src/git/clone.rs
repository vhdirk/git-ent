use crate::error::Result;
use crate::git::ParseOutput;
use crate::git::ToArgs;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::process::Output;

// TODO: map full command-line interface for `git clone`

fn to_canonical_absolute_path(path: &Path) -> Result<PathBuf> {
    let canonical = path.canonicalize()?;
    if canonical.is_absolute() {
        Ok(canonical)
    } else {
        let mut current = std::env::current_dir()?;
        current.push(&canonical);
        Ok(current)
    }
}

pub struct CloneCmd {
    pub url: String,
    pub destination: PathBuf,
    pub recurse_submodules: bool,
}

impl ToArgs for CloneCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("-c".into());
        args.push("protocol.file.allow=always".into());
        args.push("clone".into());
        if self.recurse_submodules {
            args.push("--recurse-submodules".into());
        }
        args.push(self.url.clone().into());
        args.push(self.destination.as_os_str().into());
    }
}

impl ParseOutput for CloneCmd {
    type Output = ();

    fn parse_output(&self, _output: &Output) -> Result<Self::Output> {
        Ok(())
    }
}
