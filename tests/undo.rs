#[macro_use]
extern crate lazy_static;

extern crate rex;

mod util;

use std::path::Path;

use rex::frontend::{KeyPress};

use util::mock_filesystem::{MockFilesystem, ThreadLocalMockFilesystem};

#[test]
fn test_undo_insert() {
    let v : Vec<u8> = (0..0xff).into_iter().collect();
    let result = v.clone();

    let (mut edit, mut frontend) = util::simple_init_with_vec(v);

    frontend.run_str(&mut edit, "AA");
    assert_eq!(edit.get_position(), 1);

    frontend.run_keys(&mut edit, vec![KeyPress::Shortcut('z'), KeyPress::Shortcut('z')]);
    assert_eq!(edit.get_position(), 0);

    edit.save(Path::new("test_undo_insert"));
    util::assert_iter_eq(result.iter(), MockFilesystem::<ThreadLocalMockFilesystem>::get_inner("test_undo_insert").iter());
}

#[test]
fn test_undo_delete() {
    let v : Vec<u8> = (0..0xff).into_iter().collect();
    let result = v.clone();

    let (mut edit, mut frontend) = util::simple_init_with_vec(v);

    frontend.run_keys(&mut edit, vec![KeyPress::Delete]);

    frontend.run_keys(&mut edit, vec![KeyPress::Shortcut('z')]);
    assert_eq!(edit.get_position(), 0);

    edit.save(Path::new("test_undo_delete"));
    util::assert_iter_eq(result.iter(), MockFilesystem::<ThreadLocalMockFilesystem>::get_inner("test_undo_delete").iter());
}
