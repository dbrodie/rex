extern crate rustbox;
extern crate rustc_serialize;
extern crate gag;
extern crate toml;

use std::env::args;
use std::path::Path;
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

fn main() {
    let mut args = args();
    let config = match Config::open_default() {
        Ok(c) => c,
        Err(e) => {
            println!("Couldn't open config: {}", e);
            return;
        }
    };

    let mut edit = HexEdit::new(config);

    if args.len() > 1 {
        edit.open(&Path::new(&args.nth(1).unwrap()));
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
