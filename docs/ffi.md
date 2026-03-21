# Alif — C FFI Reference

This document is the complete reference for the C-compatible foreign-function
interface (FFI) exposed by `libalif`.  It covers the function signature,
parameter contract, return code semantics, linking instructions, thread-safety
guarantees, and complete example programs in C and Python.

---

## Introduction

The `alif` crate is compiled as both an `rlib` (for use from Rust) and a
`cdylib` (C-compatible dynamic library).  The `cdylib` output — `libalif.so` on
Linux, `libalif.dylib` on macOS, `alif.dll` on Windows — exports a single
function, `alif_verify`, decorated with `#[no_mangle]` and the `extern "C"`
calling convention.

**When to use the FFI:**

- You are writing a program in C, C++, Python, Ruby, Go, or any other language
  that can call into a C shared library.
- You want to embed proof verification in a tool that has no Rust toolchain
  dependency at runtime.
- You need a simple go/no-go result and do not need structured error objects.

**When to use the Rust API instead:**

- You are writing a Rust program and want access to rich error types
  (`CheckError`, `ParseError`), the parsed AST, or individual checker
  components.
- You need the step index, the failing `ProofStep`, or any other structured
  information about why a proof failed — the FFI returns only a coarse integer
  code.
- You are embedding Alif as a library dependency in another Rust crate.

---

## C header

Create the following header file and include it in any C (or C++) translation
unit that calls `alif_verify`:

```c
#ifndef ALIF_H
#define ALIF_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * alif_verify — verify all theorems in a null-terminated UTF-8 Alif source string.
 *
 * Parameters
 * ----------
 * source  Non-null pointer to a null-terminated, UTF-8-encoded Alif source string.
 *         The string must remain valid and unmodified for the entire duration of
 *         the call.  The library does not retain any reference to it after the
 *         function returns.
 *
 * Return values
 * -------------
 *   0   All theorems verified successfully.
 *   1   At least one theorem's proof is invalid (proof error).
 *   2   The source could not be parsed, the source pointer is null, or the
 *       string is not valid UTF-8.
 */
extern int alif_verify(const char *source);

#ifdef __cplusplus
}
#endif

#endif /* ALIF_H */
```

Save this as `alif.h` in your project's include path.

---

## Function reference

### `alif_verify`

```c
int alif_verify(const char *source);
```

Verify all theorems in the Alif source string pointed to by `source`.

**Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `source` | `const char *` | Non-null pointer to a null-terminated, UTF-8-encoded Alif source string |

**Return values:**

| Code | Meaning |
|---|---|
| `0` | All theorems in the source verified successfully.  Equivalent to `Ok(())` from `verify_source`. |
| `1` | At least one theorem's proof is incorrect.  This corresponds to `AlifError::Check`.  No structured error information is available through the FFI; use the Rust API for details. |
| `2` | The source could not be parsed (grammar or lexer error), the `source` pointer is `NULL`, or the bytes are not valid UTF-8.  Corresponds to `AlifError::Parse` or a null/invalid input guard. |

**Detailed behaviour:**

1. If `source` is `NULL`, returns `2` immediately without dereferencing the pointer.
2. Constructs a `CStr` from the pointer using `CStr::from_ptr`.  If the bytes
   are not valid UTF-8, returns `2`.
3. Calls `verify_source(s)` on the resulting `&str`.
4. Maps `Ok(())` to `0`, `AlifError::Parse(_)` to `2`, and `AlifError::Check(_)` to `1`.

The function does not print anything to stdout or stderr.  All output is encoded
in the return value.

**Safety requirements:**

- `source` must be either `NULL` or a valid pointer to a contiguous sequence of
  bytes terminated by a `\0` byte.
- The bytes must be valid UTF-8 (a superset of ASCII).
- The pointed-to memory must remain valid and unmodified for the entire duration
  of the call.  The library does not retain the pointer after returning.
- Passing a dangling, misaligned, or non-UTF-8 pointer (other than `NULL`)
  is undefined behaviour.

