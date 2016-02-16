use std::path::{Path, PathBuf};
use std::io;
use std::io::{Cursor, Read, Write};

use super::test::Bencher;
use super::super::ui::view::HexEdit;
use super::super::frontend::{Frontend, Event, Style, KeyPress};

use super::util::{EmptyFilesystem, EmptyFrontend};

struct DefaultEmptyFS;
impl EmptyFilesystem for DefaultEmptyFS {
    fn open_config() -> Option<&'static [u8]> {
        None
    }
}

#[bench]
fn bench_draw_default(b: &mut Bencher) {
    let mut edit: HexEdit<DefaultEmptyFS> = HexEdit::new();
    let mut frontend = EmptyFrontend;
    edit.open_vec((0..0x10000).map(|x| (x & 0xff) as u8).collect());

    b.iter(|| {
        edit.draw(&mut frontend);
    });
}

struct NoAsciiEmptyFS;
impl EmptyFilesystem for NoAsciiEmptyFS {
    fn open_config() -> Option<&'static [u8]> {
        Some(b"show_ascii=false\nshow_linenum=false")
    }
}

#[bench]
fn bench_draw_no_ascii(b: &mut Bencher) {
    let mut edit: HexEdit<NoAsciiEmptyFS> = HexEdit::new();
    let mut frontend = EmptyFrontend;
    edit.open_vec((0..0x10000).map(|x| (x & 0xff) as u8).collect());

    b.iter(|| {
        edit.draw(&mut frontend);
    });
}
