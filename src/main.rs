extern crate rustbox;
extern crate rustc_serialize;
extern crate gag;
extern crate toml;
extern crate itertools;
extern crate docopt;

use std::path::Path;
use std::error::Error;
use std::io;
use std::io::Write;
use std::process;
use docopt::Docopt;
use rustbox::{RustBox, Event, InputMode, InitOptions};
use rustbox::keyboard::Key;
use gag::Hold;

#[macro_use] mod signals;
mod config;
mod ui;
mod buffer;
mod util;
mod segment;

use ui::view::HexEdit;
use config::Config;

static USAGE: &'static str = "
Usage: hyksa [options] [-C CONF... FILE]
       hyksa --help

Options:
    -h, --help                  Show this help message
    -c FILE, --config FILE      Use FILE as the config file
    -C CONF                     Set a configuration option, for example: -C line_width=16
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_FILE: Option<String>,
    flag_config: Option<String>,
    flag_C: Vec<String>,
}

fn exit_err<E: Error>(msg: &str, error: E) -> ! {
    write!(&mut io::stderr(), "{}: {}", msg, error).unwrap();
    process::exit(1);
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(
        |d| d.decode()).unwrap_or_else(
        |e| e.exit());

    let config_res = if let Some(config_filename) = args.flag_config {
        Config::from_file(config_filename)
    } else {
        Config::open_default()
    };
    let mut config = config_res.unwrap_or_else(
        |e| exit_err("Couldn't open config file", e)
    );

    for config_line in &args.flag_C {
        config.set_from_string(config_line).unwrap_or_else(
            |e| exit_err("Couldn't parse command line config option", e)
        );
    }

    let mut edit = HexEdit::new(config);

    if let Some(ref filename) = args.arg_FILE {
        edit.open(&Path::new(filename));
    }

    let hold = (Hold::stdout().unwrap(), Hold::stderr().unwrap());

    let rb = RustBox::init(InitOptions{
        buffer_stderr: false,
        input_mode: InputMode::Esc,
    }).unwrap();

    edit.resize(rb.width() as i32, rb.height() as i32);
    edit.draw(&rb);
    rb.present();
    loop {
        let event = rb.poll_event(false).unwrap();
        match event {
            // This case is here, since we want to have a 'way ouy' till we fixed bugs
            Event::KeyEvent(Some(Key::Ctrl('q'))) => break,
            Event::KeyEvent(Some(key)) => edit.input(key),
            Event::ResizeEvent(w, h) => { edit.resize(w, h) }
            _ => ()
        };
        rb.clear();
        edit.draw(&rb);
        rb.present();
    }
    drop(rb);
    drop(hold);
}
