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
| `while`        | Executes the block after it as long as the top value on stack equals to true. |
| `return`       | Leaves the current global block.                                              |
| `return_local` | Leaves the current local block.                                               |