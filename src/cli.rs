//! The `clap`-derived CLI surface for sgit.

use std::path::PathBuf;

use crate::cmd::add::AddCmd;
use crate::cmd::branch::BranchCmd;
use crate::cmd::checkout::CheckoutCmd;
use crate::cmd::clone::CloneCmd;
use crate::cmd::command::{Command, Context};
use crate::cmd::commit::CommitCmd;
use crate::cmd::merge::MergeCmd;
use crate::cmd::push::PushCmd;
use crate::cmd::rebase::RebaseCmd;
use crate::cmd::reset::ResetCmd;
use crate::cmd::restore::RestoreCmd;
use crate::cmd::squash::SquashCmd;
use crate::cmd::status::StatusCmd;
use crate::cmd::switch::SwitchCmd;
use crate::cmd::update::UpdateCmd;
use clap::{Parser, Subcommand};

use crate::error::Result;

/// sgit (/ʃɪt/) - manage projects with (nested) git submodules.
#[derive(Debug, Parser)]
#[command(name = "sgit", version, about, long_about = None)]
pub struct Cli {
    #[arg(short = 'C')]
    pub workdir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    Clone(CloneCmd),
    Update(UpdateCmd),
    Status(StatusCmd),
    Branch(BranchCmd),
    Checkout(CheckoutCmd),
    Switch(SwitchCmd),
    Add(AddCmd),
    Commit(CommitCmd),
    Push(PushCmd),
    Restore(RestoreCmd),
    Reset(ResetCmd),
    Merge(MergeCmd),
    Rebase(RebaseCmd),
    Squash(SquashCmd),
}

pub fn run(cli: Cli) -> Result<()> {
    let ctx = Context {
        workdir: cli.workdir.clone(),
    };

    match cli.command {
        CliCommand::Clone(args) => args.run(&ctx),
        CliCommand::Update(args) => args.run(&ctx),
        CliCommand::Status(args) => args.run(&ctx),
        CliCommand::Branch(args) => args.run(&ctx),
        CliCommand::Checkout(args) => args.run(&ctx),
        CliCommand::Switch(args) => args.run(&ctx),
        CliCommand::Add(args) => args.run(&ctx),
        CliCommand::Commit(args) => args.run(&ctx),
        CliCommand::Push(args) => args.run(&ctx),
        CliCommand::Restore(args) => args.run(&ctx),
        CliCommand::Reset(args) => args.run(&ctx),
        CliCommand::Merge(args) => args.run(&ctx),
        CliCommand::Rebase(args) => args.run(&ctx),
        CliCommand::Squash(args) => args.run(&ctx),
    }
}
