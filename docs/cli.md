# Command line interface

The `alif` executable verifies proof files. It is built from `src/main.rs`. See the [README](../README.md#build) for build and install instructions.

## Synopsis

```
alif verify <file>...
alif check <file>...
alif --help
alif --version
```

## Commands

| Command | Description |
|---------|-------------|
| `verify <file>...` | Verify every theorem in each file. |
| `check <file>...` | Same as `verify`. |

At least one file is required. A file argument of `-` reads the source from standard input.

| Option | Description |
|--------|-------------|
| `-h`, `--help` | Print the usage text and the exit status table. Exit status 0. |
| `-V`, `--version` | Print `alif` and the package version. Exit status 0. |

Options are recognised only as the first argument.

## Processing

Each file is processed on its own. The standard library is loaded for every file, and the declarations of one file are not visible in another, except through `import`. Processing continues with the next file after a failure.

Within a file, items are processed in order. Processing stops at the first error in that file.

Imports are resolved relative to the directory of the file that contains them. Source read from standard input has no directory, so an `import` in it is an error.

## Output

On success, one line is written to standard output.

| Number of files | Output |
|-----------------|--------|
| one | `✓ QED` |
| several | `<path>: ✓ QED` for each file, where `<path>` is the argument as given (`-` for standard input) |

On failure, one line is written to standard error. The format is described in [errors.md](errors.md). Nothing is written to standard output for a file that failed.

```
$ alif verify examples/and_comm.alif
✓ QED

$ alif verify examples/and_comm.alif examples/socrates.alif
examples/and_comm.alif: ✓ QED
examples/socrates.alif: ✓ QED

$ alif verify bad.alif
bad.alif:4:3: proof error in theorem `t`, step 2: the result depends on assumption `h: B`, which is neither discharged nor a hypothesis of the theorem

$ printf 'theorem t: A |- A\nproof\n  assume h: A\n  exact h\nqed\n' | alif verify -
✓ QED
```

## Exit status

| Status | Meaning |
|--------|---------|
| 0 | Every file was verified. |
| 1 | At least one proof error was found, and no file failed with status 2. |
| 2 | A syntax error, a load error or a usage error occurred. |

Load errors cover unreadable files, failed imports, import cycles and duplicate names. With several files the exit status is the highest status of any file.

Usage errors are a missing command, an unknown command, and `verify` or `check` without a file. They print the usage text to standard error.

## Platform warning

On macOS and Windows a warning line is printed to standard error before any other output. The text is `warning: Alif is running on macOS. ...` or `warning: Alif is untested on Windows. ...`. The warning does not change the exit status.
