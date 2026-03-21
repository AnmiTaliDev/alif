# Alif CLI Reference

This document is the complete reference for the `alif` command-line interface.

---

## Synopsis

```
alif verify <file>
alif check  <file>
```

---

## Description

`alif` is the command-line interface to the Alif formal proof verifier. It reads
a single `.alif` source file, parses it, loads the standard library axioms
embedded in the verifier binary, and verifies every theorem declared in the file
in the order it appears. If all theorems are proved correctly, the program prints
`✓ QED` to standard output and exits with code `0`. If any theorem fails, or if
the file cannot be read or parsed, the program prints a diagnostic to standard
error and exits with a non-zero code.

`alif` is built as a Cargo example (`examples/alif.rs`) and is compiled with:

```sh
cargo build --example alif --release
```

The resulting binary is located at `target/release/examples/alif`.

`alif` does not modify any file. It is a read-only, purely functional tool: given
the same input file it always produces the same output.

---

## Commands

### `alif verify <file>`

Verifies the `.alif` source file at path `<file>`.

**Behaviour:**

1. Reads the entire contents of `<file>` as a UTF-8 string. If the file cannot
   be opened or read (permission denied, file not found, I/O error, etc.), an
   error message is printed to standard error and the program exits with code `2`.
2. Calls the library's `verify_source` function, which:
   a. Lexes and parses the source text.
   b. Loads the standard library axioms from the binary's embedded copy of
      `stdlib/logic.alif`.
   c. Processes every top-level item in declaration order: `axiom` items are added
      to the axiom map; `theorem` items are checked proof step by proof step.
3. If all theorems verify, prints `✓ QED` to standard output and exits `0`.
4. On the first parse error, prints the error to standard error and exits `2`.
5. On the first proof error, prints the error to standard error and exits `1`.

Processing stops at the first error. Theorems that appear after a failing theorem
are never checked.

### `alif check <file>`

An alias for `alif verify`. The two subcommands are completely identical in every
respect. `check` exists as a convenient alternative spelling. There is no
difference in output, exit codes, or error handling.

---

## Exit Codes

| Code | Condition                                                    |
|------|--------------------------------------------------------------|
| `0`  | All theorems in the file verified successfully.              |
| `1`  | At least one theorem's proof is invalid (proof error).       |
| `2`  | Parse error, I/O error, unknown command, or insufficient arguments. |

Exit code `1` is used exclusively for proof-level errors: a step produced the
wrong formula, a name was not in scope, a rule's arity was wrong, or the final
formula did not match the conclusion. It is never produced by I/O or syntax
failures.

Exit code `2` covers all failure modes that occur before proof checking begins:
file not found, unreadable file, unrecognised character in source, grammar error,
unknown subcommand, or too few arguments on the command line.

---

## Output Formats

### Success

On success, exactly one line is written to **standard output**:

```
✓ QED
```

The `✓` character is Unicode U+2713 (CHECK MARK). Standard error is empty.

### Failure: proof error

When a proof step fails, the following is written to **standard error** (nothing
is written to standard output):

```
proof error: proof error at step <N>: <message>
```

- `<N>` is the zero-based index of the failing step within the theorem's proof
  block. Step `0` is the first step (`assume`, `have`, or `exact`).
- `<message>` is a plain-English description of the failure. Common messages
  include:
  - `derived formula '<F1>' does not match declared formula '<F2>'` — a `have`
    step's justification produced a different formula than the one declared.
  - `` `exact` produced '<F1>' but the theorem's conclusion is '<F2>'`` — the
    formula cited in `exact` does not equal the conclusion.
  - `unknown name '<name>': not in scope and not a known axiom` — the bare name
    used in a justification was not found in the environment or the axiom map.
  - `rule '<R>' failed: <reason>` — a rule application failed; the reason
    describes the arity mismatch or structural incompatibility.
  - `proof has no steps` — the proof block is empty (no steps between `proof` and
    `qed`).

### Failure: parse error

When the source cannot be parsed, the following is written to **standard error**:

```
parse error: parse error: <message>
```

or, when the byte offset of the failure is known:

```
parse error: parse error at offset <byte-offset>: <message>
```

- `<byte-offset>` is the zero-based byte position in the source string where the
  failure was detected.
- `<message>` describes the unexpected token or end-of-input condition. Examples:
  - `unrecognised token at byte offset 42: "@"`
  - `expected identifier, got Qed ("qed")`
  - `expected '|-' or proof, got end of input`

### Failure: I/O error

When the file cannot be read, the following is written to **standard error**:

```
error reading '<file>': <os-error>
```

where `<os-error>` is the operating-system error message (e.g.,
`No such file or directory (os error 2)`).

### Failure: unknown command or missing arguments

When the subcommand is not recognised, or when fewer than two arguments are
provided:

```
usage: alif verify <file.alif>
       alif check  <file.alif>
```

or:

