#[macro_use]
extern crate lazy_static;

extern crate rex;

mod util;

use std::path::Path;

use rex::frontend::{Event, KeyPress};

use util::mock_filesystem::MockFilesystem;

// Little helper function till Iterator.eq stabalizes
fn iter_eq<I, J>(one: I, other: J) -> bool where
    I: IntoIterator,
    J: IntoIterator,
    I::Item: PartialEq<J::Item>,
{
    let mut one = one.into_iter();
    let mut other = other.into_iter();

    loop {
        match (one.next(), other.next()) {
            (None, None) => return true,
            (None, _) | (_, None) => return false,
            (Some(x), Some(y)) => if x != y { return false },
        }
    }
}

#[test]
fn test_basic_open() {
    // Create a vec with a marker in the end
    let mut v = vec![0; 1000];
    let len = v.len();
    v[len-1] = 0xAA;

    let (mut edit, mut frontend) = util::simple_init_empty();
    let pedit = &mut edit;

    MockFilesystem::put("test_basic_open", v);

    // Open file with the marker
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('o')]);
    frontend.run_str(pedit, "test_basic_open");
    frontend.run_keys(pedit, vec![KeyPress::Enter]);

    // Find the marker
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('f')]);
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('h')]);
    frontend.run_str(pedit, "AA");
    frontend.run_keys(pedit, vec![KeyPress::Enter]);

    // Make sure the opened file name is correct
    let name = Path::new("test_basic_open");
    assert_eq!(name, pedit.get_file_path().unwrap());

    // And make sure it is in the right place
    assert_eq!(pedit.get_position(), (len-1) as isize);
}

#[test]
fn test_basic_save() {
    // Create a view over a generic vector
    let v = util::generate_vec(1000);
    let (mut edit, mut frontend) = util::simple_init_with_vec(v.clone());
    let pedit = &mut edit;

    // Save it to a file
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('s')]);
    frontend.run_str(pedit, "test_basic_save");
    frontend.run_keys(pedit, vec![KeyPress::Enter]);

    // Make sure they are equal
    assert!(iter_eq(v.iter(), MockFilesystem::get_inner("test_basic_save").iter()));
}
