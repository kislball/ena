# ena
> exhaustingly notorious algorithms

Ena is a very bad language, whose implementation was originally written in C, but was later rewritten in Rust because the main developer was tired of debugging segfaults.

## Syntax
Ena is a stack-based language. This means it works on the stack. Seems logical, doesn't it?

It is much easier to show than to explain.

Ena doesn't have functions, it has words. The only difference is that words do not take arguments. Instead, it takes the arguments it needs from the stack.

```
# Comments are written in Python style.

# Let's implement our own println word based on the ena.io.print word.
println {
  # The stack currently has our "Hello, world!" string and the "\n" string.
  "\n" # "\n" "Hello, world!"
  # Since we need to append \n to our "Hello, world!", we need to swap them and concat them using the concat_str word.
  swap # "Hello, world!" "\n"
  # concat_str takes the top word on the stack and appends the second string to the top word.
  # Both arguments are dropped.
  concat_str # Now the stack only contains the string "Hello, world!\n".
  ena.io.print # now we just print it
}

# The word "main" is the entry point of an ena program.
main {
  "Hello, world!" println
  "It can be any string, by the way" println
}
```

With all the comments removed, the result is a very short and tidy piece of code.

```
println {
  "\n" swap concat_str print
}

main {
  "Hello, world!" println
  "It can be any string, by the way" println
}
```