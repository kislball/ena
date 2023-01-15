#include <stdbool.h>
#include <stdlib.h>

#ifndef ENA_AST_H
#define ENA_AST_H

#define ena_tok_null ena_new_token(ena_token_null, NULL, 0, 0, 0)

enum ena_tok_err_type {
  ena_tok_err_none,
  ena_tok_err_unexpected_token,
  ena_tok_err_unexpected_numeric_point,
  ena_tok_err_unknown_escape_sequence,
};

enum ena_token_type {
   ena_token_double,
   ena_token_identifier,
   ena_token_string,
   ena_token_escaped_identifier,
   ena_token_open,
   ena_token_close,
   ena_token_null,
};

struct ena_token {
  void * data;
  enum ena_token_type type;
  size_t col, line, abs;
};

double ena_token_number_value(struct ena_token);
char * ena_token_char_value(struct ena_token);
bool ena_is_token_value_numeric(struct ena_token);
bool ena_is_token_value_string(struct ena_token);
bool ena_is_token_value_nil(struct ena_token);
bool ena_is_token_null(struct ena_token);
struct ena_token ena_new_token(enum ena_token_type, void *, size_t, size_t, size_t);
void ena_free_token_value(struct ena_token);
char * ena_stringify_tok_type(enum ena_token_type);

struct ena_tok_list {
  size_t allocated;
  size_t size;
  struct ena_token * begin;
};

struct ena_tok_list ena_create_tok_list(size_t);
void ena_free_tok_list(struct ena_tok_list *, bool);
void ena_add_tok_to_list(struct ena_tok_list *, struct ena_token);
void ena_reallocate_tok_list(struct ena_tok_list *, size_t);
struct ena_token ena_token_at(struct ena_tok_list *, size_t);
void ena_debug_tok_list(struct ena_tok_list * list);

struct ena_tok_err ena_build_tok_list(struct ena_tok_list *, char *);

struct ena_tok_err {
  enum ena_tok_err_type code;
  size_t col, line, abs;
};

void ena_debug_token(struct ena_token);
void ena_print_tok_err(struct ena_tok_err);
char * ena_stringify_type(enum ena_tok_err_type);

enum ena_ast_node_type {
  ena_ast_node_program,
  ena_ast_node_number,
  ena_ast_node_identifier,
  ena_ast_node_keyword,
  ena_ast_node_escaped_identifier,
  ena_ast_node_string,
  ena_ast_node_block
};

enum ena_ast_err_type {
  ena_parse_err_none,
  ena_parse_err_unexpected_token,
};

enum ena_ast_node_keyword {
  ena_keyword_if,
  ena_keyword_unless,
  ena_keyword_while,
  ena_keyword_unknown,
};

struct ena_ast_node {
  size_t content_size;
  union ena_ast_node_content * content;
  enum ena_ast_node_type type;
};

union ena_ast_node_content {
  struct ena_ast_node * node;
  enum ena_ast_node_keyword keyword;
  double number;
  char * string;
};

bool ena_is_host_node(struct ena_ast_node);
void ena_free_ast_node(struct ena_ast_node);
void ena_append_ast_node_to_node(struct ena_ast_node *, struct ena_ast_node);
struct ena_ast_node ena_new_ast_node_host(enum ena_ast_node_type, size_t prealloc);
struct ena_ast_node ena_new_ast_node_number(double);
struct ena_ast_node ena_new_ast_node_string(enum ena_ast_node_type, char *);
struct ena_ast_node ena_new_ast_node_keyword(enum ena_ast_node_keyword);
struct ena_ast_node ena_new_root_ast_node(size_t prealloc);

enum ena_ast_node_keyword ena_to_keyword(char *);

struct ena_ast_err ena_build_ast(struct ena_ast_node *, struct ena_tok_list *);
void ena_debug_ast(struct ena_ast_node);
char * ena_stringify_ast_node_type(enum ena_ast_node_type);

struct ena_ast_err {
  enum ena_ast_err_type type;
  size_t line, col, abs;
};
#endif // ENA_AST_H