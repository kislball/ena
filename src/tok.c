#include "ast.h"
#include "util.h"
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <ctype.h>
#include <stdio.h>

double ena_token_number_value(struct ena_token token) {
  return * (double *) token.data;
}

char * ena_token_char_value(struct ena_token token) {
  return (char *) token.data;
}

bool ena_is_token_value_numeric(struct ena_token token) {
  return token.type == ena_token_double;
}

bool ena_is_token_value_string(struct ena_token token) {
  return token.type == ena_token_identifier
    || token.type == ena_token_string
    || token.type == ena_token_escaped_identifier;
}

bool ena_is_token_value_nil(struct ena_token token) {
  return token.type == ena_token_open || token.type == ena_token_close;
}  

bool ena_is_token_null(struct ena_token token) {
  return token.type == ena_token_null;
}

struct ena_token ena_new_token(enum ena_token_type type, void * data, size_t col, size_t line, size_t abs) {
  const struct ena_token token = {.type = type, .data = data, .col = col, .line = line, .abs = abs};
  return token;
}

void ena_free_token_value(struct ena_token token) {
  free(token.data);
}

struct ena_tok_list ena_create_tok_list(size_t prealloc) {
  struct ena_tok_list list = {prealloc, 0, malloc(prealloc * sizeof(struct ena_token))};

  return list;
}

void ena_free_tok_list(struct ena_tok_list * list, bool do_free_tokens) {
  if (do_free_tokens) {
    for (size_t i = 0; i < list->size; i++) {
      ena_free_token_value(ena_token_at(list, i));
    }
  }
  list->allocated = 0;
  list->size = 0;
  free(list->begin);
}

void ena_add_tok_to_list(struct ena_tok_list * list, struct ena_token token) {
  if (list->size == list->allocated) {
    ena_reallocate_tok_list(list, list->allocated * 2);
  }
  *(list->begin + list->size) = token;
  list->size++;
}

void ena_reallocate_tok_list(struct ena_tok_list * list, size_t new) {
  list->allocated = new;
  struct ena_token * ne = realloc(list->begin, list->allocated * sizeof(struct ena_token));
  if (ne == NULL) {
    ena_errf("ena_reallocate_tok_list: failed to reallocate token list\n");
    exit(-1);
  }

  list->begin = ne;
}

struct ena_token ena_token_at(struct ena_tok_list * list, size_t index) {
  if (index >= list->size) {
    return ena_tok_null;
  }
  return *(list->begin + index);
}

bool __ena_is_id(char ch) {
  return isalpha((int)ch) != 0
    || ch == '.'
    || ch == '?'
    || ch == '-'
    || ch == '_'
    || ch == '+'
    || ch == '/'
    || ch == '*';
}

size_t ena_parse_id(size_t line, size_t col, size_t len, size_t i, char * code, char ch, struct ena_tok_list * list, bool escaped) {
  char * id = ena_alloc_str();

  while (i < len) {
    ch = code[i];

    if (__ena_is_id(ch)) {
      ena_append_char(id, ch);
    } else {
      break;
    }

    i++;
  }

  struct ena_token token = ena_new_token(escaped ? ena_token_escaped_identifier : ena_token_identifier, id, col, line, i);
  ena_add_tok_to_list(list, token);

  return i;
}

struct ena_tok_err ena_build_tok_list(struct ena_tok_list * list, char * code) {
  size_t len = strlen(code);
  struct ena_tok_err result = {};
  result.col = 0;
  result.line = 1;
  result.code = ena_tok_err_none;

