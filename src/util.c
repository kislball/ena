#include "util.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdarg.h>

char * ena_append_string_safe(char * s1, char * s2) {
  char *result = malloc(strlen(s1) + strlen(s2) + 1);
  if (result == NULL) {
    ena_errf("ena_append_string_safe: failed to allocate additional memory\n");
    exit(-1);
  }    
  strcpy(result, s1);
  strcat(result, s2);
  return result;
}

char * ena_append_char_safe(char * s1, char s2) {
  size_t len = strlen(s1);
  char * str2 = malloc(len + 2);
  strcpy(str2, s1);
  str2[len] = s2;
  str2[len + 1] = '\0';
  return str2;
}

char * ena_alloc_str() {
  char * p = malloc(1);
  p[0] = '\0';
  return p;
}

void ena_errf(const char * template, ...) {
  va_list args;
  va_start(args, template);

  vfprintf(stderr, template, args);

  va_end(args);
}