# ena
> exhaustingly notorious algorithms

Ena is a very bad language, whose implementation was originally written in C, but was later rewritten in Rust because the main developer was tired of debugging segfaults.

## Installing
As of now, Ena does not provide prebuilt binaries, it is your task to build it yourself.

Ena is written in Rust, so you should have `cargo` and `rustc` installed.

Build and install command: 

```sh
$ cargo install --path .
```

Test if you have installed Ena:

```sh
$ enalang --version
```

If you see an error, check if your PATH variable is configured.

## Syntax
Ena is a stack-based language. This means it works on the stack. Seems logical, doesn't it?

It is much easier to show than to explain.

Ena doesn't have functions, it has blocks. The only difference is that blocks do not take arguments. Instead, it takes the arguments it needs from the stack.

```
# Comments are written in Python style.

# Let's implement our own println block based on the ena.io.print block.
println {
  # The stack currently has our "Hello, world!" string and the "\n" string.
  "\n" # "\n" "Hello, world!"
  # Since we need to append \n to our "Hello, world!", we need to swap them and concat them using the concat_str block.
  swap # "Hello, world!" "\n"
  # concat_str takes the top block on the stack and appends the second string to the top block.
  # Both arguments are dropped.
  concat_str # Now the stack only contains the string "Hello, world!\n".
  ena.io.print # now we just print it
}

# The block "main" is the entry point of an ena program.
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