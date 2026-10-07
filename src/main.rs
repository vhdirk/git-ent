use std::process::ExitCode;

use clap::{Parser, Subcommand};
use git_ent::cli::add::AddCmd;
use git_ent::cli::branch::BranchCmd;
use git_ent::cli::checkout::CheckoutCmd;
use git_ent::cli::clone::CloneCmd;
use git_ent::cli::command::{Cmd, Context};
use git_ent::cli::commit::CommitCmd;
use git_ent::cli::merge::MergeCmd;
use git_ent::cli::push::PushCmd;
use git_ent::cli::rebase::RebaseCmd;
use git_ent::cli::reset::ResetCmd;
use git_ent::cli::restore::RestoreCmd;
use git_ent::cli::squash::SquashCmd;
use git_ent::cli::status::StatusCmd;
use git_ent::cli::switch::SwitchCmd;
use git_ent::cli::update::UpdateCmd;

/// git-ent - manage projects with (nested) git submodules.
#[derive(Debug, Parser)]
#[command(name = "git-ent", version, about, long_about = None)]
pub struct Cli {
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
    env_logger::init();
    let cli = Cli::parse();

    let ctx = Context::default();

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
