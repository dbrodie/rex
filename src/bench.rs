#[cfg(feature = "nightly")]


mod rex_bench {
    use std::path::{Path, PathBuf};
    use std::io;
    use std::io::{Cursor, Read, Write};

    extern crate test;
    use self::test::Bencher;
    use super::super::ui::view::HexEdit;
    use super::super::frontend::{Frontend, Event, Style, KeyPress};
    use super::super::filesystem::Filesystem;


    /// Represents an empty frontend, can be merged in the future with the mock frontend
    struct EmptyFrontend;

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

    trait EmptyFilesystem {
        fn open_config() -> Option<&'static [u8]>;
    }

    struct EmptyFile;

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

}
