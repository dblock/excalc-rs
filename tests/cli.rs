//! Integration tests that exercise the actual `excalc` binary end-to-end,
//! covering `src/main.rs` (argument parsing, exit codes, stdout/stderr).

use std::process::Command;

fn excalc() -> Command {
    Command::new(env!("CARGO_BIN_EXE_excalc"))
}

#[test]
fn evaluates_a_valid_expression() {
    let output = excalc().arg("2 + 2 * 3").output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "8");
}

#[test]
fn joins_multiple_argv_words_into_one_expression() {
    let output = excalc().args(["2", "+", "2"]).output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "4");
}

#[test]
fn empty_expression_exits_with_usage_error() {
    let output = excalc().output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage: excalc"));
}

#[test]
fn evaluation_error_exits_with_code_one() {
    let output = excalc().arg("1 / 0").output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("error: division by zero"));
}
