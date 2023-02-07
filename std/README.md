# Ena Standard Library

This folder includes the entirety of Ena's standard library.

## Structure
* `base` contains parts of Ena standard library that only rely on Ena's core operators.
* `c` and `vm` both have the same interface, but they differ in implementation. `vm` uses `ena.vm` namespace to access VM's methods, whereas `c` uses `ena.c` namespace to call C functions. You should only link against one of those.

## Building

```shell
$ ena compile ./base/**/*.ena -o ./base-std.enair
$ ena compile ./c/**/*.ena -o ./platform.enair # for C
$ ena compile ./vm/**/*.ena -o ./platform.enair # for VM
$ ena link ./base-std.enair ./platform.enair -o ./std.enair
```

## Usage

To use Ena's standary library, you will have to link it with your program:

```shell
$ ena compile ./my_program.ena -o ./my_program.enair
$ ena link ./std.enair ./my_program.enair
$ ena run ./output.enair # if you used VM bindings
# if you used C bindings
$ ena cgen ./output.enair -o ./output.c
$ cc ./output.c
```