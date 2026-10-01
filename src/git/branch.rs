use crate::error::Result;
use crate::git::{ParseOutput, ToArgs};
use std::ffi::OsString;
use std::process::Output;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub upstream: Option<String>,
}

// --- Configuration Structs ---

#[derive(Default, Debug, Clone)]
pub struct List {
    pub all: bool,                   // -a
    pub remotes: bool,               // -r
    pub verbose: u8,                 // -v or -vv (0 = off, 1 = -v, 2+ = -vv)
    pub merged: Option<String>,      // --merged
    pub no_merged: Option<String>,   // --no-merged
    pub contains: Option<String>,    // --contains
    pub no_contains: Option<String>, // --no-contains
    pub points_at: Option<String>,   // --points-at <commit>
    pub format: Option<String>,      // --format=<format>
    pub sort: Option<String>,        // --sort=<key>
    pub patterns: Vec<String>,
}

impl ToArgs for List {
    fn to_args(&self, args: &mut Vec<OsString>) {
        if self.all {
            args.push("-a".into());
        }
        if self.remotes {
            args.push("-r".into());
        }

        // Handle verbosity (-v or -vv)
        if self.verbose == 1 {
            args.push("-v".into());
        } else if self.verbose >= 2 {
            args.push("-vv".into());
        }

        if let Some(m) = &self.merged {
            args.push("--merged".into());
            args.push(m.into());
        }
        if let Some(nm) = &self.no_merged {
            args.push("--no-merged".into());
            args.push(nm.into());
        }
        if let Some(c) = &self.contains {
            args.push("--contains".into());
            args.push(c.into());
        }
        if let Some(nc) = &self.no_contains {
            args.push("--no-contains".into());
            args.push(nc.into());
        }
        if let Some(pa) = &self.points_at {
            args.push("--points-at".into());
            args.push(pa.into());
        }
        if let Some(f) = &self.format {
            args.push(format!("--format={f}").into());
        }
        if let Some(s) = &self.sort {
            args.push(format!("--sort={s}").into());
        }

        args.extend(self.patterns.iter().map(OsString::from));
    }
}

#[derive(Default, Debug, Clone)]
pub struct Create {
    pub name: String,
    pub start_point: Option<String>,
    pub force: bool, // -f / --force
}

impl ToArgs for Create {
    fn to_args(&self, args: &mut Vec<OsString>) {
        if self.force {
            args.push("--force".into());
        }
        args.push(self.name.as_str().into());
        if let Some(sp) = &self.start_point {
            args.push(sp.into());
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Delete {
    pub names: Vec<String>,
    pub force: bool,  // -D (force) vs -d (safe)
    pub remote: bool, // -r
}

impl ToArgs for Delete {
    fn to_args(&self, args: &mut Vec<OsString>) {
        if self.force {
            args.push("-D".into());
        } else {
            args.push("-d".into());
        }
        if self.remote {
            args.push("-r".into());
        }
        args.extend(self.names.iter().map(OsString::from));
    }
}

#[derive(Default, Debug, Clone)]
pub struct Rename {
    pub old_name: Option<String>,
    pub new_name: String,
    pub force: bool, // -M
}

impl ToArgs for Rename {
    fn to_args(&self, args: &mut Vec<OsString>) {
        if self.force {
            args.push("-M".into());
        } else {
            args.push("-m".into());
        }
        if let Some(old) = &self.old_name {
            args.push(old.into());
        }
        args.push(self.new_name.as_str().into());
    }
}

#[derive(Default, Debug, Clone)]
pub struct SetUpstream {
    pub branch: Option<String>,
    pub upstream: String,
}

impl ToArgs for SetUpstream {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push(format!("--set-upstream-to={}", self.upstream).into());
        if let Some(b) = &self.branch {
            args.push(b.into());
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct UnsetUpstream {
    pub branch: Option<String>,
}

impl ToArgs for UnsetUpstream {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("--unset-upstream".into());
        if let Some(b) = &self.branch {
            args.push(b.into());
        }
    }
}

// --- Main Enum ---

#[derive(Debug, Clone)]
pub enum BranchCmd {
    List(List),
    Create(Create),
    Delete(Delete),
    Rename(Rename),
    SetUpstream(SetUpstream),
    UnsetUpstream(UnsetUpstream),
}

impl ToArgs for BranchCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("branch".into());

        match self {
            Self::List(t) => t.to_args(args),
            Self::Create(t) => t.to_args(args),
            Self::Delete(t) => t.to_args(args),
            Self::Rename(t) => t.to_args(args),
            Self::SetUpstream(t) => t.to_args(args),
            Self::UnsetUpstream(t) => t.to_args(args),
        }
    }
}

impl ParseOutput for BranchCmd {
    type Output = Option<Vec<BranchInfo>>;

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        match self {
            Self::List(_) => {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut branches = Vec::new();

                for line in text.lines() {
                    if line.is_empty() {
                        continue;
                    }

                    let is_current = line.starts_with('*');
                    let clean_line = if line.starts_with('*') || line.starts_with(' ') {
                        &line[2..]
                    } else {
                        line
                    };

                    let branch_part = clean_line.split_whitespace().next().unwrap_or(clean_line);

                    let mut upstream = None;
                    if let Some(start_idx) = clean_line.find('[') {
                        if let Some(end_idx) = clean_line.find(']') {
                            if end_idx > start_idx {
                                upstream = Some(clean_line[start_idx + 1..end_idx].to_string());
                            }
                        }
                    }

                    branches.push(BranchInfo {
                        name: branch_part.to_string(),
                        is_current,
                        upstream,
                    });
                }

                Ok(Some(branches))
            }
            _ => Ok(None),
        }
    }
}
