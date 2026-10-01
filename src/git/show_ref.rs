use std::{ffi::OsString, process::Output};

use crate::error::Result;
use crate::git::{ParseOutput, ToArgs};

/// Typed output variants for `git show-ref`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitReference {
    pub oid: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShowRefOutput {
    /// Parsed references (used by `List` and `Verify`)
    Refs(Vec<GitReference>),
    /// Result of `--exists` (true if exit status is success)
    Exists(bool),
    /// Filtered lines returned by `--exclude-existing`
    Lines(Vec<String>),
}

#[derive(Default, Debug, Clone)]
pub struct Display {
    pub dereference: bool,      // -d: peeled tags shown with ^{} appended
    pub ids_only: bool,         // -s / --hash: OID only, no ref name
    pub abbrev: Option<Abbrev>, // --abbrev[=<n>]
}

#[derive(Debug, Clone)]
pub enum Abbrev {
    Default,
    Length(u8),
}

#[derive(Default, Debug, Clone)]
pub struct List {
    pub head: bool,     // --head: include HEAD even if filtered out
    pub branches: bool, // --branches (--heads is a deprecated synonym)
    pub tags: bool,     // --tags (combinable with --branches)
    pub display: Display,
    pub patterns: Vec<String>,
}

#[derive(Default, Debug, Clone)]
pub struct Verify {
    pub quiet: bool, // -q: only valid with --verify
    pub display: Display,
    pub refs: Vec<String>,
}

#[derive(Default, Debug, Clone)]
pub struct ExcludeExisting {
    pub pattern: Option<String>,
    pub stdin_lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ShowRefCmd {
    /// `git show-ref [--head] [-d] [-s|--hash[=<n>]] [--abbrev[=<n>]] [--branches] [--tags] [--] [<pattern>...]`
    List(List),
    /// `git show-ref --verify [-q] [-d] [-s|--hash[=<n>]] [--abbrev[=<n>]] [--] [<ref>...]`
    Verify(Verify),
    /// `git show-ref --exclude-existing[=<pattern>]`: stdin filter.
    ExcludeExisting(ExcludeExisting),
    /// `git show-ref --exists <ref>`
    Exists { reference: String },
}

impl ToArgs for Display {
    fn to_args(&self, args: &mut Vec<OsString>) {
        if self.dereference {
            args.push("--dereference".into());
        }
        if self.ids_only {
            args.push("--hash".into());
        }
        match self.abbrev {
            None => {}
            Some(Abbrev::Default) => args.push("--abbrev".into()),
            Some(Abbrev::Length(n)) => args.push(format!("--abbrev={n}").into()),
        }
    }
}

impl ToArgs for ShowRefCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("show-ref".into());
        match self {
            Self::List(l) => {
                if l.head {
                    args.push("--head".into());
                }
                if l.branches {
                    args.push("--branches".into());
                }
                if l.tags {
                    args.push("--tags".into());
                }
                l.display.to_args(args);
                args.push("--".into());
                args.extend(l.patterns.iter().map(OsString::from));
            }
            Self::Verify(v) => {
                args.push("--verify".into());
                if v.quiet {
                    args.push("--quiet".into());
                }
                v.display.to_args(args);
                args.push("--".into());
                args.extend(v.refs.iter().map(OsString::from));
            }
            Self::ExcludeExisting(e) => match &e.pattern {
                Some(p) => args.push(format!("--exclude-existing={p}").into()),
                None => args.push("--exclude-existing".into()),
            },
            Self::Exists { reference } => {
                args.push("--exists".into());
                args.push(reference.as_str().into());
            }
        }
    }
}

/// Helper function to share parsing logic between List and Verify
fn parse_refs_output(output: &Output, display: &Display) -> Result<ShowRefOutput> {
    let text = String::from_utf8_lossy(&output.stdout);
    let mut refs = Vec::new();

    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        if display.ids_only {
            refs.push(GitReference {
                oid: line.to_string(),
                name: String::new(),
            });
        } else {
            let mut parts = line.split_whitespace();
            let oid = parts.next().unwrap_or_default().to_string();
            let name = parts.next().unwrap_or_default().to_string();
            refs.push(GitReference { oid, name });
        }
    }

    Ok(ShowRefOutput::Refs(refs))
}

impl ParseOutput for ShowRefCmd {
    type Output = ShowRefOutput;

    fn allow_failure(&self) -> bool {
        matches!(self, Self::Exists { .. })
    }

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        match self {
            Self::Exists { .. } => {
                // --exists uses exit codes (0 = exists, non-zero = missing/error)
                Ok(ShowRefOutput::Exists(output.status.success()))
            }
            Self::ExcludeExisting { .. } => {
                // --exclude-existing writes remaining unmatched lines to stdout
                let text = String::from_utf8_lossy(&output.stdout);
                let lines = text.lines().map(|l| l.to_string()).collect();
                Ok(ShowRefOutput::Lines(lines))
            }
            Self::List(l) => parse_refs_output(output, &l.display),
            Self::Verify(v) => parse_refs_output(output, &v.display),
        }
    }
}
