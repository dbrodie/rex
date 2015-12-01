#[macro_use]
extern crate lazy_static;
extern crate typenum;

extern crate rex;

mod util;

use std::iter;

use rex::frontend::{Event, KeyPress};

#[test]
/// Test that moving over the top works
fn test_top_cutoff() {
    let (mut edit, mut frontend) = util::simple_init(0x1000);

    assert_eq!(edit.get_position(), 0);

    frontend.run_keys(&mut edit, vec![KeyPress::Right, KeyPress::Up, KeyPress::Left]);
    assert_eq!(edit.get_position(), 0);

    frontend.run_keys(&mut edit, vec![KeyPress::Down, KeyPress::Left, KeyPress::Up]);
    assert_eq!(edit.get_position(), 0);

    frontend.run_keys(&mut edit, vec![KeyPress::Down, KeyPress::Right, KeyPress::Up, KeyPress::Up]);
    assert_eq!(edit.get_position(), 0);

    frontend.run_keys(&mut edit, vec![KeyPress::PageDown, KeyPress::PageUp, KeyPress::PageUp]);
    assert_eq!(edit.get_position(), 0);

    frontend.run_keys(&mut edit, vec![KeyPress::PageDown, KeyPress::Right, KeyPress::PageUp, KeyPress::PageUp]);
    assert_eq!(edit.get_position(), 0);
}

#[test]
/// Test that moving under the bottom works
fn test_bottom_cutoff() {
    let size: isize = 0x1000;
    let (mut edit, mut frontend) = util::simple_init(size as usize);

    assert_eq!(edit.get_position(), 0);

    frontend.run_keys(&mut edit, vec![KeyPress::Shortcut('g')]);
    frontend.run_str(&mut edit, "4100");
    frontend.run_keys(&mut edit, vec![KeyPress::Enter]);
    assert_eq!(edit.get_position(), size);

    frontend.run_keys(&mut edit, vec![KeyPress::Left, KeyPress::Down, KeyPress::Right]);
    assert_eq!(edit.get_position(), size);

    frontend.run_keys(&mut edit, vec![KeyPress::Up, KeyPress::Right, KeyPress::Down]);
    assert_eq!(edit.get_position(), size);

    frontend.run_keys(&mut edit, vec![KeyPress::Up, KeyPress::Left, KeyPress::Down, KeyPress::Down]);
    assert_eq!(edit.get_position(), size);

    frontend.run_keys(&mut edit, vec![KeyPress::PageUp, KeyPress::PageDown, KeyPress::PageDown]);
    assert_eq!(edit.get_position(), size);

    frontend.run_keys(&mut edit, vec![KeyPress::PageUp, KeyPress::Left, KeyPress::PageDown, KeyPress::PageDown]);
    assert_eq!(edit.get_position(), size);
}

#[test]
/// Test the goto behavior
fn test_goto() {
    let size: isize = 0x1000;
    let (mut edit, mut frontend) = util::simple_init(size as usize);
    let pedit = &mut edit;

    assert_eq!(pedit.get_position(), 0);

    // Default is decimal
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('g')]);
    frontend.run_str(pedit, "100");
    frontend.run_keys(pedit, vec![KeyPress::Enter]);
    assert_eq!(pedit.get_position(), 100);

    // Then comes hex
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('g')]);
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('h')]);
    frontend.run_str(pedit, "100");
    frontend.run_keys(pedit, vec![KeyPress::Enter]);
    assert_eq!(pedit.get_position(), 0x100);

    // Then comes octal
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('g')]);
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('o')]);
    frontend.run_str(pedit, "100");
    frontend.run_keys(pedit, vec![KeyPress::Enter]);
    assert_eq!(pedit.get_position(), 0o100);

    // And now just a big of random til we come back to decimal
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('g')]);
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('o')]);
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('h')]);
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('d')]);
    frontend.run_str(pedit, "50");
    frontend.run_keys(pedit, vec![KeyPress::Enter]);
    assert_eq!(pedit.get_position(), 50);
}

#[test]
/// Test the find behavior
fn test_find() {
    let mut vec: Vec<u8> = iter::repeat(0).take(100).collect();
    vec.append(&mut vec![0x78, 0x78, 0x78, 0x78]);
    vec.append(&mut iter::repeat(0).take(100).collect());
    let (mut edit, mut frontend) = util::simple_init_with_vec(vec);
    let pedit = &mut edit;

    // Try Ascii
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('f')]);
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('a')]);
    frontend.run_str(pedit, "xxxx");
    frontend.run_keys(pedit, vec![KeyPress::Enter]);
    assert_eq!(pedit.get_position(), 100);

    // Reset position
    frontend.run_keys(pedit, vec![KeyPress::PageUp, KeyPress::PageUp]);

    // Try Hex
    // Try Ascii
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('f')]);
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('h')]);
    frontend.run_str(pedit, "78787878");
    frontend.run_keys(pedit, vec![KeyPress::Enter]);
    assert_eq!(pedit.get_position(), 100);
}
