#[macro_use]
extern crate lazy_static;
extern crate typenum;
extern crate odds;

extern crate rex;

mod util;

use std::iter;
use std::fmt::Debug;
use std::path::Path;
use std::convert::{From, Into};
use std::io::{Cursor, Write, Seek, SeekFrom};

use odds::vec::VecExt;

use rex::frontend::{Event, KeyPress};

use util::mock_filesystem::{DefMockFilesystem, MockFilesystem};

fn assert_iter_eq<I, J>(one: I, other: J) where
    I: IntoIterator,
    J: IntoIterator,
    I::Item: PartialEq<J::Item> + Debug,
    J::Item: Debug,
{
    let mut one = one.into_iter();
    let mut other = other.into_iter();

    loop {
        match (one.next(), other.next()) {
            (None, None) => return,
            (Some(x), Some(y)) => assert_eq!(x, y),
            (a @ _, b @ _) => panic!("left is {:?}, right is {:?}", a, b)
        }
    }
}
