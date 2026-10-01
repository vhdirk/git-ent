use crate::Result;
use crate::git::{ParseOutput, ToArgs};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

fn push_file(args: &mut Vec<OsString>, file: &Option<PathBuf>) {
    if let Some(f) = file {
        args.push("--file".into());
        args.push(f.into());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigOutput {
    /// A single configuration value (e.g., from `--get`)
    Value(String),
    /// Multiple values (if `--get-all` is used)
    Values(Vec<String>),
    /// Key-value pairs (from `--list`)
    Entries(Vec<(String, String)>),
    /// Confirmation that a write/unset/add succeeded
    Success,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigScope {
    #[default]
    Default, // No explicit scope flag (uses git's default resolution)
    Local,    // --local (.git/config)
    Global,   // --global (~/.gitconfig)
    System,   // --system (/etc/gitconfig)
    Worktree, // --worktree
}

impl ToArgs for ConfigScope {
    fn to_args(&self, args: &mut Vec<OsString>) {
        match self {
            ConfigScope::Local => args.push("--local".into()),
            ConfigScope::Global => args.push("--global".into()),
            ConfigScope::System => args.push("--system".into()),
            ConfigScope::Worktree => args.push("--worktree".into()),
            ConfigScope::Default => {}
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Get {
    pub name: String, // Key or regular expression pattern (e.g. "^alias\.")
    pub value_pattern: Option<String>,
    pub all: bool,        // --all
    pub show_names: bool, // --show-names
    pub regexp: bool,     // --regexp
    pub scope: ConfigScope,
    pub file: Option<PathBuf>, // --file <path>
}

impl ToArgs for Get {
    fn to_args(&self, args: &mut Vec<OsString>) {
        if self.all {
            args.push("--all".into());
        }
        if self.show_names {
            args.push("--show-names".into());
        }
        if self.regexp {
            args.push("--regexp".into());
        }
        self.scope.to_args(args);
        push_file(args, &self.file);

        args.push(self.name.as_str().into());
        if let Some(vp) = &self.value_pattern {
            args.push(vp.into());
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Set {
    pub name: String,
    pub value: String,
    pub scope: ConfigScope,
    pub file: Option<PathBuf>,
}

#[derive(Default, Debug, Clone)]
pub struct Add {
    pub name: String,
    pub value: String,
    pub scope: ConfigScope,
    pub file: Option<PathBuf>,
}

#[derive(Default, Debug, Clone)]
pub struct Unset {
    pub name: String,
    pub value_pattern: Option<String>,
    pub scope: ConfigScope,
    pub file: Option<PathBuf>,
}

#[derive(Default, Debug, Clone)]
pub struct List {
    pub scope: ConfigScope,
    pub file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum ConfigCmd {
    Get(Get),
    Set(Set),
    Add(Add),
    Unset(Unset),
    List(List),
}

impl ConfigCmd {}

impl ToArgs for ConfigCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("config".into());
        args.push("-z".into());

        match self {
            Self::Get(g) => g.to_args(args),
            Self::Set(s) => {
                s.scope.to_args(args);
                push_file(args, &s.file);
                args.push(s.name.as_str().into());
                args.push(s.value.as_str().into());
            }
            Self::Add(ad) => {
                args.push("--add".into());
                ad.scope.to_args(args);
                push_file(args, &ad.file);
                args.push(ad.name.as_str().into());
                args.push(ad.value.as_str().into());
            }
            Self::Unset(u) => {
                args.push("--unset".into());
                u.scope.to_args(args);
                push_file(args, &u.file);
                args.push(u.name.as_str().into());
                if let Some(vp) = &u.value_pattern {
                    args.push(vp.into());
                }
            }
            Self::List(l) => {
                args.push("--list".into());
                l.scope.to_args(args);
                push_file(args, &l.file);
            }
        }
    }
}

impl ParseOutput for ConfigCmd {
    type Output = ConfigOutput;

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        // Note: git config returns exit code 1 if a key is not found during a get/unset query.
        // You can handle code 1 specially if "not found" is a valid expected state in your app.
        // if !output.status.success() {
        //     return Err(crate::Error::GitCommandFailed {
        //         status: output.status,
        //         stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        //     });
        // }

        let stdout = &output.stdout;

        match self {
            Self::List(_) => {
                let mut entries = Vec::new();
                // Each entry ends with a null byte (\0)
                for entry in stdout.split(|&b| b == b'\0') {
                    if entry.is_empty() {
                        continue;
                    }
                    // Key and value are separated by a single newline (\n)
                    if let Some(pos) = entry.iter().position(|&b| b == b'\n') {
                        let key = String::from_utf8_lossy(&entry[..pos]).to_string();
                        let val = String::from_utf8_lossy(&entry[pos + 1..]).to_string();
                        entries.push((key, val));
                    }
                }
                return Ok(ConfigOutput::Entries(entries));
            }
            Self::Get(g) => {
                let mut values = Vec::new();
                for val_bytes in stdout.split(|&b| b == b'\0') {
                    if val_bytes.is_empty() {
                        continue;
                    }
                    values.push(String::from_utf8_lossy(val_bytes).to_string());
                }
                if g.all {
                    return Ok(ConfigOutput::Values(values));
                } else {
                    return Ok(ConfigOutput::Value(
                        values.into_iter().next().unwrap_or_default(),
                    ));
                }
            }
            Self::Set(_) | Self::Add(_) | Self::Unset(_) => Ok(ConfigOutput::Success),
        }
    }
}