**Thread safety:**

`alif_verify` is stateless.  It holds no global mutable state; all working
memory is allocated on the stack or in temporary heap allocations that are freed
before the function returns.  It is safe to call `alif_verify` concurrently from
multiple threads with different or the same `source` pointers, provided each
pointer individually satisfies the validity requirements above.

---

## Building the shared library

```sh
cd /path/to/alif
cargo build --release
```

The compiled shared library is placed at:

```
target/release/libalif.so      # Linux
target/release/libalif.dylib   # macOS
target/release/alif.dll        # Windows
```

The `cdylib` crate type is declared in `Cargo.toml`:

```toml
[lib]
name       = "alif"
crate-type = ["rlib", "cdylib"]
```

Both outputs are produced by a single `cargo build` invocation.

---

## Linking

### Linux and other Unix-like systems

Pass `-L` pointing at the directory containing `libalif.so` and use `-lalif` to
let the linker resolve the correct file extension automatically:

```sh
gcc -o myapp myapp.c -I./include -L./target/release -lalif -lpthread -ldl
```

`-lpthread` and `-ldl` are required transitive dependencies of the Rust runtime
on Linux.  On some distributions `libdl` is part of `libc` and the `-ldl` flag
is redundant but harmless.

To run the binary without installing the library system-wide, set
`LD_LIBRARY_PATH`:

```sh
LD_LIBRARY_PATH=./target/release ./myapp proof.alif
```

Alternatively, embed the path at link time with `-Wl,-rpath`:

```sh
gcc -o myapp myapp.c \
    -I./include \
    -L./target/release \
    -Wl,-rpath,'$ORIGIN/../target/release' \
    -lalif -lpthread -ldl
```

Do **not** hardcode the `.so` extension (e.g. `-l:libalif.so`).  Using
`-lalif` allows the same build script to work on macOS (`.dylib`) without
modification.

### macOS

```sh
clang -o myapp myapp.c -I./include -L./target/release -lalif
```

Set `DYLD_LIBRARY_PATH` if the library is not in a standard location:

```sh
DYLD_LIBRARY_PATH=./target/release ./myapp proof.alif
```

### Windows

On Windows, `cargo build --release` produces `target\release\alif.dll` and an
import library `target\release\alif.dll.lib`.  Link against the import library:

```bat
cl myapp.c /I include /link alif.dll.lib /LIBPATH:target\release
```

Place `alif.dll` in the same directory as the executable or in a directory on
`PATH`.

---

## Complete C example

The program below reads an Alif source file from the command line, calls
`alif_verify`, and prints a human-readable result.

```c
/*
 * verify.c — minimal Alif proof verifier front-end in C
 *
 * Build (Linux):
 *   gcc -o verify verify.c -I./include -L./target/release -lalif -lpthread -ldl
 *
 * Run:
 *   LD_LIBRARY_PATH=./target/release ./verify examples/and_comm.alif
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "alif.h"

/*
 * Read the entire contents of `path` into a heap-allocated, null-terminated
 * buffer.  The caller is responsible for freeing the buffer.
 * Returns NULL on error.
 */
static char *read_file(const char *path) {
    FILE *fp = fopen(path, "rb");
    if (!fp) {
        perror("fopen");
        return NULL;
    }

    if (fseek(fp, 0, SEEK_END) != 0) {
        perror("fseek");
        fclose(fp);
        return NULL;
    }

    long size = ftell(fp);
    if (size < 0) {
        perror("ftell");
        fclose(fp);
        return NULL;
    }
    rewind(fp);

    char *buf = malloc((size_t)size + 1);
    if (!buf) {
        fprintf(stderr, "out of memory\n");
        fclose(fp);
        return NULL;
    }

    size_t read = fread(buf, 1, (size_t)size, fp);
    fclose(fp);

    if ((long)read != size) {
        fprintf(stderr, "read error: expected %ld bytes, got %zu\n", size, read);
        free(buf);
        return NULL;
    }

    buf[size] = '\0';
    return buf;
}

int main(int argc, char *argv[]) {
    if (argc != 2) {
        fprintf(stderr, "usage: %s <file.alif>\n", argv[0]);
        return 2;
    }

    char *source = read_file(argv[1]);
    if (!source) {
        return 2;
    }

    int result = alif_verify(source);
    free(source);

    switch (result) {
    case 0:
        printf("✓ QED\n");
        return 0;
    case 1:
        fprintf(stderr, "proof error: one or more theorems are invalid\n");
        fprintf(stderr, "hint: use the Rust API for a detailed error message\n");
        return 1;
    case 2:
        fprintf(stderr, "parse error: source could not be parsed\n");
        return 2;
    default:
        fprintf(stderr, "unexpected return code %d\n", result);
        return 2;
    }
}
```

