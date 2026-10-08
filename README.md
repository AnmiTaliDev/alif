# Alif

[![License: GPL-3.0-only](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)

## About

Alif is a checker for natural deduction proofs in propositional and first-order logic. It is for people who write short formal proofs by hand, and for programs that need to validate such proofs.

## Dependencies

- Rust (stable) and Cargo.
- The `logos` crate. Cargo downloads it during the build.
- A C compiler, needed only for `examples/ffi_demo.c`.

## Build

```sh
cargo build --release
```

Outputs in `target/release/`:

| File | Content |
|------|---------|
| `alif` | command line tool |
| `libalif.so` | shared library (`libalif.dylib` on macOS, `alif.dll` on Windows) |
| `libalif.rlib` | Rust library |

Install the command line tool to `~/.cargo/bin`:

```sh
cargo install --path .
```

Run the tests and the linter:

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```

## Usage

Save a proof as `hello.alif`:

```
theorem hello: A |- A
proof
  assume h: A
  exact h
qed
```

Check it:

```sh
alif verify hello.alif
```

The tool prints `✓ QED` and exits with status 0. If a proof is wrong, it writes a diagnostic to standard error and exits with a non-zero status:

```
<file>:<line>:<column>: proof error in theorem `<name>`, step <n>: <reason>
```

| Exit status | Meaning |
|-------------|---------|
| 0 | all files verified |
| 1 | proof error |
| 2 | syntax, load or usage error |

All options are described in [docs/cli.md](docs/cli.md).

## Language

A file holds `axiom`, `theorem` and `import` items. A theorem states a sequent, written `hypotheses |- conclusion`, followed by a proof:

```
theorem syllogism:
  forall X: human(X) => mortal(X), human(socrates) |- mortal(socrates)
proof
  assume all: forall X: human(X) => mortal(X)
  assume h: human(socrates)
  have step: human(socrates) => mortal(socrates) := ForallElim(all, socrates)
  have m: mortal(socrates) := ModusPonens(step, h)
  exact m
qed
```

A proof is a list of steps:

- `assume name: F` adds the formula `F` as an assumption.
- `have name: F := J` derives `F` with the justification `J` and stores it under `name`.
- `exact J` derives the conclusion and ends the proof.

A justification is a built-in rule, a theorem, an axiom or the name of an earlier step. The checker tracks which assumptions each formula depends on. A proof is accepted only if, at `exact`, the conclusion depends on nothing except the hypotheses of the theorem.

There are 21 built-in rules, covering the connectives, `FALSE`, the quantifiers and equality. A proved theorem can be applied in later proofs, with formulas substituted for its propositional letters. The reference is [docs/inference-rules.md](docs/inference-rules.md).

## Library and C interface

The crate is built as `rlib` and `cdylib`. Rust code calls `alif::verify_source` or `alif::verify_file`. C code uses the functions declared in `include/alif.h`:

```c
int alif_verify(const char *source);
int alif_verify_file(const char *path);
const char *alif_last_error(void);
```

Details are in [docs/api.md](docs/api.md) and [docs/ffi.md](docs/ffi.md).

## Platform support

| Platform | Status |
|----------|--------|
| Linux, FreeBSD, OpenBSD, NetBSD, Illumos | supported |
| macOS | builds, prints a warning to standard error |
| Windows | builds, prints a warning to standard error |

## Acknowledgments

- [`logos`](https://crates.io/crates/logos), the lexer generator used by the tokenizer.
- Gerhard Gentzen, who introduced natural deduction.

## Documentation

| Document | Content |
|----------|---------|
| [docs/syntax.md](docs/syntax.md) | file format, formulas, declarations, scoping |
| [docs/inference-rules.md](docs/inference-rules.md) | proof model and the 21 rules |
| [docs/stdlib.md](docs/stdlib.md) | standard library |
| [docs/cli.md](docs/cli.md) | command line tool |
| [docs/api.md](docs/api.md) | Rust interface |
| [docs/ffi.md](docs/ffi.md) | C interface |
| [docs/errors.md](docs/errors.md) | diagnostics and their causes |
| [docs/examples.md](docs/examples.md) | worked examples and common mistakes |
| [docs/architecture.md](docs/architecture.md) | internals |
| [CONTRIBUTING.md](CONTRIBUTING.md) | development workflow |

## License

Copyright 2026 AnmiTaliDev <anmitalidev@nuros.org>

Alif is licensed under the GNU General Public License, version 3 only (GPL-3.0-only). See [LICENSE](LICENSE).
