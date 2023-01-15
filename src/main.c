#include <stdio.h>
#include "ast.h"
#include "util.h"

char * read_file(char * name) {
    FILE *f = fopen(name, "rb");
    fseek(f, 0, SEEK_END);
    long fsize = ftell(f);
    fseek(f, 0, SEEK_SET);  /* same as rewind(f); */

    char *string = malloc(fsize + 1);
    fread(string, fsize, 1, f);
    fclose(f);

    string[fsize] = 0;
    return string;
}

int main() {
    struct ena_tok_list list = ena_create_tok_list(2);
    char * file_content = read_file("test.ena");
    ena_print_tok_err(ena_build_tok_list(&list, file_content));
    struct ena_ast_node program = ena_new_root_ast_node(20);
    ena_build_ast(&program, &list);
    ena_debug_ast(program);
    ena_free_tok_list(&list, true);
    ena_free_ast_node(program);
    free(file_content);
}