  #define return_with(err_code) do { \
    result.code = err_code; \
    result.abs = i; \
    return result; \
  } while (0)

  for (size_t i = 0; i < len; ++i) {
    char ch = code[i];

    if (ch == '\n') {
      result.col = 0;
      result.line++;
      continue;
    } else {
      result.col++;
    }

    if (ch == '{') {
      ena_add_tok_to_list(list, ena_new_token(ena_token_open, NULL, result.col, result.line, result.abs));
      continue;
    }

    if (ch == '}') {
      ena_add_tok_to_list(list, ena_new_token(ena_token_close, NULL, result.col, result.line, result.abs));
      continue;
    }

    if (__ena_is_id(ch)) {
      size_t old_i = i;
      i = ena_parse_id(result.line, result.col, len, i, code, ch, list, false);
      result.col += i - old_i;
      continue;
    }

    if (ch == '"') {
      char * str = ena_alloc_str();
      i++;
      ch = code[i];

      while (true) {
        ch = code[i];

        if (ch == '\\') {
          char next = code[i + 1];
          if (next == 'n') {
            ena_append_char(str, '\n');
          } else if (next == 'r') {
            ena_append_char(str, '\r');
          } else if (next == '"') {
            ena_append_char(str, '"');
          } else if (next == 'a') {
            ena_append_char(str, '\a');
          } else if (next == 'e') {
            ena_append_char(str, '\e');
          } else if (next == 'f') {
            ena_append_char(str, '\f');
          } else if (next == 't') {
            ena_append_char(str, '\t');
          } else if (next == 'v') {
            ena_append_char(str, '\v');
          } else if (next == '\\') {
            ena_append_char(str, '\\');
          } else {
            return_with(ena_tok_err_unknown_escape_sequence);
          }

          // todo: add unicode support

          i += 2;
          continue;
        }

        if (ch == '"') {
          break;
        }

        ena_append_char(str, ch);

        i++;
      }

      ena_add_tok_to_list(list, ena_new_token(ena_token_string, str, result.col, result.line, i));
      continue;
    }

    if (ch == '\'') {
      i++;
      size_t old_i = i;
      i = ena_parse_id(result.line, result.col, len, i, code, ch, list, true);
      result.col += i - old_i;
      continue;
    }

    if (isspace(ch) != 0) {
      continue;
    }

    if (isdigit(ch) != 0) {
      char * number = ena_alloc_str();
      bool had_dot = false;
      size_t begin_col = result.col;
      size_t begin_line = result.line;
      size_t begin_abs = i;
            
      while (i < len) {
        ch = code[i];
        if (ch == '.') {
          if (had_dot) {
            return_with(ena_tok_err_unexpected_numeric_point);
          } else {
            had_dot = true;
            ena_append_char(number, '.');    
          }
        } else if (isdigit(ch) != 0) {
          ena_append_char(number, ch);
        } else {
          break;
        }
        i++;
        result.col++;
      }
      double num = strtod(number, NULL);
      double * pnum = malloc(sizeof(num));
      *pnum = num;
      struct ena_token token = ena_new_token(ena_token_double, pnum, begin_col, begin_line, begin_abs);      
      ena_add_tok_to_list(list, token);
      free(number);
      continue;
    }

    return_with(ena_tok_err_unexpected_token);
  }

  #undef return_with

  return result;
}

void ena_debug_token(struct ena_token token) {
  if (ena_is_token_value_numeric(token)) {
    printf("(%lu:%lu)   DOUBLE:%.10f(%p)\n", token.line, token.col, ena_token_number_value(token), token.data);
  } else if (ena_is_token_value_string(token)) {
    printf("(%lu:%lu)   %s:\"%s\"(%p)\n", token.line, token.col, ena_stringify_tok_type(token.type), ena_token_char_value(token), token.data);
  } else if (ena_is_token_value_nil(token)) {
    printf("(%lu:%lu)   NILV:%d\n", token.line, token.col, token.type);
  } else {
    printf("NULL:NULL\n");
  }
}

void ena_debug_tok_list(struct ena_tok_list * list) {
  if (list->size == 0) {
    printf("ena_debug_tok_list: tok list is empty\n");
  }
  for (size_t i = 0; i < list->size; ++i) {
    ena_debug_token(ena_token_at(list, i));
  }
}

char * ena_stringify_type(enum ena_tok_err_type type) {
  switch (type) {
    case ena_tok_err_none: return "ok";
    case ena_tok_err_unexpected_token: return "unexpected token";
    case ena_tok_err_unexpected_numeric_point: return "unexpected numeric point";
    case ena_tok_err_unknown_escape_sequence: return "unknown escape sequence";
  }
}

void ena_print_tok_err(struct ena_tok_err err) {
  printf("%s at %lu:%lu(%lu)\n", ena_stringify_type(err.code), err.line, err.col, err.abs + 1);
}

char * ena_stringify_tok_type(enum ena_token_type type) {
  switch (type) {
    case ena_token_double: return "DOUBLE";
    case ena_token_identifier: return "IDENTIFIER";
    case ena_token_string: return "STRING";
    case ena_token_escaped_identifier: return "ESCAPED_IDENTIFIER";
    case ena_token_open: return "OPEN";
    case ena_token_close: return "CLOSE";
    case ena_token_null: return "NULL";
  }
}