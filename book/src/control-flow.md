# Chapter 3 - Control Flow

Control flow in Ena is managed through a combination of keywords, stack manipulation, and block calls. This chapter explores the various control flow mechanisms available in the language.

## Conditional Execution with `if`

The `if` keyword executes a block when the top value on the stack is `true`.

```ena
check_positive {
    n ( unit )
    n =
    
    0 n @ > if {  # a b > tests b > a, so 0 n @ > tests n > 0
        "Number is positive" println
        return
    }
    
    "Number is not positive" println
}

main {
    5 check_positive
    -3 check_positive
}
```

**Important:** The `if` keyword pops the boolean value from the stack before executing the block.

## Loops with `while`

The `while` keyword repeatedly executes a block as long as the condition evaluates to `true`. The block must push a boolean value onto the stack at the end of each iteration.

A common pattern is to use `true while` with conditional returns:

```ena
count_to_five {
    i ( unit )
    0 i =
    
    true while {
        i @ 5 == if {
            return
        }
        i @ print " " print
        i @ 1 + i =
        true
    }
}

main {
    count_to_five
}
```

**Output:** `0 1 2 3 4`

### Loop Control

Ena provides two special operations for loop control:

- **`continue`**: Skips to the next iteration (implemented as `true return_local`)
- **`break`**: Exits the loop (implemented as `false return_local`)

```ena
skip_threes {
    i ( unit )
    0 i =
    
    true while {
        i @ 10 == if {
            return
        }
        
        # Skip value 3
        i @ 3 == if {
            i @ 1 + i =
            true
            continue
        }
        
        i @ print " " print
        i @ 1 + i =
        true
    }
}
```

**Output:** `0 1 2 4 5 6 7 8 9`

## Early Returns

### Global Return with `return`

The `return` keyword exits the current global block entirely:

```ena
find_value {
    i ( unit )
    0 i =
    
    true while {
        i @ 100 == if {
            "Not found\n" print
            return
        }
        
        i @ 42 == if {
            "Found 42!\n" print
            return
        }
        
        i @ 1 + i =
        true
    }
}
```

### Local Return with `return_local`

The `return_local` keyword exits only the innermost local block (enclosed in parentheses):

```ena
example {
    value ( 
        condition if {
            special_value return_local
        }
        default_value
    )
    
    value @ println
}
```

## Pattern: Condition Blocks

A common pattern in Ena is to evaluate conditions in your loop body:

```ena
process_items {
    i ( unit )
    0 i =
    
    true while {
        i @ 5 == if {
            return
        }
        
        i @ print " " print
        i @ 1 + i =
        true
    }
}
```

## Recursion

Ena supports recursive function calls. Here's the factorial example from the standard examples:

```ena
factorial_inner {
    dup 1 == if {
        1 return
    }
    
    dup 1 swap - factorial_inner *
}

factorial {
    factorial_inner *
}

main {
    5 factorial println  # Outputs: 120
}
```

## Short-Circuit Evaluation

Ena does not have built-in short-circuit evaluation for boolean operations. You must implement it manually using `if` blocks:

```ena
safe_check {
    value ( unit )
    value =
    
    # Check if value is not null first
    value @ null == if {
        false return
    }
    
    # Now safe to perform additional checks
    value @ some_property_check
}
```

## Comparison Operators

Ena provides several comparison operators that work on the stack:

- **`==`**: Equal to
- **`>`**: Greater than
- **`<`**: Less than  
- **`>=`**: Greater than or equal to
- **`<=`**: Less than or equal to

```ena
comparisons {
    # Operators work in reverse: a b > tests if b > a
    3 5 >   # Tests if 5 > 3: true
    5 3 >   # Tests if 3 > 5: false
    3 5 <   # Tests if 5 < 3: false
    5 3 <   # Tests if 3 < 5: true
    5 5 ==  # Tests if 5 == 5: true
}
```

**Important:** Comparison operators work in reverse of typical expectations. `a b >` tests if `b > a`, not `a > b`. This is due to how values are popped from the stack.

## Boolean Operations

- **`!`**: Logical NOT
- **`and`**: Logical AND
- **`or`**: Logical OR

```ena
boolean_example {
    val1 ( unit )
    val2 ( unit )
    
    true val1 =
    false val2 =
    
    result ( unit )
    val1 @ val2 @ or result =
    
    result @ if {
        "At least one is true" println
    }
}
```

## Exception Handling

Ena supports exception handling through the `try` keyword:

```ena
will_fail {
    drop  # Fails because stack is empty
}

main {
    'will_fail try
    "Error caught: " print
    ena.vm.debug
}
```

When an exception occurs, it is placed on the stack and can be inspected.

## Best Practices

1. **Always push condition values**: When using `while`, ensure the condition boolean is pushed at the end of each iteration.

2. **Use early returns**: Simplify complex logic with early `return` statements.

3. **Minimize nesting**: Prefer early returns over deeply nested `if` blocks.

4. **Stack order matters**: Remember that operators pop values in reverse order due to stack-based execution.

5. **Document complex control flow**: Use comments to explain non-obvious control flow patterns.
