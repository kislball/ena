# Ena Standard Library

This folder includes the entirety of Ena's standard library.

## Compiling

```sh
$ enalang build ./std/*.ena
```

## Using

To use the standard library, you will have to link it to your IR:

```sh
$ enalang link ./std.enair ./my_program.enair
```