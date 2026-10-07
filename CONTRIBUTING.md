# Contributing

## Setup

Install a stable Rust toolchain with Cargo. Clone the repository and run:

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

Both `cargo test` and `cargo clippy` have to pass before a change is submitted.

The C example in `examples/ffi_demo.c` is built by the CI job. To run it locally, follow [docs/ffi.md](docs/ffi.md).

## Layout

The modules and their responsibilities are listed in [docs/architecture.md](docs/architecture.md). Tests are described there as well.

New Rust source files start with the line used by the existing files:

```
// SPDX-License-Identifier: GPL-3.0-only
```

## Reporting a problem

Include in the report:

- the `.alif` file or the smallest excerpt that shows the problem,
- the command that was run,
- the complete output and the exit status,
- the output of `alif --version` and the operating system.

For a proof that is accepted but should not be, state which statement is not valid. For a proof that is rejected but should be accepted, state the derivation that justifies it.

## Adding a test

Proof-level tests are files. Put a new `.alif` file into the matching directory under `tests/fixtures/`:

| Directory | Expected result |
|-----------|-----------------|
| `valid` | verifies |
| `invalid_proof` | fails with a proof error (exit status 1) |
| `invalid_syntax` | fails with a parse error (exit status 2) |
| `invalid_load` | fails with a load error (exit status 2) |

`tests/fixtures.rs` picks up every `.alif` file in these directories and in `examples/`. Files in `invalid_proof` should contain one defect each, so that a failure points at one cause.

Unit tests live in the module they test, in a `tests` submodule.

## Adding an inference rule

1. Add a variant to `Rule` and an entry to the `RULES` table in `src/rules.rs`. Update the array length.
2. Implement it in the matching function in `src/rules.rs`: `propositional`, `quantifier` or `equality`. Return the dependencies of the result. A rule that discharges an assumption has to remove exactly that assumption.
3. If the rule introduces or eliminates a variable, check that the variable is not fixed, as `ForallIntro` and `ExistsElim` do.
4. Add unit tests for success, wrong argument shapes and dependencies.
5. Add valid and invalid fixtures.
6. Update `docs/inference-rules.md`, `docs/errors.md` and the rule count in the documentation.

A rule has to be sound. Describe in the change why every accepted input is a valid inference.

## Changing the standard library

The standard library is `stdlib/logic.alif`. Every theorem in it is verified each time the library is loaded, so a mistake in it fails every run. Add the name to the list in the test in `src/stdlib.rs` and to the table in `docs/stdlib.md`.

## Documentation

The documents are in `docs/` and in the README. Keep them in line with the behaviour of the code. Messages quoted in the documentation have to match the messages in the source.

## License

Contributions are licensed under GPL-3.0-only, the license of the project.