---

## Python bindings example

Python's `ctypes` module can load `libalif.so` without any additional tooling.

```python
#!/usr/bin/env python3
"""
alif_verify.py — call libalif from Python via ctypes

Usage:
    python3 alif_verify.py examples/and_comm.alif
"""

import ctypes
import sys
import os

# Adjust the path to wherever libalif.so lives.
LIB_PATH = os.path.join(
    os.path.dirname(__file__), "target", "release", "libalif.so"
)

def load_library(path: str) -> ctypes.CDLL:
    lib = ctypes.CDLL(path)
    # Declare the signature so ctypes enforces it.
    lib.alif_verify.argtypes = [ctypes.c_char_p]
    lib.alif_verify.restype  = ctypes.c_int
    return lib

def verify_file(path: str) -> int:
    """Return 0 (ok), 1 (proof error), or 2 (parse error / io error)."""
    try:
        with open(path, "rb") as fh:
            source_bytes = fh.read()
    except OSError as exc:
        print(f"error reading {path!r}: {exc}", file=sys.stderr)
        return 2

    # source_bytes is already bytes; ctypes c_char_p accepts bytes objects and
    # passes a null-terminated pointer to the underlying C function.
    lib = load_library(LIB_PATH)
    return lib.alif_verify(source_bytes)

def main():
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <file.alif>", file=sys.stderr)
        sys.exit(2)

    path = sys.argv[1]
    code = verify_file(path)

    if code == 0:
        print("✓ QED")
    elif code == 1:
        print("proof error", file=sys.stderr)
        sys.exit(1)
    else:
        print("parse error or invalid input", file=sys.stderr)
        sys.exit(2)

if __name__ == "__main__":
    main()
```

**Notes on the Python binding:**

- `ctypes.c_char_p` automatically appends a null terminator when the value is a
  `bytes` object, which satisfies the FFI contract.
- Opening the file in binary mode (`"rb"`) and reading raw bytes ensures no
  codec transformation occurs before the bytes reach `alif_verify`.
- For production use, consider wrapping the library loading in a module-level
  singleton so the shared library is loaded only once per process.

---

## Error handling

`alif_verify` returns only three distinct codes.  This coarse interface is
intentional: the FFI is designed for simple go/no-go checks from host languages
that cannot represent Rust enums.

If you need structured error information — the step index, the text of the
failing step, the parse offset — you have two options:

1. **Use the Rust API directly.**  Link against the `rlib` output, or embed
   `alif` as a Rust dependency, and call `verify_source` or the lower-level
   `parse_source` / `check` functions.

2. **Extend the FFI.**  You can add additional exported functions to `src/lib.rs`
   that write error details into caller-supplied buffers.  For example, a
   `alif_verify_with_message(source, buf, buf_len)` function that writes a UTF-8
   error string into `buf`.  Refer to the Rust API reference for the relevant
   types.

---

## Symbol visibility

The `#[no_mangle]` attribute on `alif_verify` preserves the exact symbol name in
the shared library.  You can verify it is exported with:

```sh
nm -D target/release/libalif.so | grep alif_verify
```

Expected output:

```
0000000000001234 T alif_verify
```

All other Rust symbols are mangled and are not part of the stable ABI.
