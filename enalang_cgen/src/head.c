// This file contains some basic ena runtime for C.
// Used for enalang_cgen.

#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>

#define STACK_PREALLOC 30

enum ena_value_type {
    ena_number,
    ena_string,
    ena_bool,
    ena_pointer,
    ena_block,
    ena_exception,
    ena_atom,
    ena_null
};

union ena_value_inner {
    double number;
    char * string;
    bool boolean;
    size_t pointer;
    struct ena_value * exception;
    char * atom;
    void * null;
};

struct ena_value {
    enum ena_value_type type;
    union ena_value_inner value;
};

struct ena_value * stack_begin = NULL;
size_t stack_capacity = STACK_PREALLOC;
size_t stack_size = 0;

void init_stack() {
    stack_begin = malloc(STACK_PREALLOC * sizeof(struct ena_value));
}

void realloc_stack() {
    stack_capacity = stack_capacity * 2;
    struct ena_value * new_stack = realloc(stack_begin, stack_capacity * sizeof(struct ena_value));
    if (new_stack == NULL) {
        fprintf(stderr, "failed to allocate memory for stack expansion");
        exit(1);
    }
    stack_begin = new_stack;
}

void free_stack() {
    free(stack_begin);
}

void ena_run();

int main(void) {
    init_stack();

    ena_run();

    free_stack();
    return 0;
}
