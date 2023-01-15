#include "ast.h"
#include <stdbool.h>
#include <stdlib.h>
#include "util.h"
#include <string.h>
#include <stdio.h>

bool ena_is_host_node(struct ena_ast_node node) {
  return node.type == ena_ast_node_program || node.type == ena_ast_node_block;
}

void ena_free_ast_node(struct ena_ast_node node) {
  if (ena_is_host_node(node)) {
    for (size_t i = 0; i < node.content_size; ++i) {
      ena_free_ast_node(node.content->node[i]);
    }
    free(node.content->node);
  }

  if (node.type == ena_ast_node_string) {
    free(node.content->string);
  }
}

void ena_append_ast_node_to_node(struct ena_ast_node * dest, struct ena_ast_node src) {
  dest->content_size++;
  struct ena_ast_node * new = realloc(dest->content->node, dest->content_size * sizeof(struct ena_ast_node));
  if (new == NULL) {
    ena_errf("ena_append_ast_node_to_node: failed to allocate memory");
    exit(-1);
  }

  printf("%p -> %p\n", dest->content->node, new);
  dest->content->node = new;
  dest->content->node[dest->content_size - 1] = src;
}

struct ena_ast_node ena_new_ast_node_host(enum ena_ast_node_type type, size_t prealloc) {
  union ena_ast_node_content content;
  content.node = NULL;
  struct ena_ast_node node = {
    .content_size = 0,
    .type = type,
    .content = &content,
  };

  return node;
}

struct ena_ast_node ena_new_ast_node_number(double num) {
  union ena_ast_node_content content;
  content.number = num;
  struct ena_ast_node node = {
    .type = ena_ast_node_number,
    .content = &content,
    .content_size = 0,
  };

  return node;
}

struct ena_ast_node ena_new_ast_node_string(enum ena_ast_node_type type, char * str) {
  union ena_ast_node_content content;
  content.string = str;
  struct ena_ast_node node = {
    .type = type,
    .content = &content,
    .content_size = 0,
  };

  return node;
}

struct ena_ast_node ena_new_ast_node_keyword(enum ena_ast_node_keyword keyword) {
  union ena_ast_node_content content;
  content.keyword = keyword;
  struct ena_ast_node node = {
    .type = ena_ast_node_keyword,
    .content = &content,
    .content_size = 0, 
  };

  return node;
}

struct ena_ast_node ena_new_root_ast_node(size_t prealloc) {
  return ena_new_ast_node_host(ena_ast_node_program, prealloc);
}

enum ena_ast_node_keyword ena_to_keyword(char * str) {
  if (strcmp(str, "if") == 0) {
    return ena_keyword_if;
  } else if (strcmp(str, "unless")) {
    return ena_keyword_unless;
  } else if (strcmp(str, "while")) {
    return ena_keyword_while;
  } else {
    return ena_keyword_unknown;
  }
}

struct ena_ast_err ena_walk(struct ena_ast_node * node, struct ena_tok_list * list, size_t * at) {
  struct ena_ast_err result = {};
  size_t i = *at;

  for (; i < list->size; (i)++) {
    struct ena_token token = ena_token_at(list, i);

    if (ena_is_token_value_string(token)) {
      char * sval = ena_token_char_value(token);
      if (token.type == ena_token_identifier) {
        enum ena_ast_node_keyword keyword = ena_to_keyword(sval);
        if (keyword != ena_keyword_unknown) {
          ena_append_ast_node_to_node(node, ena_new_ast_node_keyword(keyword));  
        } else {
          ena_append_ast_node_to_node(node, ena_new_ast_node_string(ena_ast_node_identifier, sval));
        }
      } else if (token.type == ena_token_escaped_identifier) {        
        ena_append_ast_node_to_node(node, ena_new_ast_node_string(ena_ast_node_escaped_identifier, sval));
      } else if (token.type == ena_token_string) {
        ena_append_ast_node_to_node(node, ena_new_ast_node_string(ena_ast_node_string, sval));
      }
      continue;
    } else if (token.type == ena_token_double) {
      ena_append_ast_node_to_node(node, ena_new_ast_node_number(ena_token_number_value(token)));
    } else {
      result.type = ena_parse_err_unexpected_token;
    }
  }

  *at = i;
  return result;
}

struct ena_ast_err ena_build_ast(struct ena_ast_node * node, struct ena_tok_list * list) {
  size_t i = 0;
  return ena_walk(node, list, &i);
}

char * __repeat_tab(size_t amount) {
  char * alloc = malloc(amount + 1);
  for (size_t i = 0; i < amount - 1; i++)
    alloc[i] = '\t';
  alloc[amount - 1] = '\0';

  return alloc;
}

void __ena_debug_ast(struct ena_ast_node node, size_t indent_level) {
  if (ena_is_host_node(node)) {
    for (size_t i = 0; i < node.content_size; i++) {
      __ena_debug_ast(*node.content[i].node, indent_level + 1);
    }
  } else {
    printf("%s%s\n", indent_level == 0 ? "" : __repeat_tab(indent_level), ena_stringify_ast_node_type(node.type));
  }
}

void ena_debug_ast(struct ena_ast_node node) {
  __ena_debug_ast(node, 0);
}

char * ena_stringify_ast_node_type(enum ena_ast_node_type type) {
  switch (type) {
    case ena_ast_node_program: return "PROGRAM";
    case ena_ast_node_number: return "NUMBER";
    case ena_ast_node_identifier: return "IDENTIFIER";
    case ena_ast_node_keyword: return "KEYWORD";
    case ena_ast_node_escaped_identifier: return "ESCAPED_IDENTIFIER";
    case ena_ast_node_string: return "STRING";
    case ena_ast_node_block: return "BLOCK";
    default: return "UNKNOWN";
  }
}