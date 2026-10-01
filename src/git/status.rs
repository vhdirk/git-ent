use std::{ffi::OsString, process::Output};

use crate::git::{ParseOutput, ToArgs};

// TODO: map full command-line interface for `git status`

/// A file change classification, modelled after `git status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    New,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
}

impl ChangeKind {
    /// Left-padded label matching `git status` output.
    pub fn label(&self) -> &'static str {
        match self {
            ChangeKind::New => "new file:   ",
            ChangeKind::Modified => "modified:   ",
            ChangeKind::Deleted => "deleted:    ",
            ChangeKind::Renamed => "renamed:    ",
            ChangeKind::TypeChange => "typechange: ",
        }
    }
}

/// A single status entry for a file.
#[derive(Debug, Clone)]
pub struct StatusEntry {
    pub kind: ChangeKind,
    /// Path relative to the owning repository's working directory.
    pub path: String,
}

fn parse_change_kind(x: char) -> Option<ChangeKind> {
    match x {
        'A' => Some(ChangeKind::New),
        'M' => Some(ChangeKind::Modified),
        'D' => Some(ChangeKind::Deleted),
        'R' => Some(ChangeKind::Renamed),
        'T' => Some(ChangeKind::TypeChange),
        _ => None,
    }
}

pub struct Status {
    pub staged: Vec<StatusEntry>,
    pub unstaged: Vec<StatusEntry>,
    pub untracked: Vec<StatusEntry>,
}

impl Status {
    pub fn is_empty(&self) -> bool {
        self.staged.is_empty() && self.unstaged.is_empty() && self.untracked.is_empty()
    }
}

pub struct StatusCmd {
    pub untracked_files: bool,
    pub ignore_submodules_dirty: bool,
}

impl ToArgs for StatusCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("status".into());
        args.push("--porcelain=v2".into());
        args.push("-z".into());
        if self.untracked_files {
            args.push("-uall".into());
        }

        if self.ignore_submodules_dirty {
            args.push("--ignore-submodules=dirty".into());
        }
    }
}

impl ParseOutput for StatusCmd {
    type Output = Status;

    fn parse_output(&self, output: &Output) -> crate::Result<Status> {
        let bytes = &output.stdout;
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();

        let mut iter = bytes.split(|&b| b == 0);
        while let Some(entry_bytes) = iter.next() {
            if entry_bytes.is_empty() {
                continue;
            }
            let s = String::from_utf8_lossy(entry_bytes);
            if let Some(rest) = s.strip_prefix("1 ") {
                // Ordinary change: 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
                let tokens: Vec<&str> = rest.splitn(8, ' ').collect();
                if tokens.len() == 8 {
                    let xy = tokens[0];
                    let path = tokens[7].to_string();
                    let x = xy.chars().next().unwrap_or('.');
                    let y = xy.chars().nth(1).unwrap_or('.');

                    if let Some(x_kind) = parse_change_kind(x) {
                        staged.push(StatusEntry {
                            kind: x_kind,
                            path: path.clone(),
                        });
                    }

                    if let Some(y_kind) = parse_change_kind(y) {
                        unstaged.push(StatusEntry { kind: y_kind, path });
                    }
                }
            } else if let Some(rest) = s.strip_prefix("2 ") {
                // Renamed/copied entry: 2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <Xscore> <path>
                let tokens: Vec<&str> = rest.splitn(9, ' ').collect();
                let _orig_path = iter.next(); // consume origPath in NUL-delimited stream
                if tokens.len() == 9 {
                    let xy = tokens[0];
                    let path = tokens[8].to_string();
                    let x = xy.chars().next().unwrap_or('.');
                    let y = xy.chars().nth(1).unwrap_or('.');

                    if let Some(x_kind) = parse_change_kind(x) {
                        staged.push(StatusEntry {
                            kind: x_kind,
                            path: path.clone(),
                        });
                    }

                    if let Some(y_kind) = parse_change_kind(y) {
                        unstaged.push(StatusEntry { kind: y_kind, path });
                    }
                }
            } else if let Some(path) = s.strip_prefix("? ") {
                untracked.push(StatusEntry {
                    kind: ChangeKind::New,
                    path: path.to_string(),
                });
            } else if let Some(rest) = s.strip_prefix("u ") {
                // Unmerged: u <XY> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>
                let tokens: Vec<&str> = rest.splitn(10, ' ').collect();
                if tokens.len() == 10 {
                    let path = tokens[9].to_string();
                    unstaged.push(StatusEntry {
                        kind: ChangeKind::Modified,
                        path,
                    });
                }
            }
        }

        Ok(Status {
            staged,
            unstaged,
            untracked,
        })
    }
}
