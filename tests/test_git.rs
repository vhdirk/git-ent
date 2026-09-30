//! Tests for the git process boundary.

mod common;

use sgit::error::SgitError;
use sgit::git::{run_git, run_git_allow_failure, run_git_with};
use std::ffi::{OsStr, OsString};

#[test]
fn successful_invocation_in_specified_cwd() {
    let repo = common::plain_repo();
    let args = [OsString::from("status")];
    let output = run_git(&repo.path, &args).expect("git status should succeed");
    assert!(output.status.success());

    let rev_parse_args = [
        OsString::from("rev-parse"),
        OsString::from("--show-toplevel"),
    ];
    let rev_parse_out = run_git(&repo.path, &rev_parse_args).expect("rev-parse should succeed");
    let stdout = String::from_utf8_lossy(&rev_parse_out.stdout);
    let canonical_repo = repo.path.canonicalize().unwrap();
    let expected = canonical_repo.to_str().unwrap();
    assert_eq!(stdout.trim(), expected);
}

#[test]
fn distinct_argument_preservation_with_spaces_and_metacharacters() {
    let repo = common::plain_repo();
    let weird_value = "val with spaces and ; && `whoami` $HOME 'single' \"double\"";
    let set_args = [
        OsString::from("config"),
        OsString::from("custom.weird-key"),
        OsString::from(weird_value),
    ];
    run_git(&repo.path, &set_args)
        .expect("setting config key with spaces/metacharacters should succeed");

    let get_args = [
        OsString::from("config"),
        OsString::from("--get"),
        OsString::from("custom.weird-key"),
    ];
    let output = run_git(&repo.path, &get_args).expect("reading config should succeed");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), weird_value);

    // Verify leading dashes do not get interpreted as flags
    let dash_value = "--not-a-flag";
    let set_dash_args = [
        OsString::from("config"),
        OsString::from("custom.dash-key"),
        OsString::from(dash_value),
    ];
    run_git(&repo.path, &set_dash_args)
        .expect("setting config key with leading dash should succeed");

    let get_dash_args = [
        OsString::from("config"),
        OsString::from("--get"),
        OsString::from("custom.dash-key"),
    ];
    let dash_output =
        run_git(&repo.path, &get_dash_args).expect("reading dash config should succeed");
    assert_eq!(
        String::from_utf8_lossy(&dash_output.stdout).trim(),
        dash_value
    );
}

#[test]
fn non_zero_exit_includes_repository_context_and_stderr() {
    let repo = common::plain_repo();
    let args = [
        OsString::from("checkout"),
        OsString::from("non-existent-branch-xyz"),
    ];
    let result = run_git(&repo.path, &args);

    match &result {
        Err(SgitError::GitExit {
            workdir,
            code,
            stderr,
        }) => {
            assert_eq!(workdir.as_path(), repo.path.as_path());
            assert!(code.is_some());
            assert!(stderr.contains("non-existent-branch-xyz"));
        }
        other => panic!("expected SgitError::GitExit, got {other:?}"),
    }

    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains(&repo.path.display().to_string()),
        "error message should contain working directory context: {err_str}"
    );
    assert!(
        err_str.contains("non-existent-branch-xyz"),
        "error message should contain stderr details: {err_str}"
    );
}

#[test]
fn unavailable_executable_reports_launch_failure() {
    let repo = common::plain_repo();
    let args = [OsString::from("status")];
    let result = run_git_with(
        OsStr::new("nonexistent-git-binary-xyz-12345"),
        &repo.path,
        &args,
    );

    match &result {
        Err(SgitError::GitLaunch { workdir, .. }) => {
            assert_eq!(workdir.as_path(), repo.path.as_path());
        }
        other => panic!("expected SgitError::GitLaunch, got {other:?}"),
    }

    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains(&repo.path.display().to_string()),
        "error message should contain working directory context: {err_str}"
    );
}

#[test]
fn allow_failure_returns_non_zero_status() {
    let repo = common::plain_repo();
    let args = [
        OsString::from("checkout"),
        OsString::from("non-existent-branch-xyz"),
    ];
    let output =
        run_git_allow_failure(&repo.path, &args).expect("allow_failure should return Ok(Output)");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("non-existent-branch-xyz"));
}
