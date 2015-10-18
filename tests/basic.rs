extern crate rex;

mod util;

use rex::frontend::{Event, KeyPress};

#[test]
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
