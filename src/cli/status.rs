use std::path::{Path, PathBuf};

use colored::Colorize;

use crate::RepoTree;
use crate::cli::command::{Cmd, Context};
use crate::error::Result;
use crate::repo_tree::prefix_path;
use clap::Args;

/// Show status recursively across all submodules.
#[derive(Default, Debug, Args)]
pub struct StatusCmd;

fn format_path(p: &Path) -> String {
    if p.starts_with("./") {
        return p.strip_prefix("./").unwrap().to_string_lossy().into_owned();
    }
    p.to_string_lossy().into_owned()
}

impl Cmd for StatusCmd {
    fn run(&self, _ctx: &Context) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let tree = RepoTree::discover(None)?;

        let status = tree.status()?;

        if status.is_empty() {
            return Ok(());
        }

        let branch_name = tree
            .root
            .branch_name()
            .unwrap_or_else(|| "(detached HEAD)".to_string());
        println!("On branch {branch_name}");

        if !status.staged.is_empty() {
            println!("\nChanges to be committed:");
            println!("  (use \"git ent restore --staged <file>...\" to unstage)");
            for s in &status.staged {
                println!(
                    "\t{}",
                    format!("{}{}", s.kind.label(), format_path(&s.path)).green()
                );
            }
        }

        if !status.unstaged.is_empty() {
            println!("\nChanges not staged for commit:");
            println!("  (use \"git ent add <file>...\" to update what will be committed)");
            println!(
                "  (use \"git ent restore <file>...\" to discard changes in working directory)"
            );
            for s in &status.unstaged {
                println!(
                    "\t{}",
                    format!("{}{}", s.kind.label(), format_path(&s.path)).red()
                );
            }
        }

        if !status.untracked.is_empty() {
            println!("\nUntracked files:");
            println!("  (use \"git ent add <file>...\" to include in what will be committed)");
            for s in &status.untracked {
                println!("\t{}", format_path(&s.path).red());
            }
        }

        Ok(())
    }
}
