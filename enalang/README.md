#### Disclaimer: this readme is for developers interested in embedding Ena in their programs. If you are intersted in using Ena, please read the readme from the [Github page](https://github.com/kislball/ena).

# Enalang

This crate contains the binary package for Ena and a Wrapper(`enalang::Ena`). It also reexports `enalang_compiler` as compiler and `enalang_vm` as vm.

## Example

```rs
let mut ena = enalang::Ena::new(enalang::EnaOptions::default());
ena.read_files(&[String::from("./test.ena")]).unwrap();
ena.parse_files().unwrap();
ena.compile_files().unwrap();
ena.link_files().unwrap();
ena.run("main").unwrap();
```

See the [Github page](https://github.com/kislball/ena) for more info.