use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

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

/// Query status with `git status`.
#[derive(Debug, Clone, Default)]
pub struct StatusCommand {
    pub untracked_files: bool,
    pub ignore_submodules_dirty: bool,
}

impl GitCommand for StatusCommand {
    type Output = (Vec<StatusEntry>, Vec<StatusEntry>, Vec<StatusEntry>);

    fn to_args(&self) -> Vec<OsString> {
        let mut args = vec![OsString::from("status")];
        args.push(OsString::from("--porcelain=v2"));
        args.push(OsString::from("-z"));

        if self.untracked_files {
            args.push(OsString::from("-uall"));
        }
        if self.ignore_submodules_dirty {
            args.push(OsString::from("--ignore-submodules=dirty"));
        }
        args
    }

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
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

                    match x {
                        'A' => staged.push(StatusEntry {
                            kind: ChangeKind::New,
                            path: path.clone(),
                        }),
                        'M' => staged.push(StatusEntry {
                            kind: ChangeKind::Modified,
                            path: path.clone(),
                        }),
                        'D' => staged.push(StatusEntry {
                            kind: ChangeKind::Deleted,
                            path: path.clone(),
                        }),
                        'R' => staged.push(StatusEntry {
                            kind: ChangeKind::Renamed,
                            path: path.clone(),
                        }),
                        'T' => staged.push(StatusEntry {
                            kind: ChangeKind::TypeChange,
                            path: path.clone(),
                        }),
                        _ => {}
                    }

                    match y {
                        'M' => unstaged.push(StatusEntry {
                            kind: ChangeKind::Modified,
                            path,
                        }),
                        'D' => unstaged.push(StatusEntry {
                            kind: ChangeKind::Deleted,
                            path,
                        }),
                        'R' => unstaged.push(StatusEntry {
                            kind: ChangeKind::Renamed,
                            path,
                        }),
                        'T' => unstaged.push(StatusEntry {
                            kind: ChangeKind::TypeChange,
                            path,
                        }),
                        _ => {}
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

                    match x {
                        'R' => staged.push(StatusEntry {
                            kind: ChangeKind::Renamed,
                            path: path.clone(),
                        }),
                        'A' => staged.push(StatusEntry {
                            kind: ChangeKind::New,
                            path: path.clone(),
                        }),
                        'M' => staged.push(StatusEntry {
                            kind: ChangeKind::Modified,
                            path: path.clone(),
                        }),
                        _ => {}
                    }

                    match y {
                        'R' => unstaged.push(StatusEntry {
                            kind: ChangeKind::Renamed,
                            path,
                        }),
                        'M' => unstaged.push(StatusEntry {
                            kind: ChangeKind::Modified,
                            path,
                        }),
                        'D' => unstaged.push(StatusEntry {
                            kind: ChangeKind::Deleted,
                            path,
                        }),
                        _ => {}
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

        Ok((staged, unstaged, untracked))
    }
}
