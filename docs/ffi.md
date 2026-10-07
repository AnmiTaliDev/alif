# C API

The crate builds a shared library, `libalif.so` on Linux and the BSDs, `libalif.dylib` on macOS and `alif.dll` on Windows. It exports three C functions, declared in `include/alif.h`.

```c
int alif_verify(const char *source);
int alif_verify_file(const char *path);
const char *alif_last_error(void);
```

## Functions

### `alif_verify`

Verifies a source text. The argument is a null-terminated UTF-8 string. It behaves like `verify_source`: imports are rejected.

### `alif_verify_file`

Verifies the file at the given path. The argument is a null-terminated UTF-8 string. It behaves like `verify_file`: imports are resolved relative to the directory of the file.

### Return values

| Value | Meaning |
|-------|---------|
| 0 | Everything was verified. |
| 1 | A proof error was found. |
| 2 | A syntax error, a load error, a null pointer or text that is not valid UTF-8. |

### `alif_last_error`

Returns the message of the last failed call made on the calling thread, as a null-terminated string. Returns `NULL` when the last call on the thread succeeded or when no call has been made.

- The message has the format described in [errors.md](errors.md).
- The pointer stays valid until the next `alif_verify` or `alif_verify_file` call on the same thread. Copy the string if it has to live longer.
- The caller must not free or modify the string.
- Each thread has its own message. The functions can be called from several threads at the same time.

## Preconditions

The pointer arguments must be null or point to a null-terminated string that stays valid for the duration of the call. A null pointer is reported with status 2 and the message `null pointer argument`. Text that is not valid UTF-8 is reported with status 2 and the message `argument is not valid UTF-8`.

On macOS and Windows each call to `alif_verify` or `alif_verify_file` prints the platform warning to standard error.

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

Build the library and compile the example on Linux:

```sh
cargo build --release
gcc -Iinclude -o ffi_demo examples/ffi_demo.c \
    -Ltarget/release -lalif -Wl,-rpath,"$PWD/target/release"
./ffi_demo
```

The program prints `verified`.

## Use from C++

`alif.h` wraps the declarations in `extern "C"` when it is included from C++.
