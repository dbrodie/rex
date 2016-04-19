#[macro_use]
extern crate lazy_static;

extern crate rex;

mod util;

use std::io::{Read, Write};
use std::str;

use rex::frontend::KeyPress;

use util::mock_filesystem::{MockFilesystem, ThreadedMockFilesystem};


#[test]
fn open_save_conf_test() {
    // Write a simple configuration file
    {
        let mut f = ThreadedMockFilesystem::save_config().unwrap();
        f.write_all("show_ascii=false\n".as_bytes()).unwrap();
    }

    // Create an editor
    let (mut edit, mut frontend) = util::simple_init_empty();
    let pedit = &mut edit;

    assert_eq!(pedit.get_config().show_ascii, false);

    // Hardcoded changing of an option (until we get a more generic way to change an option)
    // Open Configration and edit :
    frontend.run_keys(pedit, vec![KeyPress::Shortcut('\\')]);
    frontend.run_str(pedit, "c");
    frontend.run_keys(pedit, vec![KeyPress::Enter]);

    // Change it to true
    frontend.run_keys(pedit, vec![KeyPress::Backspace; 5]);
    frontend.run_str(pedit, "true");
    frontend.run_keys(pedit, vec![KeyPress::Enter]);

    assert_eq!(pedit.get_config().show_ascii, true);

    {
        let mut buf = Vec::new();
        let mut f = ThreadedMockFilesystem::open_config().unwrap();
        f.read_to_end(&mut buf).unwrap();
        assert_eq!(str::from_utf8(&buf).unwrap().lines().next().unwrap(), "show_ascii=true");
    }
}
