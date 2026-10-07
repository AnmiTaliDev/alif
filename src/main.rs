// SPDX-License-Identifier: GPL-3.0-only

use std::env;
use std::io::{self, Read};
use std::path::Path;
use std::process;

use alif::AlifError;

const USAGE: &str = "usage: alif <verify|check> <file>...
       alif --help
       alif --version";

const HELP: &str = "usage: alif <verify|check> <file>...
       alif --help
       alif --version

commands:
  verify <file>...  verify every theorem in each file
  check <file>...   same as verify

A file argument of `-` reads the source from standard input.
Imports are not available for standard input.

exit status:
  0  all files verified
  1  a proof error was found
  2  a parse, load or usage error occurred";

fn main() {
    if let Some(warning) = alif::platform_warning() {
        eprintln!("{}", warning);
    }

    let args: Vec<String> = env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("-h") | Some("--help") => {
            println!("{}", HELP);
            0
        }
        Some("-V") | Some("--version") => {
            println!("alif {}", env!("CARGO_PKG_VERSION"));
            0
        }
        Some("verify") | Some("check") if args.len() > 1 => verify_all(&args[1..]),
        Some("verify") | Some("check") | None => {
            eprintln!("{}", USAGE);
            2
        }
        Some(other) => {
            eprintln!("unknown command `{}`", other);
            eprintln!("{}", USAGE);
            2
        }
    };
    process::exit(code);
}

fn verify_all(paths: &[String]) -> i32 {
    let labelled = paths.len() > 1;
    let mut code = 0;
    for path in paths {
        code = code.max(verify_one(path, labelled));
    }
    code
}

fn verify_one(path: &str, labelled: bool) -> i32 {
    let result = if path == "-" {
        let mut source = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut source) {
            eprintln!("error reading standard input: {}", e);
            return 2;
        }
        alif::verify_source(&source)
    } else {
        alif::verify_file(Path::new(path))
    };

    match result {
        Ok(()) => {
            if labelled {
                println!("{}: \u{2713} QED", path);
            } else {
                println!("\u{2713} QED");
            }
            0
        }
        Err(error) => {
            eprintln!("{}", error);
            exit_code(&error)
        }
    }
}

fn exit_code(error: &AlifError) -> i32 {
    match error {
        AlifError::Check(_) => 1,
        AlifError::Parse(_) | AlifError::Load(_) => 2,
    }
}
