# Contributing

As of now, I don't accept any contributions. I want to have some time to work on the language on my own.

However, I will provide some information about repository structure(very little but still).

## Repo

This repo is a Cargo workspace, which consists of three packages:
    - `enalang` - umbrella package, which contains the binary 
    - `enalang_compiler` - contains IR and the compiler. Independent of virtual machine.
    - `enalang_vm` - virtual machine for Ena.
    - `enalang_llvm` - planned. LLVM code generator.

## `./tools.py`

### Versioning

Ena uses a static version for all packages. Before release, you should change version in version.txt file and
run `./tools.py --set-version true`.

### Publishing

You can publish Ena using `./tools.py --publish true`.

### Dry mode

`./tools.py` can also be ran in dry mode: `./tools.py --dry true`