# ena
> exhaustingly notorious algorithms

Ena is a stack-based programming language. The implementation was originally written in C and subsequently rewritten in Rust to improve memory safety and development efficiency.

## Installing
Currently, Ena does not provide prebuilt binaries. Users must build the project from source.

Ena is implemented in Rust and requires `cargo` and `rustc` to be installed on your system.

Build and install command: 

```sh
# Install the stable version from crates.io
$ cargo install enalang

# Install from a cloned repository
$ cargo install --path ./enalang
```

Verify the installation:

```sh
$ ena --version
```

If the command fails, ensure that your PATH environment variable is properly configured.

## Syntax
Ena is a stack-based programming language that operates by manipulating values on a stack data structure.

The language design is best understood through examples.

Ena uses blocks rather than traditional functions. Unlike functions, blocks do not explicitly declare parameters. Instead, blocks consume their required arguments directly from the stack.

```
# Comments follow Python syntax conventions.

# Example: printing hello world.

# The main block serves as the program entry point.
main {
    print_hello # Invokes the print_hello block.
}

# Define a separate block that can be called from main.
print_hello {
    "hello world!" # String literals are pushed onto the stack.
    println # The println operation pops the top stack value and outputs it with a newline.
}
```

With comments removed, the code remains concise and readable.

Additional examples are available in the `examples` directory.

## Learn
You can further learn about ENA by reading (the book)[https://kislball.github.io/ena] or by inspecting examples in the corresponding folder. 