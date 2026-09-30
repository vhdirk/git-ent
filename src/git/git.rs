use std::{ffi::{OsStr, OsString}, path::Path, process::{Command, Output}};
use crate::{config::SgitConfig, error::{Result, SgitError}};
use std::path::PathBuf;
use crate::{config::SgitConfig, error::{Result, SgitError}};


#[derive(Debug, Clone)]
pub enum AddMode {
    All,
    Update,
    Paths(Vec<PathBuf>),
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Git {
    pub executable: OsString,
}

impl Default for Git {
    fn default() -> Self {
        Self {
            executable: SgitConfig::load_global().git.into(),
        }
    }
}

impl Git {
    /// Create a new `Git` instance with the specified executable.
    pub fn new(executable: OsString) -> Self {
        Self { executable }
    }

    /// Internal helper to execute a Git binary with arguments in a given working directory.
    pub fn run(&self, cwd: &Path, args: &[OsString]) -> Result<Output> {
        let output = Command::new(&self.executable)
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(|source| SgitError::GitLaunch {
                workdir: cwd.to_path_buf(),
                source,
            })?;

        if output.status.success() {
            return Ok(output);
        }

        let stderr_raw = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stderr = if stderr_raw.is_empty() {
            match output.status.code() {
                Some(code) => format!("command exited with code {code}"),
                None => "command terminated by signal".to_string(),
            }
        } else {
            stderr_raw
        };
        return Err(SgitError::GitExit {
            workdir: cwd.to_path_buf(),
            code: output.status.code(),
            stderr,
        });
    }

    pub fn add(&self, cwd: &Path, add_mode: &AddMode) -> Result<Output> {
        let args = match add_mode {
            AddMode::All => vec![OsString::from("add"), OsString::from("-A")],
            AddMode::Update => vec![OsString::from("add"), OsString::from("-u")],
            AddMode::Paths(paths) => {
                let mut args = vec![OsString::from("add"), OsString::from("--")];
                for path in paths {
                    args.push(path.as_os_str().to_os_string());
                }
                args
            }
        };

        self.run(cwd, args.as_slice())
    }


}

/// Run Git in `cwd` with `args`, failing if Git exits with a non-zero status.
pub fn run_git(cwd: &Path, args: &[OsString]) -> Result<Output> {
    Git::default().run(cwd, args)
}

/// Run Git using the specified `executable` in `cwd` with `args`.
pub fn run_git_with(executable: &OsStr, cwd: &Path, args: &[OsString]) -> Result<Output> {
    Git::new(executable.to_os_string()).run(cwd, args)
}