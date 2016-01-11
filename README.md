# Rex - Lean hex editor written in [Rust](http://www.rust-lang.org/)

[![Build Status](https://api.travis-ci.org/dbrodie/rex.svg?branch=master)](https://travis-ci.org/dbrodie/rex)

Rex is a small and lean terminal hex editor that is in an extremely alpha stage.

## Motivation

Most simple open source hex editor make certain operations too painful. From
inserting/deleting in the middle of a file to easily selecting and copy/pasting,
aims to make it simple and easy to use.

## Status

Currently Rex is in an extremely alpha stage, and while quite functional,
should not be used without backups. Future goals for Rex include:

- Better support for huge files (mmap based)
- Support a simple QML/Gtk GUI
- Basic struct/marking support

## Using

To be able to compile Rex, make sure you have the latest version of Rust
installed (stable release channel should suffice).
To compile the latest binary:

```base
git clone git@github.com:dbrodie/rex.git
cd rex
cargo build --release
./target/release/trex --help
```

To get help about how to use the program, after running it, press `Ctrl-/`.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
