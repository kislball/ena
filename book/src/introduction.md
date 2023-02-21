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

Ena uses a VM to run code. VM cannot read Ena code directly, it reads an intermediate representation from Ena's compiler:

```console
$ ena compile ./hello_world.ena -o hello_world.enair
```

Now, let's try to run it:

```console
$ ena run ./hello_world.enair
```

We will run into an error. This is because Ena does not link standard library by default. To link it, we will first have
to compile it.

```console
$ ena compile ./std/vm/**/*.ena ./std/base/**/*.ena -o std.enair
$ ena link ./std.enair ./hello_world.enair -o executable.enair
```

Now let's finally run it:

```console
$ ena run ./executable.enair
```

Now, you should see a "Hello, world!" message.

## What's next?

In the next chapter, we will look into more details about coding in Ena.

