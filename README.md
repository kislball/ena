# ena
> exhaustingly notorious algorithms

ena is a very bad language, implementation of which was initially written in c, but later rewritten in rust,
because the main developer was tired of debugging segfaults.

ena is always lowercase.

## Syntax

ena is a stack-based language. It means, that it operates on stack. Seems logical, isn't it?

It is much easier to show, than to explain.

ena doesn't have functions, but it has *words*. The only difference is that words *do not* accept any kind of arguments.
Instead, it takes arguments it needs from the stack.

```
# comments are written python style

# lets implement our own println word based on top of ena.print word.
println {
  # the stack currently has our "Hello, world!" string and the "\n" string.
  "\n" # "\n" "Hello, world!"
  # since we need to append \n to our "Hello, world!", we need to swap them and concat them using concat_str word.
  swap # "Hello, world!" "\n"
  # concat_str takes the top word on the stack and appends the second string to the top word.
  # both arguments are dropped.
  concat_str # now the stack only contains "Hello, world!\n" string.
  print # now we just print it
}

# word "main" is the entry point of an ena program
main {
  "Hello, world!" println
  "it can be any string by the way" println
}
```

If remove all comments, the resulting code is pretty short and neat.

```
println {
  "\n" swap concat_str print
}

main {
  "Hello, world!" println
  "it can be any string by the way" println
}
```