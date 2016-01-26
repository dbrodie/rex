#![cfg_attr(all(test, feature = "nightly"), feature(test))]

extern crate rustbox;
extern crate rustc_serialize;
extern crate toml;
extern crate itertools;
extern crate odds;
#[macro_use] extern crate custom_derive;
#[macro_use] extern crate newtype_derive;
#[cfg(test)] pub mod bench;

#[macro_use] pub mod util;
pub mod config;
pub mod filesystem;
pub mod frontend;
pub mod ui;
