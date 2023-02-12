// This file contains some basic ena runtime for C.
// Used for enalang_cgen.

#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <math.h>

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
    struct ena_value * pointer;
    struct ena_value * exception;
    char * atom;
    void * null;
};

struct ena_value {
    enum ena_value_type type;
    union ena_value_inner value;
};

struct ena_value null = {
    .type = ena_null,
    .value = {
        .null = NULL,
    },
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

void expected_double() {
    fprintf(stderr, "expected number");
    exit(1);
}

void expected_int() {
    fprintf(stderr, "expected int");
    exit(1);
}

void push_stack(struct ena_value val) {
    if (stack_size >= stack_capacity) {
        realloc_stack();
    }
    stack_size++;
    stack_begin[stack_size - 1] = val;
}


void free_value(struct ena_value val) {
    switch (val.type) {
        case ena_string:
        case ena_pointer:
        case ena_atom:
        case ena_exception:
            free(val.value.atom);
        default:
            break;
    }
}

struct ena_value pop_stack() {
    if (stack_size == 0) {
        fprintf(stderr, "stack ended");
        exit(1);
    }
    stack_size -= 1;
    struct ena_value val = stack_begin[stack_size];
    return val;
}

void free_stack() {
    for (size_t i = 0; i < stack_size; ++i) {
        struct ena_value val = stack_begin[i];
        free_value(val);
    }
    free(stack_begin);
}

void ena_run();

void alloc() {
    struct ena_value val = pop_stack();
    if (val.type != ena_number) {
        expected_double();
    }

    if (val.value.number != roundf(val.value.number)) {
        expected_int();
    }

    struct ena_value * new_mem = malloc(sizeof(struct ena_value) * val.value.number);
    if (new_mem == NULL) {
        fprintf(stderr, "failed to allocate memory for ena heap");
        exit(1);
    }
    
    push_stack(val);
}

int main(void) {
    init_stack();

    ena_run();

    free_stack();
    return 0;
}
