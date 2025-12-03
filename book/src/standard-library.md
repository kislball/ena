# Chapter 4 - Standard Library

The Ena standard library provides essential functionality for I/O operations, memory management, string manipulation, control flow helpers, and more. This chapter documents the key modules and their usage.

## Core Operations (`ops.ena`)

### Logical Operators

```ena
# Logical NOT equal
!= {
    == !
}
```

## Input/Output (`io.ena`)

### Basic Printing

```ena
# Print value as string without newline
print {
    into_string ena.vm.io.print
}

# Print value with newline
println {
    print "\n" ena.vm.io.print
}
```

**Example:**
```ena
main {
    "Hello, World!" println
    42 print " is the answer" println
}
```

## Loop Control (`loop.ena`)

### Break and Continue

```ena
# Exit from current loop
break {
    false return_local
}

# Skip to next iteration
continue {
    true return_local
}
```

**Example:**
```ena
find_number {
    i ( unit )
    0 i =
    
    true while {
        i @ 10 == if {
            break
        }
        i @ println
        i @ 1 + i =
        true
    }
}
```

## Memory Management (`mem.ena`)

### Memory Allocation

```ena
# Allocate memory units
units {
    alloc
}

# Allocate single unit
unit {
    1 units
}
```

**Example:**
```ena
main {
    # Allocate array of 5 elements
    arr ( unit )
    5 units arr =
    
    # Set specific values
    42 arr @ 0 + =
    99 arr @ 1 + =
    
    # Read values
    arr @ 0 + @ println  # 42
    arr @ 1 + @ println  # 99
}
```

**Note:** The `memcpy` and `memfill` functions in the standard library use outdated loop patterns and may not work correctly. Use manual loops for memory operations.

## String Operations

Ena provides built-in string operations through the VM:

### String Length

```ena
main {
    "Hello, World!" string.len println  # 13
}
```

### String Concatenation

**Note:** String concatenation works in reverse order due to stack popping - the second string is popped first.

```ena
main {
    "World!" "Hello, " string.concat println  # Produces "Hello, World!"
}
```

### String Splitting

```ena
main {
    " " "one two three" string.split
    # Stack now contains: ["one", "two", "three", 3]
    ena.vm.debug_stack
}
```

### String Contains

```ena
main {
    "lo" "Hello" string.contains println  # true
    "xyz" "Hello" string.contains println  # false
}
```

### String Characters

```ena
main {
    "ABC" string.chars
    # Stack now contains individual characters
    ena.vm.debug_stack
}
```

## Number Operations (`number.ena`)

### Square Root Operations

```ena
sqrt {
    2 swap root
}

rsqrt {
    sqrt 1 /
}
```

## Block Operations (`call.ena`)

### Dynamic Block Calling

```ena
# Call a block by reference
# Expects escaped block on stack
call_block {
    'my_block call
}
```

## Collections (`collections/vec.ena`)

The standard library includes vector operations for working with dynamic arrays. **Note:** Vector operations require explicitly compiling and linking `std/collections/vec.ena` with your program - they are not included in the default `std.enair`.

### Common Vector Operations

- `ena.vec.with_capacity`: Create vector with capacity
- `ena.vec.push`: Add element to vector
- `ena.vec.at`: Get element at index
- `ena.vec.cap`: Get vector capacity
- `ena.vec.size`: Get vector size
- `ena.vec.from_stack`: Create vector from stack values
- `ena.vec.reverse`: Reverse vector elements
- `ena.vec.each`: Apply block to each element

**Example:**
```ena
# Note: Requires compiling and linking with std/collections/vec.ena
main {
    vector ( unit )
    5 ena.vec.with_capacity vector =
    
    10 vector @ ena.vec.push
    20 vector @ ena.vec.push
    30 vector @ ena.vec.push
    
    0 vector @ ena.vec.at println  # 10
    vector @ ena.vec.size println  # 3
}
```

## VM Debug Operations

### Stack Inspection

```ena
# Print top value
ena.vm.debug

# Print entire stack
ena.vm.debug_stack

# Print call stack
ena.vm.debug_calls
```

### Random Numbers

```ena
main {
    # Get random number
    ena.vm.random println
}
```

## Type Operations

### Type Checking

```ena
is_exception {
    # Check if top value is an exception
}

into_exception {
    # Convert value to exception
}

into_string {
    # Convert value to string
}
```

## File System Operations (`fs.ena`)

The standard library provides file system operations for reading and writing files:

```ena
# Read file contents
# Arguments: filename
ena.vm.fs.read

# Write to file
# Arguments: content filename
ena.vm.fs.write
```

## Operating System Operations (`os.ena`)

```ena
# Get environment variable
# Arguments: var_name
ena.vm.os.env

# Execute system command
# Arguments: command
ena.vm.os.exec
```

## Best Practices

1. **Import only what you need**: While the standard library is automatically linked, be mindful of which operations you use.

2. **Memory management**: Always free allocated memory when no longer needed to prevent leaks.

3. **Error handling**: Use exception handling with VM operations that may fail (file I/O, system calls).

4. **Stack discipline**: VM debug operations don't consume stack values, making them useful for debugging without disrupting flow.

5. **String operations**: Most string operations return new values rather than modifying in place.

## Extending the Standard Library

You can create your own standard library modules by placing `.ena` files in the `std/` directory and compiling them with your standard library compilation command.
