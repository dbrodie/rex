#[macro_use]
extern crate lazy_static;
extern crate odds;

extern crate rex;

mod util;

use std::path::Path;

use odds::vec::VecExt;

use rex::frontend::KeyPress;

use util::mock_filesystem::{MockFilesystem, ThreadedMockFilesystem};

#[test]
/// Test that copy/paste works in insert mode
fn test_insert_copy_paste() {
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

    println!("v_copy = {:?}", v_copy);

    edit.save(Path::new("test_insert_copy_paste"));
    util::assert_iter_eq(v_copy.iter(), ThreadedMockFilesystem::get_inner("test_insert_copy_paste").iter());
}

#[test]
/// Test that copy/paste works in overwrite mode
fn test_overwrite_copy_paste() {
    let v : Vec<u8> = (0..0xff).into_iter().collect();
    let mut v_copy = v.clone();

    let (mut edit, mut frontend) = util::simple_init_with_vec(v);

    // Add some junk data in the begining
    frontend.run_str(&mut edit, "AABBCCDDEE");
    v_copy.splice(..5, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

    // Copy it
    frontend.run_keys(&mut edit, vec![KeyPress::Left, KeyPress::Shortcut(' '), KeyPress::Home, KeyPress::Shortcut('c')]);

    // Paste it in the middle
    frontend.run_keys(&mut edit, vec![KeyPress::Shortcut('g')]);
    frontend.run_str(&mut edit, "50");
    frontend.run_keys(&mut edit, vec![KeyPress::Enter, KeyPress::Shortcut('v')]);
    assert_eq!(edit.get_position(), 55);
    v_copy.splice(50..55, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

    // Paste it in the end - in overwrite mode, pasting past the end does an insert
    frontend.run_keys(&mut edit, vec![KeyPress::PageDown, KeyPress::PageDown, KeyPress::PageDown, KeyPress::Shortcut('v')]);
    let l = v_copy.len();
    v_copy.splice(l.., vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

    println!("v_copy = {:?}", v_copy);

    edit.save(Path::new("test_overwrite_copy_paste"));
    util::assert_iter_eq(v_copy.iter(), ThreadedMockFilesystem::get_inner("test_overwrite_copy_paste").iter());
}


#[test]
/// Test that cut/paste works in insert mode
fn test_cut_paste() {
    let v : Vec<u8> = (0..0xff).into_iter().collect();
    let mut v_copy = v.clone();

    let (mut edit, mut frontend) = util::simple_init_with_vec(v);

    // Add some junk data in the begining
    frontend.run_keys(&mut edit, vec![KeyPress::Insert]);
    frontend.run_str(&mut edit, "AABBCCDDEE");

    // Cut it
    frontend.run_keys(&mut edit, vec![KeyPress::Left, KeyPress::Shortcut(' '), KeyPress::Home, KeyPress::Shortcut('x')]);

    // Paste it in the end
    frontend.run_keys(&mut edit, vec![KeyPress::PageDown, KeyPress::PageDown, KeyPress::PageDown, KeyPress::Shortcut('v')]);
    let l = v_copy.len();
    v_copy.splice(l.., vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

    println!("v_copy = {:?}", v_copy);

    edit.save(Path::new("test_cut_paste"));
    util::assert_iter_eq(v_copy.iter(), ThreadedMockFilesystem::get_inner("test_cut_paste").iter());
}
