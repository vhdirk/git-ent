//! Tests for top-level `cli::run(Cli)` dispatch.

mod common;

use sgit::cli::{Cli, Command, run};

#[test]
/// Dispatches `status` through the global CLI runner.
fn run_dispatches_status() {
    let p = common::plain_repo();
    let res = common::in_cwd(&p.path, || {
        run(Cli {
            command: Command::Status,
        })
    });
    assert!(res.is_ok());
}

#[test]
/// Dispatches `add` through the global CLI runner.
fn run_dispatches_add() {
    let p = common::plain_repo();
    std::fs::write(p.path.join("new.txt"), "hello\n").unwrap();

    let res = common::in_cwd(&p.path, || {
        run(Cli {
            command: Command::Add {
                filenames: vec!["new.txt".to_string()],
                all: false,
                update: false,
            },
        })
    });
    assert!(res.is_ok());

    let out = common::git_out(&p.path, &["diff", "--cached", "--name-only"]);
    let staged = String::from_utf8_lossy(&out.stdout);
    assert!(staged.contains("new.txt"));
}

#[test]
/// Returns an error for invalid command usage via global CLI runner.
fn run_propagates_command_usage_error() {
    let p = common::plain_repo();
    // Invalid: add with both -A and -u semantics enabled.
    let res = common::in_cwd(&p.path, || {
        run(Cli {
            command: Command::Add {
                filenames: vec![],
                all: true,
                update: true,
            },
        })
    });
    assert!(res.is_err());
}
