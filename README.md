# Alif

[![License: GPL-3.0-only](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)

## About

Alif is a proof checker for propositional and first-order logic. It reads `.alif` files that contain axioms and theorems with step-by-step natural deduction proofs, and verifies every step. It is meant for people who write small formal proofs by hand and for programs that need to check such proofs.

## Dependencies

- Rust toolchain (stable) with Cargo.
- The `logos` crate, fetched by Cargo during the build.
- A C compiler, only to build the C example in `examples/ffi_demo.c`.

## Build

```sh
cargo build --release
```

The build produces:

| Artifact | Path |
|----------|------|
| CLI | `target/release/alif` |
| Shared library | `target/release/libalif.so` (`.dylib` on macOS, `.dll` on Windows) |
| Rust library | `target/release/libalif.rlib` |

Install the CLI into `~/.cargo/bin`:

```sh
cargo install --path .
```

Run the tests and the linter:

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```

## Usage

Write a proof in `hello.alif`:

```
theorem hello: A |- A
proof
  assume h: A
  exact h
qed
```

Verify it:

```sh
alif verify hello.alif
```

On success the CLI prints `✓ QED` and exits with status 0. On failure it prints the location and the reason to standard error, in the form:

```
<file>:<line>:<column>: proof error in theorem `<name>`, step <n>: <reason>
```

Exit status is 0 when every file verified, 1 for a proof error, and 2 for a syntax, load or usage error. The full command reference is in [docs/cli.md](docs/cli.md).

## Language

A file is a sequence of `axiom`, `theorem` and `import` items. A theorem states a sequent, `hypotheses |- conclusion`, and gives a proof as a list of steps:

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

- `assume name: F` introduces a formula as an assumption.
- `have name: F := J` derives `F` by a rule, a theorem, an axiom or an earlier name.
- `exact J` ends the proof and must derive the conclusion.

Every derived formula records the assumptions it depends on. `exact` succeeds only when each remaining assumption is one of the theorem's hypotheses. Rules such as `ImpliesIntro` and `NotIntro` remove an assumption from that record.

Alif provides 21 inference rules for conjunction, disjunction, implication, negation, equivalence, `FALSE`, quantifiers and equality. A theorem that was proved earlier in the same file, in an imported file or in the standard library can be applied to later steps, with propositional variables replaced by formulas.

## Library and C API

The crate builds as `rlib` and `cdylib`. The Rust entry points are `alif::verify_source` and `alif::verify_file`. The C entry points are declared in `include/alif.h`:

```c
int alif_verify(const char *source);
int alif_verify_file(const char *path);
const char *alif_last_error(void);
```

See [docs/api.md](docs/api.md) and [docs/ffi.md](docs/ffi.md).

## Platform support

| Platform | Status |
|----------|--------|
| Linux, FreeBSD, OpenBSD, NetBSD, Illumos | Supported |
| macOS | Builds. A warning is printed to standard error. |
| Windows | Builds. A warning is printed to standard error. |

## Acknowledgments

- The [`logos`](https://crates.io/crates/logos) crate, used for lexing.
- Gerhard Gentzen, who introduced natural deduction, on which the rule set is based.

## Documentation

- [docs/syntax.md](docs/syntax.md): lexical rules, grammar and name scoping.
- [docs/inference-rules.md](docs/inference-rules.md): proof model, dependencies and all 21 rules.
- [docs/stdlib.md](docs/stdlib.md): standard library theorems.
- [docs/cli.md](docs/cli.md): command line interface.
- [docs/api.md](docs/api.md): Rust API.
- [docs/ffi.md](docs/ffi.md): C API.
- [docs/errors.md](docs/errors.md): error kinds, formats and messages.
- [docs/examples.md](docs/examples.md): annotated examples and common mistakes.
- [docs/architecture.md](docs/architecture.md): modules and design.
- [CONTRIBUTING.md](CONTRIBUTING.md): development workflow.

## License

Copyright 2026 AnmiTaliDev <anmitalidev@nuros.org>

Alif is released under the GNU General Public License, version 3 only (GPL-3.0-only). The full text is in [LICENSE](LICENSE).
