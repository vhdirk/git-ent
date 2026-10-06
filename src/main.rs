use std::process::ExitCode;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use git_nest::cli::add::AddCmd;
use git_nest::cli::branch::BranchCmd;
use git_nest::cli::checkout::CheckoutCmd;
use git_nest::cli::clone::CloneCmd;
use git_nest::cli::command::{Cmd, Context};
use git_nest::cli::commit::CommitCmd;
use git_nest::cli::merge::MergeCmd;
use git_nest::cli::push::PushCmd;
use git_nest::cli::rebase::RebaseCmd;
use git_nest::cli::reset::ResetCmd;
use git_nest::cli::restore::RestoreCmd;
use git_nest::cli::squash::SquashCmd;
use git_nest::cli::status::StatusCmd;
use git_nest::cli::switch::SwitchCmd;
use git_nest::cli::update::UpdateCmd;

/// git-nest - manage projects with (nested) git submodules.
#[derive(Debug, Parser)]
#[command(name = "git-nest", version, about, long_about = None)]
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

/// Parse CLI args, run the selected command, and map errors to exit code 1.
fn main() -> ExitCode {
    let cli = Cli::parse();

    let ctx = Context {
        workdir: cli.workdir.clone(),
    };

    let result = match cli.command {
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
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}
