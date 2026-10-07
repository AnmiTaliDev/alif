// SPDX-License-Identifier: GPL-3.0-only

use std::fs;
use std::path::{Path, PathBuf};

use alif::{verify_file, AlifError};

fn alif_files(dir: &str) -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
    let mut files: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("cannot read {}: {}", root.display(), e))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "alif"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no .alif files in {}", root.display());
    files
}

fn assert_all_verify(dir: &str) {
    for file in alif_files(dir) {
        if let Err(e) = verify_file(&file) {
            panic!("{} should verify: {}", file.display(), e);
        }
    }
}

fn assert_all_fail(dir: &str, expected: fn(&AlifError) -> bool, kind: &str) {
    for file in alif_files(dir) {
        match verify_file(&file) {
            Ok(()) => panic!("{} should fail with a {} error", file.display(), kind),
            Err(e) => assert!(
                expected(&e),
                "{} should fail with a {} error, got: {}",
                file.display(),
                kind,
                e
            ),
        }
    }
}

#[test]
fn examples_verify() {
    assert_all_verify("examples");
}

#[test]
fn valid_fixtures_verify() {
    assert_all_verify("tests/fixtures/valid");
}

#[test]
fn invalid_proof_fixtures_fail_with_check_error() {
    assert_all_fail(
        "tests/fixtures/invalid_proof",
        |e| matches!(e, AlifError::Check(_)),
        "proof",
    );
}

#[test]
fn invalid_syntax_fixtures_fail_with_parse_error() {
    assert_all_fail(
        "tests/fixtures/invalid_syntax",
        |e| matches!(e, AlifError::Parse(_)),
        "parse",
    );
}

#[test]
fn invalid_load_fixtures_fail_with_load_error() {
    assert_all_fail(
        "tests/fixtures/invalid_load",
        |e| matches!(e, AlifError::Load(_)),
        "load",
    );
}

#[test]
fn every_error_has_a_location() {
    for dir in [
        "tests/fixtures/invalid_proof",
        "tests/fixtures/invalid_syntax",
        "tests/fixtures/invalid_load",
    ] {
        for file in alif_files(dir) {
            let location = match verify_file(&file).unwrap_err() {
                AlifError::Check(e) => e.location,
                AlifError::Parse(e) => e.location,
                AlifError::Load(e) => e.location,
            };
            let location = location.unwrap_or_else(|| panic!("{} has no location", file.display()));
            assert!(location.line >= 1 && location.column >= 1);
        }
    }
}
