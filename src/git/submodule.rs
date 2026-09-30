use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

use crate::error::Result;
use crate::git::GitCommand;

/// Query submodule paths declared in `.gitmodules`.
#[derive(Debug, Clone, Default)]
pub struct SubmodulePathsCommand;

impl GitCommand for SubmodulePathsCommand {
    type Output = Vec<PathBuf>;

    fn to_args(&self) -> Vec<OsString> {
        vec![
            OsString::from("config"),
            OsString::from("-z"),
            OsString::from("--file"),
            OsString::from(".gitmodules"),
            OsString::from("--get-regexp"),
            OsString::from(r"^submodule\..*\.path$"),
        ]
    }

    fn parse_output(&self, output: &Output) -> Result<Vec<PathBuf>> {
        if !output.status.success() {
            return Ok(Vec::new());
        }
        let bytes = &output.stdout;
        let mut paths = Vec::new();
        for record in bytes.split(|&b| b == 0) {
            if record.is_empty() {
                continue;
            }
            if let Some(pos) = record.iter().position(|&b| b == b'\n') {
                let val_bytes = &record[pos + 1..];
                let val = String::from_utf8_lossy(val_bytes);
                paths.push(PathBuf::from(val.as_ref()));
            }
        }
        Ok(paths)
    }
}

/// A submodule entry parsed from `git submodule status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmoduleStatusEntry {
    pub flag: char,
    pub path: String,
}

impl SubmoduleStatusEntry {
    /// True if the submodule working tree HEAD differs from the index pointer (`+`).
    pub fn is_dirty_pointer(&self) -> bool {
        self.flag == '+'
    }

    /// True if the submodule has not been initialized (`-`).
    pub fn is_uninitialized(&self) -> bool {
        self.flag == '-'
    }
}

/// Query submodule status with `git submodule status`.
#[derive(Debug, Clone, Default)]
pub struct SubmoduleStatusCommand;

impl GitCommand for SubmoduleStatusCommand {
    type Output = Vec<SubmoduleStatusEntry>;

    fn to_args(&self) -> Vec<OsString> {
        vec![OsString::from("submodule"), OsString::from("status")]
    }

    fn parse_output(&self, output: &Output) -> Result<Vec<SubmoduleStatusEntry>> {
        if !output.status.success() {
            return Ok(Vec::new());
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let mut entries = Vec::new();
        for line in text.lines() {
            if line.is_empty() {
                continue;
            }
            let flag = line.chars().next().unwrap_or(' ');
            let rest = &line[1..].trim_start();
            let mut parts = rest.split_whitespace();
            let _sha = parts.next();
            if let Some(path) = parts.next() {
                entries.push(SubmoduleStatusEntry {
                    flag,
                    path: path.to_string(),
                });
            }
        }
        Ok(entries)
    }
}

/// Query changed submodule paths (worktree or index modified) using `git status`.
#[derive(Debug, Clone, Default)]
pub struct ChangedSubmodulesCommand;

impl GitCommand for ChangedSubmodulesCommand {
    type Output = Vec<String>;

    fn to_args(&self) -> Vec<OsString> {
        vec![
            OsString::from("status"),
            OsString::from("--porcelain=v2"),
            OsString::from("-z"),
            OsString::from("--ignore-submodules=dirty"),
        ]
    }

    fn parse_output(&self, output: &Output) -> Result<Vec<String>> {
        let bytes = &output.stdout;
        let mut changed = Vec::new();
        for entry_bytes in bytes.split(|&b| b == 0) {
            if entry_bytes.is_empty() {
                continue;
            }
            let s = String::from_utf8_lossy(entry_bytes);
            if let Some(rest) = s.strip_prefix("1 ") {
                let tokens: Vec<&str> = rest.splitn(8, ' ').collect();
                if tokens.len() == 8 {
                    let xy = tokens[0];
                    let sub = tokens[1];
                    let path = tokens[7];
                    if sub.starts_with('S') {
                        let x = xy.chars().next().unwrap_or('.');
                        let y = xy.chars().nth(1).unwrap_or('.');
                        if x == 'M' || y == 'M' {
                            changed.push(path.to_string());
                        }
                    }
                }
            }
        }
        Ok(changed)
    }
}

/// Initialize and update submodules recursively (`git submodule update --init --recursive`).
#[derive(Debug, Clone, Default)]
pub struct SubmoduleUpdateCommand;

impl GitCommand for SubmoduleUpdateCommand {
    type Output = ();

    fn to_args(&self) -> Vec<OsString> {
        vec![
            OsString::from("-c"),
            OsString::from("protocol.file.allow=always"),
            OsString::from("submodule"),
            OsString::from("update"),
            OsString::from("--init"),
            OsString::from("--recursive"),
        ]
    }

    fn parse_output(&self, _output: &Output) -> Result<()> {
        Ok(())
    }
}
