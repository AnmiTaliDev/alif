# Alif

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0--only-blue.svg)](LICENSES/GPL-3.0-only.txt)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)]()

**A formal proof verifier for propositional and first-order logic.**

---

## Overview

Alif is a lightweight, standalone formal proof verifier written in Rust. It reads
`.alif` source files containing axiom declarations and theorem statements with
step-by-step proofs, then verifies every proof step against a fixed set of
inference rules. Alif is **not** a theorem prover, a type checker, or a proof
assistant that generates proofs — it only *checks* that the proofs you write are
correct.

The checker works entirely in the structural domain: each proof step names a rule
or a prior result, applies it, and the verifier confirms that the formula you
claim to derive matches the formula the rule actually produces. No unification, no
type inference, no tactics — just a mechanical, line-by-line audit of your
reasoning.

---

## Features

- **Proof step verification** — every `assume`, `have`, and `exact` step is
  checked independently and in order; the first failure is reported with the step
  index and a plain-English explanation.
- **First-order logic** — supports `forall` and `exists` quantifiers with
  capture-avoiding substitution, in addition to full propositional connectives.
- **12 built-in inference rules** — a complete set covering conjunction,
  disjunction, negation, implication, and the universal and existential
  quantifiers; see the [Inference rules](#inference-rules) table below.
- **Standard library of axioms** — four foundational axioms (`identity`,
  `and_comm`, `or_comm`, `ex_falso`) are loaded automatically from
  `stdlib/logic.alif` and are available in every proof without any import
  declaration.
- **C FFI via `libalif`** — the library crate is compiled as both an `rlib` and a
  `cdylib`; a single `alif_verify` C function lets any language with a C FFI call
  the verifier.
- **CLI tool** — the `alif` binary (built as a Cargo example) provides the
  `verify` and `check` subcommands for verifying files from the shell.
- **Zero `unsafe` outside FFI** — the only `unsafe` code in the project is the
  thin C-interop layer in `src/lib.rs` that converts a raw C string pointer for
  the FFI entry point.

---

## Quick Install

### Prerequisites

- Rust toolchain (stable channel), 1.70 or later.
- `cargo` available on `PATH`.

### Build the library

```sh
cargo build --release
```

The compiled library (`libalif.so` / `libalif.dylib` / `libalif.a`) is placed in
`target/release/`.

### Build the CLI binary

```sh
cargo build --example alif --release
```

The `alif` binary is placed in `target/release/examples/alif`.

To make it available system-wide, copy or symlink it:

```sh
cp target/release/examples/alif ~/.local/bin/alif
```

---

## Quick Start

Create a file `hello.alif`:

```
-- A simple identity proof: given A, conclude A.
theorem hello: A |- A
proof
  assume h: A
  exact h
qed
```

Run the verifier:

```sh
alif verify hello.alif
```

Expected output:

```
✓ QED
```

---

## Full CLI Usage

```
alif verify <file>
alif check  <file>
```

Both subcommands are identical in behaviour: they read the specified `.alif` file,
parse it, load the standard library, and verify every theorem in declaration
order. The `check` subcommand exists as a convenient alias.

### Exit codes

| Code | Meaning                                        |
|------|------------------------------------------------|
| `0`  | All theorems verified successfully.            |
| `1`  | A proof error was detected (wrong derivation). |
| `2`  | A parse error, I/O error, or unknown command.  |

### Output

On success, the single line `✓ QED` is printed to standard output.

On failure, one of the following is printed to standard error:

```
proof error: proof error at step <N>: <message>
parse error: parse error: <message>
error reading `<file>`: <os-error>
```

---

## Language at a Glance

The following self-contained example demonstrates every major language construct:

```
-- Transitivity of implication:
-- from (A => B) and (B => C) and A, derive C.

axiom refl: A => A

theorem transitivity:
  A => B, B => C, A |- C
proof
  assume ab: A => B
  assume bc: B => C
  assume ha: A
  have b:  B := ModusPonens(ab, ha)
  have c:  C := ModusPonens(bc, b)
  exact c
qed
```

- `axiom refl: A => A` — declares a globally available axiom named `refl`.
- `theorem transitivity: A => B, B => C, A |- C` — states a theorem with three
  hypotheses and a conclusion separated by `|-`.
- `proof … qed` — delimits the proof block.
- `assume ab: A => B` — introduces a hypothesis by name into scope.
- `have b: B := ModusPonens(ab, ha)` — derives `B` using the `ModusPonens` rule
  with `ab` and `ha` as premises, checks that the result equals the declared
  formula `B`, then brings `b` into scope.
- `exact c` — finishes the proof by asserting that the in-scope name `c` holds
  the theorem's conclusion `C`.

---

## Worked Examples

Two complete example files are included in `examples/`:

### `examples/socrates.alif`

A minimal example showing predicate application syntax. The theorem
`socrates_mortal` proves `mortal(socrates) |- mortal(socrates)` using only an
`assume` and an `exact`:

```
theorem socrates_mortal:
  mortal(socrates) |- mortal(socrates)
proof
  assume h: mortal(socrates)
  exact h
qed
```

Run it:

```sh
alif verify examples/socrates.alif
```

### `examples/and_comm.alif`

A step-by-step proof of conjunction commutativity. The theorem
`and_comm` proves `A AND B |- B AND A` using `AndElimRight`, `AndElimLeft`, and
`AndIntro`:

```
theorem and_comm:
  A AND B |- B AND A
proof
  assume h: A AND B
  have b: B := AndElimRight(h)
  have a: A := AndElimLeft(h)
  have result: B AND A := AndIntro(b, a)
  exact result
qed
```

Run it:

```sh
alif verify examples/and_comm.alif
```

---

## Using as a Rust Library

Add `alif` to your `Cargo.toml`:

```toml
[dependencies]
alif = { path = "path/to/alif" }
```

The primary public entry point is `alif::verify_source`:

```rust
use alif::verify_source;

fn main() {
    let source = r#"
theorem id: A |- A
proof
  assume h: A
  exact h
qed
"#;

    match verify_source(source) {
        Ok(()) => println!("All proofs verified."),
        Err(e) => eprintln!("Verification failed: {}", e),
    }
}
```

`verify_source` parses the source string, loads the standard library axioms,
collects any `axiom` declarations found in the source, and then verifies every
`theorem` in declaration order. It returns `Ok(())` only when all theorems pass.
On failure it returns an `AlifError`, which is either an `AlifError::Parse` or an
`AlifError::Check` variant, both implementing `std::error::Error` and `Display`.

Other public types available from the crate root:

| Item                   | Description                                         |
|------------------------|-----------------------------------------------------|
| `alif::parse_source`   | Parse source to a `Vec<Item>` without checking.     |
| `alif::checker::check` | Check a single `Theorem` against an axiom map.      |
| `alif::Formula`        | The formula AST enum.                               |
| `alif::Item`           | Top-level AST item (`Axiom` or `Theorem`).          |
| `alif::Theorem`        | Theorem struct with hypotheses, conclusion, steps.  |
| `alif::AlifError`      | Top-level error enum.                               |
| `alif::ParseError`     | Parse-phase error.                                  |
| `alif::CheckError`     | Proof-checking error.                               |

---

## Using via C FFI

The crate compiles to a `cdylib` that exports a single C-compatible function:

```c
int alif_verify(const char *source);
```

**Parameters:**

- `source` — a valid, non-null, null-terminated UTF-8 string containing Alif
  source text. The caller is responsible for the lifetime of the pointer.

**Return value:**

| Value | Meaning                                 |
|-------|-----------------------------------------|
| `0`   | All proofs verified successfully.       |
| `1`   | A proof error was detected.             |
| `2`   | Parse error, invalid UTF-8, or null ptr.|

Example C program:

```c
#include <stdio.h>

// Declared in the generated header or manually:
extern int alif_verify(const char *source);

int main(void) {
    const char *src =
        "theorem id: A |- A\n"
        "proof\n"
        "  assume h: A\n"
        "  exact h\n"
        "qed\n";

    int result = alif_verify(src);
    if (result == 0) {
        puts("✓ QED");
    } else {
        fprintf(stderr, "verification failed: code %d\n", result);
    }
    return result;
}
```

Compile and link:

```sh
gcc -o demo demo.c -L target/release -lalif -Wl,-rpath,target/release
```

---

## Inference Rules

Alif provides 12 built-in inference rules. All arguments to a rule must be names
of formulas currently in scope (introduced by `assume` or derived by `have`).

| Rule name       | Premises                          | Conclusion          | Notes                                    |
|-----------------|-----------------------------------|---------------------|------------------------------------------|
| `AndIntro`      | `A`, `B`                          | `A AND B`           | Conjunction introduction.                |
| `AndElimLeft`   | `A AND B`                         | `A`                 | Left projection of a conjunction.        |
| `AndElimRight`  | `A AND B`                         | `B`                 | Right projection of a conjunction.       |
| `OrIntroLeft`   | `A`, `B`                          | `A OR B`            | Left injection; both args required.      |
| `OrIntroRight`  | `B`, `A`                          | `A OR B`            | Right injection; arg order is `B, A`.    |
| `ModusPonens`   | `A => B`, `A`                     | `B`                 | Accepts either argument order.           |
| `ImpliesIntro`  | `A`, `B`                          | `A => B`            | Packages two in-scope formulas.          |
| `NotIntro`      | `A`                               | `NOT A`             | Negation introduction.                   |
| `NotElim`       | `A`, `NOT A`                      | `⊥`                 | Produces the bottom constant `⊥`.        |
| `ForallIntro`   | `X` (Var), `F`                    | `forall X: F`       | `X` must be a bare variable name.        |
| `ForallElim`    | `forall X: F`, `t`                | `F[X := t]`         | Substitutes `t` for `X` in `F`.         |
| `ExistsIntro`   | `X` (Var), `t`, `F[X := t]`      | `exists X: F`       | Three arguments: variable, witness, body.|

> **Note on `OrIntroLeft` and `OrIntroRight`:** both rules require two arguments.
> `OrIntroLeft(a, b)` produces `A OR B` from premise `A` (first arg) and `B`
> (second arg, used only as the right-hand type). `OrIntroRight(b, a)` produces
> `A OR B` from premise `B` (first arg) and places it on the right; `a` is used
> as the left type.

---

## Standard Library

The file `stdlib/logic.alif` is compiled into the library binary at build time
(via `include_str!`) and is loaded automatically before any user proof is checked.
It declares four foundational axioms:

| Axiom name  | Formula      | Informal meaning                            |
|-------------|--------------|---------------------------------------------|
| `identity`  | `A`          | A bare propositional variable.              |
| `and_comm`  | `A AND B`    | A conjunction of two variables.             |
| `or_comm`   | `A OR B`     | A disjunction of two variables.             |
| `ex_falso`  | `NOT A`      | Negation of a propositional variable.       |

These axioms are available by name in every proof file without any declaration.
User-defined `axiom` declarations in a `.alif` file are added to the same axiom
map and shadow any standard-library axiom with the same name.

---

## Platform Support

Alif is developed and tested on free Unix-like systems. The following platforms
are supported:

| Platform | Status    | Notes                        |
|----------|-----------|------------------------------|
| Linux    | Supported | Primary development platform |
| FreeBSD  | Supported |                              |
| OpenBSD  | Supported |                              |
| NetBSD   | Supported |                              |
| Illumos  | Supported |                              |
| macOS    | Compiles  | Proprietary — warning issued |
| Windows  | Compiles  | Proprietary — warning issued |

Running on a proprietary operating system prints a warning to standard error:

```
# On macOS:
warning: Alif is running on macOS. macOS is proprietary software; consider a free operating system such as GNU/Linux, FreeBSD, or OpenBSD.

# On Windows:
warning: Alif is untested on Windows. Windows is proprietary and not recommended for this tool.
```

---

## Building and Testing

```sh
# Build library and CLI in debug mode
cargo build

# Build in release mode
cargo build --release

# Build the CLI binary
cargo build --example alif --release

# Run the test suite
cargo test

# Run with all warnings treated as errors (required for contributions)
cargo clippy -- -D warnings
```

All tests live alongside the source modules they cover and are run by `cargo test`
without additional configuration.

---

## License

Copyright 2026 AnmiTaliDev \<anmitalidev@nuros.org\>

Alif is free software released under the **GNU General Public License, version 3
only** (GPL-3.0-only). You may redistribute and/or modify it under the terms of
that license. See `LICENSES/GPL-3.0-only.txt` for the full license text.
