// SPDX-License-Identifier: GPL-3.0-only

//! Alif CLI — verify Alif proof files from the command line.
//!
//! Usage:
//!   alif verify <file.alif>
//!   alif check  <file.alif>

use std::env;
use std::fs;
use std::process;

fn main() {
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

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("usage: alif verify <file.alif>");
        eprintln!("       alif check  <file.alif>");
        process::exit(2);
    }

    let command = &args[1];
    let path = &args[2];

    match command.as_str() {
        "verify" | "check" => {}
        other => {
            eprintln!("unknown command `{}`. Use `verify` or `check`.", other);
            process::exit(2);
        }
    }

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading `{}`: {}", path, e);
            process::exit(2);
        }
    };

    match alif::verify_source(&source) {
        Ok(()) => {
            println!("\u{2713} QED");
            process::exit(0);
        }
        Err(alif::AlifError::Parse(e)) => {
            eprintln!("parse error: {}", e);
            process::exit(2);
        }
        Err(alif::AlifError::Check(e)) => {
            eprintln!("proof error: {}", e);
            process::exit(1);
        }
    }
}
