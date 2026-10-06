use std::path::PathBuf;

use colored::Colorize;

use crate::RepoTree;
use crate::cli::command::{Cmd, Context};
use crate::error::Result;
use crate::repo_tree::prefix_path;
use clap::Args;

/// Show status recursively across all submodules.
#[derive(Default, Debug, Args)]
pub struct StatusCmd;

impl Cmd for StatusCmd {
    fn run(&self, ctx: &Context) -> Result<()> {
        let tree = RepoTree::discover(ctx.workdir.as_deref())?;

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
            println!("  (use \"git-nest restore --staged <file>...\" to unstage)");
            for s in &status.staged {
                println!(
                    "\t{}",
                    format!(
                        "{}{}",
                        s.kind.label(),
                        prefix_path(&PathBuf::from("."), &s.path)
                    )
                    .green()
                );
            }
        }

        if !status.unstaged.is_empty() {
            println!("\nChanges not staged for commit:");
            println!("  (use \"git-nest add <file>...\" to update what will be committed)");
            println!(
                "  (use \"git-nest restore <file>...\" to discard changes in working directory)"
            );
            for s in &status.unstaged {
                println!(
                    "\t{}",
                    format!(
                        "{}{}",
                        s.kind.label(),
                        prefix_path(&PathBuf::from("."), &s.path)
                    )
                    .red()
                );
            }
        }

        if !status.untracked.is_empty() {
            println!("\nUntracked files:");
            println!("  (use \"git-nest add <file>...\" to include in what will be committed)");
            for s in &status.untracked {
                println!("\t{}", prefix_path(&PathBuf::from("."), &s.path).red());
            }
        }

        Ok(())
    }
}
