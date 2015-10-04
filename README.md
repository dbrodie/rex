# Rex - Lean hex editor written in [Rust](http://www.rust-lang.org/)

Rex is a small and lean terminal hex editor that is in an extremly alpha stage.

## Motivation

Most simple open source hex editor make certain operations too painful. From
inserting/deleting in the middle of a file to easily selecting and copy/pasting,
aims to make it simple and easy to use.

## Status

Currently Rex is in an extrmely alpha stage, and while quite functional,
should not be used without backups. Future goals for Rex include:

- Better support for huge files (mmap based)
- Support a simple QML/Gtk GUI
- Basic struct/marking support

## Using

To be able to compile Rex, make sure you have a recent version of Rust
installed (stable release channel should suffice).
To compile the latest binary:

```base
git clone git@github.com:dbrodie/rex.git
cd rex
cargo build --release
./target/release/trex --help
```

To get help about how to use the program, after running it, press `Ctrl-/`.
