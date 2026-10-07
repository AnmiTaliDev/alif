/* SPDX-License-Identifier: GPL-3.0-only */

#ifndef ALIF_H
#define ALIF_H

#ifdef __cplusplus
extern "C" {
#endif

int alif_verify(const char *source);
int alif_verify_file(const char *path);
const char *alif_last_error(void);

#ifdef __cplusplus
}
#endif

#endif
