# C interface

The shared library exports three functions. They are declared in `include/alif.h`.

```c
int alif_verify(const char *source);
int alif_verify_file(const char *path);
const char *alif_last_error(void);
```

The library file is `libalif.so` on Linux and the BSDs, `libalif.dylib` on macOS and `alif.dll` on Windows. The header is usable from C++.

## alif_verify

Verifies source text. The argument is a null-terminated UTF-8 string. Behaves like `verify_source`, so `import` is an error.

## alif_verify_file

Verifies a file. The argument is a null-terminated UTF-8 path. Behaves like `verify_file`, so `import` paths are relative to the directory of the file.

## Return value

Both functions return:

| Value | Meaning |
|-------|---------|
| 0 | verified |
| 1 | proof error |
| 2 | syntax error, load error, null pointer or invalid UTF-8 |

## alif_last_error

Returns the diagnostic of the most recent failed call on the calling thread, or `NULL` if the most recent call succeeded or no call was made.

- The text has the format described in [errors.md](errors.md).
- The pointer is valid until the next `alif_verify` or `alif_verify_file` call on the same thread. Copy the text to keep it.
- The caller does not free or modify it.
- The message is kept per thread, so the functions can be used from several threads at once.

A null argument produces status 2 and the message `null pointer argument`. Invalid UTF-8 produces status 2 and the message `argument is not valid UTF-8`.

On macOS and Windows each call also writes the platform warning to standard error.

## Example

`examples/ffi_demo.c`:

```c
#include <stdio.h>

#include "alif.h"

int main(void)
{
    const char *source =
        "theorem t: A |- A\n"
        "proof\n"
        "  assume h: A\n"
        "  exact h\n"
        "qed\n";

    int status = alif_verify(source);
    if (status != 0) {
        fprintf(stderr, "%s\n", alif_last_error());
        return status;
    }
    puts("verified");
    return 0;
}
```

Build and run on Linux:

```sh
cargo build --release
gcc -Iinclude -o ffi_demo examples/ffi_demo.c \
    -Ltarget/release -lalif -Wl,-rpath,"$PWD/target/release"
./ffi_demo
```

The program prints `verified`.
