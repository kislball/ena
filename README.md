# ena
> exhaustingly notorious algorithms

Ena is a very bad language, whose implementation was originally written in C, but was later rewritten in Rust because the main developer was tired of debugging segfaults.

## Installing
As of now, Ena does not provide prebuilt binaries, it is your task to build it yourself.

Ena is written in Rust, so you should have `cargo` and `rustc` installed.

Build and install command: 

```sh
# If you want a stable version
$ cargo install enalang

# If you have cloned the repo
$ cargo install --path ./enalang
```

Test if you have installed Ena:

```sh
$ ena --version
```

If you see an error, check if your PATH variable is configured.

## Syntax
Ena is a stack-based language. This means it works on the stack. Seems logical, doesn't it?

It is much easier to show than to explain.

Ena doesn't have functions, it has blocks. The only difference is that blocks do not take arguments. Instead, it takes the arguments it needs from the stack.

```
# Comments are written in Python style.

# Let's print hello world.

# Main is the entrypoint of any ena program.
main {
    print_hello # Calls print_hello block.
}

# Let's define another block to call from main.
print_hello {
    "hello world!" # A literal puts the value on top of stack.
    println # println takes the literal on top of stack and prints it with a new line.
}
```

With all the comments removed, the result is a very short and tidy piece of code.

See more examples in the `examples` folder.