# Contributing

## Getting started

Install a stable Rust toolchain with Cargo, then:

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

Tests and clippy have to pass before a change is submitted. The CI job also builds the C example, `examples/ffi_demo.c`, see [docs/ffi.md](docs/ffi.md).

The modules and the test layout are described in [docs/architecture.md](docs/architecture.md). New Rust files begin with the header the existing files use:

```
// SPDX-License-Identifier: GPL-3.0-only
```

## Bug reports

Please include:

- the `.alif` input, reduced as far as possible,
- the command line,
- the complete output and exit status,
- `alif --version` and the operating system.

For a proof that is accepted but should not be, say why the statement is not valid. For a proof that is rejected but should be accepted, give the derivation that justifies it.

## Tests

Proof-level tests are `.alif` files under `tests/fixtures/`:

| Directory | Expected result |
|-----------|-----------------|
| `valid` | verifies |
| `invalid_proof` | proof error, exit status 1 |
| `invalid_syntax` | parse error, exit status 2 |
| `invalid_load` | load error, exit status 2 |

`tests/fixtures.rs` runs every `.alif` file in these directories and in `examples/`. A file in `invalid_proof` should contain one defect, so that a failure has one cause.

Unit tests are in a `tests` submodule of the module they cover.

## Adding a rule

1. Add a variant to `Rule` and an entry to the `RULES` table in `src/rules.rs`, and update the length of the array.
2. Implement it in `propositional`, `quantifier` or `equality`. Return the dependencies of the result. A rule that discharges an assumption has to remove exactly that assumption.
3. If the rule binds or frees a variable, check that the variable is not fixed, as `ForallIntro` and `ExistsElim` do.
4. Add unit tests for success, wrong argument shapes and dependencies.
5. Add valid and invalid fixtures.
6. Update `docs/inference-rules.md` and `docs/errors.md`, and every place that states the number of rules.

State in the change why each accepted input is a valid inference.

## Changing the standard library

The library is `stdlib/logic.alif`. It is checked on every run, so an error in it breaks every run. Add a new theorem to the list in the test in `src/stdlib.rs` and to the table in `docs/stdlib.md`.

## Documentation

Documentation lives in `README.md` and `docs/`. It has to describe what the code does. Messages quoted in it have to match the messages in the source, and `.alif` snippets have to verify, or fail in the way the text says.

## License

Contributions are licensed under GPL-3.0-only, like the rest of the project.
