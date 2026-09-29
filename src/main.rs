//! sgit - manage projects with (nested) git submodules.

use std::process::ExitCode;

use clap::Parser;
use sgit::cli::{Cli, run};
use sgit::config::load_root_config;

/// Parse CLI args, run the selected command, and map errors to exit code 1.
fn main() -> ExitCode {
    let raw_args: Vec<String> = std::env::args().collect();

    // Expand alias: if the first positional argument matches an alias defined
    // in the root repo config (or global config), replace it with the expansion
    // before handing off to Clap.
    let args = expand_alias(raw_args);

    let cli = Cli::parse_from(args);
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

/// Return `args` with the first positional argument expanded if it is an alias.
fn expand_alias(mut args: Vec<String>) -> Vec<String> {
    // args[0] is the binary name; args[1] (if present) is the subcommand/alias.
    let Some(name) = args.get(1).cloned() else {
        return args;
    };
    // Don't expand flags like --help or --version.
    if name.starts_with('-') {
        return args;
    }
    let cfg = load_root_config();
    let Some(expansion) = cfg.expand_alias(&name) else {
        return args;
    };
    // Replace args[1] with the expanded words.
    args.splice(1..2, expansion);
    args
}
