extern crate rustbox;
extern crate serialize;

use std::os;
use std::env::args;
use std::path::Path;
use rustbox::{RustBox, Event};
use std::default::Default;


mod ui;
mod buffer;
mod util;
mod segment;


fn main() {
    let args = args();
    let mut edit = ui::HexEdit::new();

    if args.len() > 1 {
        edit.open(&Path::new(args.nth(1).as_slice()));
    }

    let rb = RustBox::init(Default::default());

    edit.resize(rb.width() as i32, rb.height() as i32);
    edit.draw(rb);
    // tb::set_cursor(0, 0);
    rb.present();
    // tb::set_cursor(2, 0);
    loop {
    	let event = rb.poll_event();
    	// println!("{:?}", event);
	    match event {
	    	Event::KeyEvent(0, 0, 0) => break,
	    	Event::KeyEvent(m, k, c) => edit.input(m, k, c),
	    	Event::ResizeEvent(w, h) => { edit.resize(w, h) }
	    	_ => ()
	    };
	    rb.clear();
	    edit.draw(rb);
	    rb.present();
	}
    drop(rb);
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
