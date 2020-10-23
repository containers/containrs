#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Calculate the number of bytes in the last error's error message including a trailing `null`
 * character. If there are no recent error, then this returns `0`.
 */
int last_error_length(void);

/**
 * Write the most recent error message into a caller-provided buffer as a UTF-8
 * string, returning the number of bytes written.
 *
 * # Note
 *
 * This writes a **UTF-8** string into the buffer. Windows users may need to
 * convert it to a UTF-16 "unicode" afterwards.
 *
 * If there are no recent errors then this returns `0` (because we wrote 0
 * bytes). `-1` is returned if there are argument based errors, for example
 * when passed a `null` pointer or a buffer of insufficient size.
 */
int last_error_message(char *buffer, int length);
