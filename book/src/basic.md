# Chapter 2 - Basic Ena syntax

## Literals

As stated in the first chapter, all literals in Ena code put themselves on top of stack.

```ena
about_literals {
  "Hello, world!" # strings
  42069 # numbers
  true false # boolean
  null # null value
  'about_literals # puts block, making it similar to a closure
  :atom # values which only equal to themselves
}
```

> Warning: you should NEVER put local blocks on the stack. This is considered unsafe and WILL result in runtime errors.

## Blocks

A basic unit of any Ena program is a block.

```ena
# A block which is executed every time it is called
main {
  "hi"
}

# A block which is only executed once. After that, the top value is cached and will always be put on top of stack after execution.
random (
  ena.vm.random
)

# Now random will always output the same value.

main {
  random ena.vm.debug
  random ena.vm.debug
  # prints out the same value
}
```

> Block name can consist of any characters. However, ID cannot begin with a digit, quote(single or double), colon or a whitespace.

## Keywords

| Keyword        | Meaning                                                                       |
| -------------- | ----------------------------------------------------------------------------- |
| `if`           | Executes the block after it, if the top value on stack is equal to true.      |
| `while`        | Executes the block after it as long as the block pushes true onto the stack. |
| `return`       | Leaves the current global block.                                              |
| `return_local` | Leaves the current local block.                                               |

### Keyword Examples

```ena
# if example - note comparisons work in reverse
check_value {
    3 5 > if {  # Tests 5 > 3: true
        "5 is greater than 3" println
    }
}

# while example
count {
    i ( unit )
    0 i =
    
    # while executes the block and expects a boolean on stack
    5 i @ > while {
        i @ println
        i @ 1 + i =
        5 i @ >  # Push condition for next iteration
    }
}

# return example
early_exit {
    condition if {
        "Exiting early" println
        return
    }
    "This won't execute if condition is true" println
}
```

## Stack Operations

Ena provides several built-in operations for stack manipulation:

- **`dup`**: Duplicates the top value on stack
- **`swap`**: Swaps the top two values
- **`drop`**: Removes the top value
- **`clear`**: Clears the entire stack

```ena
stack_operations {
    5        # Stack: [5]
    dup      # Stack: [5, 5]
    3        # Stack: [5, 5, 3]
    swap     # Stack: [5, 3, 5]
    drop     # Stack: [5, 3]
}
```

## Memory and Variables

Ena uses a memory model with explicit allocation:

```ena
variables {
    # Allocate one unit of memory for a variable
    x ( unit )
    
    # Store value at memory location
    42 x =
    
    # Load value from memory location
    x @ println  # Prints: 42
}
```

### Operators

- **`=`**: Store value at memory location (pops value and pointer)
- **`@`**: Load value from memory location (pops pointer, pushes value)
- **`units`**: Allocate n units of memory
- **`unit`**: Allocate single unit (equivalent to `1 units`)

## Arithmetic Operations

Standard arithmetic operations work on stack values:

```ena
arithmetic {
    5 3 +    # Addition: 8
    2 10 -   # Subtraction: 8 (10 - 2, pops in reverse)
    4 3 *    # Multiplication: 12
    3 15 /   # Division: 5 (15 / 3, pops in reverse)
    2 8 pow  # Power: 256 (8^2, pops in reverse)
    2 4 root # Root: 2 (4^(1/2), pops in reverse)
}
```

**Note:** Operations pop values in reverse order (rightmost operand is popped first). For example, `2 10 -` pops 10 first, then 2, resulting in 10 - 2 = 8.

## Comparison Operations

**Note:** Comparison operators work in reverse of typical expectations. `a b >` tests if `b > a` (the second value is greater than the first).

```ena
comparisons {
    5 3 >    # Tests 3 > 5: false
    3 5 >    # Tests 5 > 3: true
    5 5 ==   # Equal: true
    3 5 >=   # Tests 5 >= 3: true
    5 3 <=   # Tests 3 <= 5: true
}
```

## Boolean Operations

```ena
boolean_ops {
    true false and  # Logical AND: false
    true false or   # Logical OR: true
    true !          # Logical NOT: false
}
```

## Block Calls

Blocks can call other blocks by name:

```ena
helper {
    "Helper called" println
}

main {
    helper  # Calls helper block
}
```

### Escaped Blocks

Escaped blocks (using single quote) put the block reference on stack:

```ena
dynamic_call {
    'helper call  # Dynamically calls the helper block
}

helper {
    "Called dynamically" println
}
```