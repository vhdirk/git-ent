//! Tests for the git process boundary.

mod common;

use sgit::error::SgitError;
use sgit::git::run;
use std::ffi::OsString;
use std::path::Path;

#[test]
fn successful_invocation_in_specified_cwd() {
    let repo = common::plain_repo();
    let args = [OsString::from("status")];
    let output = run(&repo.path, &args, false).expect("git status should succeed");
    assert!(output.status.success());

    let rev_parse_args = [
        OsString::from("rev-parse"),
        OsString::from("--show-toplevel"),
    ];
    let rev_parse_out = run(&repo.path, &rev_parse_args, false).expect("rev-parse should succeed");
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
    run(&repo.path, &set_args, false)
        .expect("setting config key with spaces/metacharacters should succeed");

    let get_args = [
        OsString::from("config"),
        OsString::from("--get"),
        OsString::from("custom.weird-key"),
    ];
    let output = run(&repo.path, &get_args, false).expect("reading config should succeed");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), weird_value);

    // Verify leading dashes do not get interpreted as flags
    let dash_value = "--not-a-flag";
    let set_dash_args = [
        OsString::from("config"),
        OsString::from("custom.dash-key"),
        OsString::from(dash_value),
    ];
    run(&repo.path, &set_dash_args, false)
        .expect("setting config key with leading dash should succeed");

    let get_dash_args = [
        OsString::from("config"),
        OsString::from("--get"),
        OsString::from("custom.dash-key"),
    ];
    let dash_output =
        run(&repo.path, &get_dash_args, false).expect("reading dash config should succeed");
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
    let result = run(&repo.path, &args, false);

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
fn invalid_workdir_reports_launch_failure() {
    let non_existent = Path::new("/nonexistent-dir-12345");
    let args = [OsString::from("status")];
    let result = run(non_existent, &args, false);

    match &result {
        Err(SgitError::GitLaunch { workdir, .. }) => {
            assert_eq!(workdir.as_path(), non_existent);
        }
        other => panic!("expected SgitError::GitLaunch, got {other:?}"),
    }

    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains(&non_existent.display().to_string()),
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
    let output = run(&repo.path, &args, true).expect("allow_failure should return Ok(Output)");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("non-existent-branch-xyz"));
}
