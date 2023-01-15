#include <stdbool.h>

#ifndef ENA_UTIL_H
#define ENA_UTIL_H

char * ena_append_string_safe(char * dest, char * orig);
char * ena_append_char_safe(char *, char);
char * ena_alloc_str();
void ena_errf(const char * msg, ...);

#define ena_append_char(str, ch) do {\
char * old = str; \
str = ena_append_char_safe(str, ch); \
free(old); \
} while (0);
#endif