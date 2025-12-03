# Chapter 1 - Introduction

Welcome to the Ena book! This book serves as a simple introduction to the Ena Programming Language.

## Disclaimer

Ena is not complete yet. This means that Ena is not a mature language you would want to use for something serious.
However, if you wanted to try something new, Ena is for you.

## How is Ena different?

Unlike many other languages, Ena is a stack-based language. If you have worked with languages like Forth,
you may find many Ena's concepts similar to those in Forth.

## Installing Ena

As of now, Ena does not provide any pre-built binaries. You will have to build it on your own.

The simplest way is to install [Rust](https://rust-lang.org/tools/install) and then compile and install Ena:

```console
$ cargo install enalang
```

You can also compile and install Ena from source:

```console
$ git clone https://github.com/kislball/enalang
$ cd enalang
$ cargo install --path ./enalang
```

## Hello, world!

The following code snippet describes a program which prints out a "Hello, world!" message.

```ena
# Comments are written using a hash sign

# Each Ena program is composed of blocks. Blocks are similar to functions, except that they do not accept
# any arguments.

# By default, Ena's VM uses `main` as the name for main block.

main {
  "Hello, world!" # Literals put values on stack. Blocks usually operate on stack.
  println # println prints the top value on stack, converts it into a string and prints it
}
```

Without comments:

```ena
main {
  "Hello, world!" println
}
```

### Running

Ena uses a VM to run code. The VM cannot read Ena code directly; it reads an intermediate representation (IR) from Ena's compiler.

First, compile your source code to IR:

```console
$ ena compile ./hello_world.ena -o hello_world.enair
```

Next, compile the standard library (you only need to do this once):

```console
$ ena compile "std/*.ena" -o std.enair
```

Link your program with the standard library:

```console
$ ena link hello_world.enair std.enair -o executable.enair
```

Finally, run your program:

```console
$ ena run ./executable.enair
```

You should now see a "Hello, world!" message.

For convenience, you can create a shell script to automate this process. The repository includes a `run_example.sh` script:

```bash
#!/bin/sh
# Usage: ./run_example.sh <filename_without_extension>

ena compile "std/*.ena" -o ./std.enair
ena compile ./examples/$1.ena -o ./main.enair
ena link ./main.enair ./std.enair -o output.enair
ena run ./output.enair
rm output.enair main.enair std.enair
```

## What's next?

In the next chapter, we will look into more details about coding in Ena.

