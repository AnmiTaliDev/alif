// SPDX-License-Identifier: GPL-3.0-only

//! Alif — formal proof verifier library.
//!
//! This crate exposes both a Rust API and a C-compatible FFI surface for
//! verifying Alif proof files.

pub mod checker;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod rules;
pub mod stdlib;
pub mod term;

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

pub use checker::check;
pub use error::{AlifError, CheckError, ParseError};
pub use parser::parse_source;
pub use term::{Formula, Item, Justification, ProofStep, Theorem};

/// Verify all theorems in an Alif source string.
///
/// Parses the source, loads the standard library axioms, then checks every
/// theorem in declaration order.
///
/// # Errors
///
/// Returns an [`AlifError`] on the first failure, either a parse error or a
/// proof error.
pub fn verify_source(source: &str) -> Result<(), AlifError> {
    let items = parse_source(source)?;
    let mut axioms = stdlib::load_stdlib()?;

    for item in &items {
        match item {
            Item::Axiom { name, formula } => {
                axioms.insert(name.clone(), formula.clone());
            }
            Item::Theorem(thm) => {
                check(thm, &axioms)?;
            }
        }
    }

    Ok(())
}

/// C-compatible entry point for verifying an Alif source string.
///
/// # Safety
///
/// `source` must be a valid, non-null, null-terminated UTF-8 C string. The
/// caller is responsible for its lifetime.
///
/// # Return value
///
/// - `0` — all proofs verified successfully
/// - `1` — proof error
/// - `2` — parse error or invalid UTF-8
#[no_mangle]
pub unsafe extern "C" fn alif_verify(source: *const c_char) -> c_int {
    if cfg!(target_os = "windows") {
        eprintln!(
            "warning: Alif is untested on Windows. Windows is proprietary and not recommended for this tool."
        );
    }
    if cfg!(target_os = "macos") {
        eprintln!(
            "warning: Alif is running on macOS. macOS is proprietary software; consider a free operating system such as GNU/Linux, FreeBSD, or OpenBSD."
        );
    }

    // SAFETY: caller guarantees `source` is a valid, non-null, null-terminated
    // UTF-8 C string whose lifetime outlasts this call.
    let c_str = if source.is_null() {
        return 2;
    } else {
        unsafe { CStr::from_ptr(source) }
    };

    let s = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 2,
    };

    match verify_source(s) {
        Ok(()) => 0,
        Err(AlifError::Parse(_)) => 2,
        Err(AlifError::Check(_)) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_identity_proof() {
        let src = r#"
theorem id: A |- A
proof
  assume h: A
  exact h
qed
"#;
        assert!(verify_source(src).is_ok());
    }

    #[test]
    fn verify_invalid_proof_returns_error() {
        let src = r#"
theorem bad: A |- B
proof
  assume h: A
  exact h
qed
"#;
        assert!(verify_source(src).is_err());
    }

    #[test]
    fn parse_error_propagated() {
        assert!(matches!(verify_source("!!!"), Err(AlifError::Parse(_))));
    }
}
