use std::path::{Path, PathBuf};
use std::io;
use std::io::{Cursor, Read, Write};

use super::test;

use super::super::frontend::{Frontend, Event, Style, KeyPress};
use super::super::filesystem::Filesystem;

/// Represents an empty frontend, can be merged in the future with the mock frontend
pub struct EmptyFrontend;

impl Frontend for EmptyFrontend {
    fn clear(&self) {
    }

    fn present(&self) {
    }

    fn print_style(&self, x: usize, y: usize, style: Style, s: &str) {
        test::black_box((x, y, style, s));
    }

    fn print_char_style(&self, x: usize, y: usize, style: Style, c: char) {
        test::black_box((x, y, style, c));
    }

    fn print_slice_style(&self, x: usize, y: usize, style: Style, chars: &[char]) {
        test::black_box((x, y, style, chars));
    }

    fn set_cursor(&mut self, x: isize, y: isize) {
        test::black_box((x, y));
    }

    fn height(&self) -> usize {
        1024
    }

    fn width(&self) -> usize {
        1024
    }

    fn poll_event(&mut self) -> Event {
        panic!("Unimplemented!");
    }
}

pub trait EmptyFilesystem {
    fn open_config() -> Option<&'static [u8]>;
}

pub struct EmptyFile;

impl Read for EmptyFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        test::black_box(buf);
        Ok(0)
    }
}

impl Write for EmptyFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        test::black_box(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<T: EmptyFilesystem> Filesystem for T {
    type FSRead = Cursor<Vec<u8>>;
    type FSWrite = Cursor<Vec<u8>>;

    fn get_config_home() -> PathBuf {
        PathBuf::from("/")
    }

    fn make_absolute<P: AsRef<Path>>(p: P) -> io::Result<PathBuf> {
        Ok(p.as_ref().into())
    }

    fn open<P: AsRef<Path>>(path: P) -> io::Result<Self::FSRead> {
        if path.as_ref() == Path::new("/config/rex/rex.conf") {
            return Ok(Self::open_config().map_or_else(|| Cursor::new(vec![]),
                |v| Cursor::new(v.into())))
        }
        Ok(Cursor::new(vec![]))
    }

    fn can_open<P: AsRef<Path>>(_p: P) -> io::Result<()> {
        Ok(())
    }

    fn save<P: AsRef<Path>>(path: P) -> io::Result<Self::FSWrite> {
        Ok(Cursor::new(vec![]))
    }

    fn can_save<P: AsRef<Path>>(_p: P) -> io::Result<()> {
        Ok(())
    }
}
