// SPDX-License-Identifier: GPL-3.0-only

pub mod checker;
pub mod context;
pub mod error;
pub mod ffi;
pub mod formula_ops;
pub mod lexer;
pub mod parser;
pub mod rules;
pub mod stdlib;
pub mod term;
pub mod verify;

pub use checker::check;
pub use context::{Context, Sequent};
pub use error::{AlifError, CheckError, LoadError, Location, ParseError};
pub use parser::parse_source;
pub use term::{Formula, Item, Justification, ProofStep, Term, Theorem};
pub use verify::{verify_file, verify_source};

pub fn platform_warning() -> Option<&'static str> {
    if cfg!(target_os = "windows") {
        Some("warning: Alif is untested on Windows. Windows is proprietary and not recommended for this tool.")
    } else if cfg!(target_os = "macos") {
        Some("warning: Alif is running on macOS. macOS is proprietary software; consider a free operating system such as GNU/Linux, FreeBSD, or OpenBSD.")
    } else {
        None
    }
}
