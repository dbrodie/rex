extern crate docopt;
extern crate gag;
extern crate rustbox;
extern crate rustc_serialize;
extern crate rex;

mod rex_term;

use std::path::Path;
use std::error::Error;
use std::process;
use docopt::Docopt;

use gag::Hold;

use rex::frontend::{Frontend, Event, KeyPress};
use rex::ui::view::HexEdit;

use rex_term::RustBoxFrontend;

static USAGE: &'static str = "
Usage: rex [options] [FILE]
       rex --help

Options:
    -h, --help                  Show this help message
";

#[derive(RustcDecodable, Debug)]
#[allow(non_snake_case)]
struct Args {
    flag_help: bool,
    arg_FILE: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(
        |d| d.help(false).decode()).unwrap_or_else(
        |e| e.exit()
    );

    if args.flag_help {
        println!("{}", USAGE.trim());
        process::exit(0);
    }

    let mut edit: HexEdit = HexEdit::new();

    if let Some(ref filename) = args.arg_FILE {
        edit.open(&Path::new(filename));
    }

    let hold = (Hold::stdout().unwrap(), Hold::stderr().unwrap());

    let mut frontend = RustBoxFrontend::new();

    edit.resize(frontend.width() as i32, frontend.height() as i32);
    edit.draw(&mut frontend);
    frontend.present();
    loop {
        let event = frontend.poll_event();
        match event {
            // This case is here, since we want to have a 'way ouy' till we fixed bugs
            Event::KeyPressEvent(KeyPress::Shortcut('q')) => break,
            Event::KeyPressEvent(key) => edit.input(key),
            Event::Resize(w, h) => { edit.resize(w as i32, h as i32) }
            // _ => ()
        };
        frontend.clear();
        edit.draw(&mut frontend);
        frontend.present();
    }
    drop(frontend);
    drop(hold);
}
