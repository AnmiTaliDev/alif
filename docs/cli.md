# Command line tool

`alif` verifies proof files. Build and install steps are in the [README](../README.md#build).

## Usage

```
alif verify <file>...
alif check <file>...
alif --help
alif --version
```

`check` is another name for `verify`. At least one file is required. The file name `-` stands for standard input.

| Option | Effect |
|--------|--------|
| `-h`, `--help` | prints usage and the exit status table, exits with 0 |
| `-V`, `--version` | prints `alif` and the version, exits with 0 |

Options are recognised only in the first position.

## Behaviour

Files are processed in the order given. Each file is a separate run: the standard library is loaded again, and declarations of one file do not reach another, except through `import`. A failure in one file does not stop the processing of the rest.

Inside a file, declarations are processed from top to bottom, and the first error ends the file.

Paths in `import` are relative to the directory of the importing file. Standard input has no directory, so `import` is an error there.

## Output

A verified file produces one line on standard output:

| Files given | Line |
|-------------|------|
| one | `✓ QED` |
| several | `<argument>: ✓ QED` |

A failed file produces one diagnostic on standard error and nothing on standard output. Formats and causes are in [errors.md](errors.md).

```
$ alif verify examples/and_comm.alif
✓ QED

$ alif verify examples/and_comm.alif examples/socrates.alif
examples/and_comm.alif: ✓ QED
examples/socrates.alif: ✓ QED

$ printf 'theorem t: A |- A\nproof\n  assume h: A\n  exact h\nqed\n' | alif verify -
✓ QED
```

## Exit status

| Status | Meaning |
|--------|---------|
| 0 | all files verified |
| 1 | at least one proof error, and no status 2 |
| 2 | at least one syntax, load or usage error |

Load errors are unreadable files, failed imports, import cycles and duplicate names. Usage errors are a missing or unknown command and `verify` without a file; the usage text goes to standard error. With several files the status is the highest one among them.

## Platform warning

On macOS and Windows the tool prints a warning to standard error before anything else. It does not change the exit status.
