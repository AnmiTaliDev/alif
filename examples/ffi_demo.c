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