```
unknown command '<cmd>'. Use `verify` or `check`.
```

Both are written to **standard error**.

---

## Examples

### Example 1: verify a correct proof

```sh
$ alif verify examples/socrates.alif
✓ QED
$ echo $?
0
```

### Example 2: verify a more complex proof

```sh
$ alif verify examples/and_comm.alif
✓ QED
```

### Example 3: use the `check` alias

```sh
$ alif check examples/and_comm.alif
✓ QED
```

### Example 4: detect a proof error

Given a file `bad.alif` with content:

```
theorem wrong: A |- B
proof
  assume h: A
  exact h
qed
```

```sh
$ alif verify bad.alif
proof error: proof error at step 1: `exact` produced `A` but the theorem's conclusion is `B`
$ echo $?
1
```

### Example 5: detect a parse error

Given a file `broken.alif` with content:

```
theorem @broken: A |- A
```

```sh
$ alif verify broken.alif
parse error: parse error: unrecognised token at byte offset 8: "@"
$ echo $?
2
```

### Example 6: file not found

```sh
$ alif verify nonexistent.alif
error reading `nonexistent.alif`: No such file or directory (os error 2)
$ echo $?
2
```

### Example 7: missing argument

```sh
$ alif verify
usage: alif verify <file.alif>
       alif check  <file.alif>
$ echo $?
2
```

### Example 8: unknown command

```sh
$ alif run examples/socrates.alif
unknown command `run`. Use `verify` or `check`.
$ echo $?
2
```

### Example 9: wrong formula in a `have` step

Given `mismatch.alif`:

```
theorem t: A AND B |- A
proof
  assume h: A AND B
  have result: B := AndElimLeft(h)
  exact result
qed
```

(`AndElimLeft` produces `A`, but the step declares `B`.)

```sh
$ alif verify mismatch.alif
proof error: proof error at step 1: derived formula `A` does not match declared formula `B`
$ echo $?
1
```

### Example 10: unknown name in justification

Given `unknown.alif`:

```
theorem t: A |- A
proof
  assume h: A
  have x: A := ghost
  exact x
qed
```

```sh
$ alif verify unknown.alif
proof error: proof error at step 1: rule `ghost` failed: name `ghost` is not in scope and is not a known axiom
$ echo $?
1
```

---

## Error Output Reference

The following table summarises all distinct error message patterns that `alif` may
print and the condition that triggers each.

| Message pattern | Exit code | Triggering condition |
|-----------------|-----------|----------------------|
| `✓ QED` (stdout) | `0` | All theorems verified. |
| `proof error: proof error at step <N>: derived formula '<F1>' does not match declared formula '<F2>'` | `1` | A `have` step's justification produced a different formula than declared. |
| ``proof error: proof error at step <N>: `exact` produced '<F1>' but the theorem's conclusion is '<F2>'`` | `1` | `exact` holds a formula that does not equal the theorem's conclusion. |
| `proof error: proof error at step <N>: unknown name '<name>': not in scope and not a known axiom` | `1` | A bare name justification could not be resolved. |
| `proof error: proof error at step <N>: rule '<R>' failed: <reason>` | `1` | A rule application failed (arity, structural mismatch, unknown rule). |
| `proof error: proof error at step <N>: proof ends with '<F>' but conclusion is '<C>'` | `1` | The last formula in the proof does not equal the conclusion (final check). |
| `proof error: proof error at step 0: proof has no steps` | `1` | Proof block is empty. |
| `parse error: parse error: <message>` | `2` | Grammar or lexer error without a known byte offset. |
| `parse error: parse error at offset <N>: <message>` | `2` | Lexer encountered an unrecognised character at the given byte offset. |
| `error reading '<file>': <os-error>` | `2` | The specified file could not be read. |
| `usage: alif verify <file.alif> …` | `2` | Fewer than 2 command-line arguments were given. |
| `unknown command '<cmd>'. Use \`verify\` or \`check\`.` | `2` | The first argument is not `verify` or `check`. |

---

## Windows Note

When the `alif` binary is executed on Windows, it unconditionally prints the
following message to **standard error** before processing any arguments:

```
warning: Alif is untested on Windows. Windows is proprietary and not recommended for this tool.
```

This warning does not affect the exit code or the verification result. Processing
continues normally after the warning is printed. Alif is not tested on Windows,
and the authors make no guarantees about correctness or behaviour on that
platform.

---

## Environment

`alif` does not read or write any environment variables. It does not consult
`HOME`, `PATH`, `XDG_*`, or any other variables. Configuration via environment is
not supported.

---

## Files

`alif` reads exactly one file: the `.alif` source file given as the second
command-line argument. It creates no files, writes no files, and does not consult
any configuration files, home-directory dotfiles, or system-wide settings. The
standard library (`stdlib/logic.alif`) is embedded directly into the binary at
compile time and does not require a file on disk at runtime.
