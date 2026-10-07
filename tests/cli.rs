// SPDX-License-Identifier: GPL-3.0-only

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn alif(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_alif"))
        .args(args)
        .current_dir(root())
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn verify_valid_file() {
    let output = alif(&["verify", "examples/and_comm.alif"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "\u{2713} QED\n");
    assert_eq!(stderr(&output), "");
}

#[test]
fn check_is_an_alias_of_verify() {
    let output = alif(&["check", "examples/and_comm.alif"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "\u{2713} QED\n");
}

#[test]
fn proof_error_exits_with_one() {
    let output = alif(&["verify", "tests/fixtures/invalid_proof/assume_not_a_hypothesis.alif"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    let err = stderr(&output);
    assert!(
        err.starts_with("tests/fixtures/invalid_proof/assume_not_a_hypothesis.alif:4:3: proof error"),
        "{}",
        err
    );
}

#[test]
fn parse_error_exits_with_two() {
    let output = alif(&["verify", "tests/fixtures/invalid_syntax/missing_qed.alif"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("parse error"));
}

#[test]
fn load_error_exits_with_two() {
    let output = alif(&["verify", "tests/fixtures/invalid_load/duplicate_axiom.alif"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("already defined"));
}

#[test]
fn missing_file_exits_with_two() {
    let output = alif(&["verify", "tests/fixtures/does-not-exist.alif"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("cannot read"));
}

#[test]
fn imports_resolve_relative_to_the_file() {
    let output = alif(&["verify", "examples/import_main.alif"]);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn multiple_files_are_labelled() {
    let output = alif(&["verify", "examples/and_comm.alif", "examples/socrates.alif"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        "examples/and_comm.alif: \u{2713} QED\nexamples/socrates.alif: \u{2713} QED\n"
    );
}

#[test]
fn multiple_files_report_every_failure_and_worst_status() {
    let output = alif(&[
        "verify",
        "tests/fixtures/invalid_proof/missing_exact.alif",
        "examples/and_comm.alif",
        "tests/fixtures/invalid_syntax/missing_qed.alif",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).contains("examples/and_comm.alif: \u{2713} QED"));
    let err = stderr(&output);
    assert!(err.contains("proof error"));
    assert!(err.contains("parse error"));
}

#[test]
fn proof_errors_alone_exit_with_one() {
    let output = alif(&[
        "verify",
        "tests/fixtures/invalid_proof/missing_exact.alif",
        "tests/fixtures/invalid_proof/unknown_name.alif",
    ]);
    assert_eq!(output.status.code(), Some(1));
}

fn run_with_stdin(input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_alif"))
        .args(["verify", "-"])
        .current_dir(root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn standard_input_is_verified() {
    let output = run_with_stdin("theorem t: A |- A\nproof\n  assume h: A\n  exact h\nqed\n");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "\u{2713} QED\n");
}

#[test]
fn standard_input_errors_have_no_file_name() {
    let output = run_with_stdin("theorem t: A |- B\nproof\n  assume h: A\n  exact h\nqed\n");
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).starts_with("4:3: proof error"));
}

#[test]
fn standard_input_rejects_imports() {
    let output = run_with_stdin("import \"x.alif\"\n");
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("file location"));
}

#[test]
fn no_arguments_prints_usage() {
    let output = alif(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("usage: alif"));
}

#[test]
fn verify_without_file_prints_usage() {
    let output = alif(&["verify"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("usage: alif"));
}

#[test]
fn unknown_command_is_rejected() {
    let output = alif(&["frobnicate", "x.alif"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("unknown command `frobnicate`"));
}

#[test]
fn help_exits_with_zero() {
    for flag in ["--help", "-h"] {
        let output = alif(&[flag]);
        assert_eq!(output.status.code(), Some(0));
        assert!(stdout(&output).contains("exit status"));
    }
}

#[test]
fn version_matches_manifest() {
    for flag in ["--version", "-V"] {
        let output = alif(&[flag]);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(
            stdout(&output),
            format!("alif {}\n", env!("CARGO_PKG_VERSION"))
        );
    }
}
