extern crate rustbox;
extern crate rustc_serialize;
extern crate gag;

use std::env::args;
use std::path::Path;
use rustbox::{RustBox, Event, InputMode, InitOptions};
use rustbox::keyboard::Key;
use std::default::Default;
use gag::Hold;

#[macro_use] mod signals;
mod ui;
mod buffer;
mod util;
mod segment;

use ui::view::HexEdit;

fn main() {
    let mut args = args();
    let mut edit = HexEdit::new();

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
    // tb::set_cursor(0, 0);
    rb.present();
    // tb::set_cursor(2, 0);
    loop {
        let event = rb.poll_event(false).unwrap();
        // println!("{:?}", event);
        match event {
            // This case is here, since we want to have a 'way ouy' till we fixed bugs
            Event::KeyEvent(Some(Key::Char('\u{0}'))) => break,  /** This should be Ctrl-` */
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

// fn test_main() {
//     let mut s = Segment::from_slice(&[1,2,3,4]);
//     println!("Segment {}", s);

//     s.insert_slice(0, &[5,6,7,8,9,10,11,12]);

//     s.insert_slice(0, &[100]);

//     println!("Segment {}", s);

//     println!("Slice {}", s.move_out_slice(0, 7));
//     // println!("Slice {}", );
//     println!("Segment {}", s);

//     println!("Slice {}", s.move_out_slice(0, 6));
//     // println!("Slice {}", );
//     println!("Segment {}", s);

// }
