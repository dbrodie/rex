#[macro_use]
extern crate lazy_static;
extern crate odds;

extern crate rex;

mod util;

use std::path::Path;
use std::iter;
use std::io::{Write, Cursor, Seek, SeekFrom};

use odds::vec::VecExt;

use rex::frontend::{Event, KeyPress};

use util::mock_filesystem::{ThreadLocalMockFilesystem, MockFilesystem};

#[test]
fn test_edit_overwrite() {
    let v : Vec<u8> = (0..0xff).into_iter().collect();
    let mut result_cursor = Cursor::new(v.clone());

    let (mut edit, mut frontend) = util::simple_init_with_vec(v);

    // Overwrite with some junk data in the begining
    frontend.run_str(&mut edit, "AABBCCDDEE");
    result_cursor.write(&vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]).unwrap();

    // Overwrite some junk in the middle
    frontend.run_keys(&mut edit, vec![KeyPress::Shortcut('g')]);
    frontend.run_str(&mut edit, "50");
    frontend.run_keys(&mut edit, vec![KeyPress::Enter]);
    frontend.run_str(&mut edit, "AABBCCDDEE");
    assert_eq!(edit.get_position(), 55);
    result_cursor.seek(SeekFrom::Start(50)).unwrap();
    result_cursor.write(&vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]).unwrap();

    // Overwrite it in the end (where it should append)
    frontend.run_keys(&mut edit, vec![KeyPress::PageDown, KeyPress::PageDown, KeyPress::PageDown]);
    frontend.run_str(&mut edit, "AABBCCDDEE");
    result_cursor.seek(SeekFrom::End(0)).unwrap();
    result_cursor.write(&vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]).unwrap();

    let result = result_cursor.into_inner();
    edit.save(Path::new("test_copy_paste"));
    util::assert_iter_eq(result.iter(), MockFilesystem::<ThreadLocalMockFilesystem>::get_inner("test_copy_paste").iter());
}

#[test]
fn test_edit_insert() {
    let v : Vec<u8> = (0..0xff).into_iter().collect();
    let mut result = v.clone();

    let (mut edit, mut frontend) = util::simple_init_with_vec(v);

    // Insert with some junk data in the begining
    frontend.run_keys(&mut edit, vec![KeyPress::Insert]);
    frontend.run_str(&mut edit, "AABBCCDDEE");
    result.splice(..0, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

    // Insert some junk in the middle
    frontend.run_keys(&mut edit, vec![KeyPress::Shortcut('g')]);
    frontend.run_str(&mut edit, "50");
    frontend.run_keys(&mut edit, vec![KeyPress::Enter]);
    frontend.run_str(&mut edit, "AABBCCDDEE");
    assert_eq!(edit.get_position(), 55);
    result.splice(50..50, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

    // Insert it in the end (where it should append)
    frontend.run_keys(&mut edit, vec![KeyPress::PageDown, KeyPress::PageDown, KeyPress::PageDown]);
    frontend.run_str(&mut edit, "AABBCCDDEE");
    let len = result.len();
    result.splice(len.., vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

    edit.save(Path::new("test_edit_insert"));
    util::assert_iter_eq(result.iter(), MockFilesystem::<ThreadLocalMockFilesystem>::get_inner("test_edit_insert").iter());
}

#[test]
fn test_edit_delete_and_bksp() {
    let v : Vec<u8> = (0..0xff).into_iter().collect();
    let mut result = v.clone();

    let (mut edit, mut frontend) = util::simple_init_with_vec(v);

    // Delete some chars in the begining (where the bksp key should be a no-op)
    frontend.run_keys(&mut edit, vec![KeyPress::Backspace, KeyPress::Backspace]);
    assert_eq!(edit.get_position(), 0);
    // And actually delete some chars
    frontend.run_keys(&mut edit, vec![KeyPress::Right, KeyPress:: Right, KeyPress::Right, KeyPress:: Right,
        KeyPress::Backspace, KeyPress::Backspace]);
    assert_eq!(edit.get_position(), 0);
    frontend.run_keys(&mut edit, vec![KeyPress::Delete, KeyPress::Delete]);
    assert_eq!(edit.get_position(), 0);
    result.splice(0..4, vec![]);

    // Delete some chars in the middle
    frontend.run_keys(&mut edit, vec![KeyPress::Shortcut('g')]);
    frontend.run_str(&mut edit, "50");
    frontend.run_keys(&mut edit, vec![KeyPress::Enter]);
    frontend.run_keys(&mut edit, vec![KeyPress::Delete, KeyPress::Delete]);
    assert_eq!(edit.get_position(), 50);
    frontend.run_keys(&mut edit, vec![KeyPress::Backspace, KeyPress::Backspace]);
    assert_eq!(edit.get_position(), 48);
    result.splice(48..52, vec![]);

    // Delete in the end (where the delete key should be a no-op)
    let mut len = result.len();
    frontend.run_keys(&mut edit, vec![KeyPress::PageDown, KeyPress::PageDown, KeyPress::PageDown]);
    frontend.run_keys(&mut edit, vec![KeyPress::Delete, KeyPress::Delete]);
    assert_eq!(edit.get_position(), len as isize);
    // And actually delete some chars
    frontend.run_keys(&mut edit, vec![KeyPress::Left, KeyPress::Left, KeyPress::Left, KeyPress::Left,
        KeyPress::Delete, KeyPress::Delete]);
    frontend.run_keys(&mut edit, vec![KeyPress::Backspace, KeyPress::Backspace]);
    result.splice((len-4).., vec![]);
    len = result.len();
    assert_eq!(edit.get_position(), len as isize);

    println!("result = {:?}", result);

    edit.save(Path::new("test_edit_delete_and_bksp"));
    util::assert_iter_eq(result.iter(), MockFilesystem::<ThreadLocalMockFilesystem>::get_inner("test_edit_delete_and_bksp").iter());
}
