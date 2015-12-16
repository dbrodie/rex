#[macro_use]
extern crate lazy_static;
extern crate typenum;
extern crate odds;

extern crate rex;

mod util;

use std::iter;
use std::path::Path;
use std::convert::{From, Into};
use std::io::{Cursor, Write, Seek, SeekFrom};

use odds::vec::VecExt;

use rex::frontend::{Event, KeyPress};

use util::mock_filesystem::{DefMockFilesystem, MockFilesystem};

#[test]
/// Test that basic copy/paste works
fn test_copy_paste() {
    let v : Vec<u8> = (0..0xff).into_iter().collect();
    let mut v_copy = v.clone();

    let (mut edit, mut frontend) = util::simple_init_with_vec(v);

    // Add some junk data in the begining
    frontend.run_keys(&mut edit, vec![KeyPress::Insert]);
    frontend.run_str(&mut edit, "AABBCCDDEE");
    v_copy.splice(..0, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

    // Copy it
    frontend.run_keys(&mut edit, vec![KeyPress::Left, KeyPress::Shortcut(' '), KeyPress::Home, KeyPress::Shortcut('c')]);

    // Paste it in the middle
    frontend.run_keys(&mut edit, vec![KeyPress::Shortcut('g')]);
    frontend.run_str(&mut edit, "50");
    frontend.run_keys(&mut edit, vec![KeyPress::Enter, KeyPress::Shortcut('v')]);
    assert_eq!(edit.get_position(), 55);
    v_copy.splice(50..50, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

    // Paste it in the end
    frontend.run_keys(&mut edit, vec![KeyPress::PageDown, KeyPress::PageDown, KeyPress::PageDown, KeyPress::Shortcut('v')]);
    let l = v_copy.len();
    v_copy.splice(l.., vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

    edit.save(Path::new("test_copy_paste"));
    util::assert_iter_eq(v_copy.iter(), DefMockFilesystem::get_inner("test_copy_paste").iter());
}
