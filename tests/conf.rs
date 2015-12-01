#[macro_use]
extern crate lazy_static;
extern crate typenum;

extern crate rex;

mod util;

use std::path::Path;
use std::io::{Read, Write, BufReader};
use std::str;

use typenum::uint::Unsigned;

use rex::frontend::{Event, KeyPress};
use rex::config::Config;

use util::mock_filesystem::{MockFilesystem, TestOpenSaveConfig};


#[test]
fn open_save_conf_test() {
    // Write a simple configuration file
    {
        let mut f = MockFilesystem::<TestOpenSaveConfig>::save_config().unwrap();
        f.write_all("show_ascii=false\n".as_bytes());
    }

    // Create an editor
    let (mut edit, mut frontend) = util::simple_init_helper::<TestOpenSaveConfig>(None);
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
        let mut f = MockFilesystem::<TestOpenSaveConfig>::open_config().unwrap();
        f.read_to_end(&mut buf);
        assert_eq!(str::from_utf8(&buf).unwrap().lines().next().unwrap(), "show_ascii=true");
    }
}
