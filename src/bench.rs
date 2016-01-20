#[cfg(feature = "nightly")]


mod rex_bench {
    extern crate test;
    use self::test::Bencher;
    use super::super::ui::view::HexEdit;
    use super::super::frontend::{Frontend, Event, Style, KeyPress};


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

    #[bench]
    fn bench_code_chunks(b: &mut Bencher) {
        let mut edit: HexEdit = HexEdit::new();
        let mut frontend = EmptyFrontend;
        edit.open_vec((0..0x10000).map(|x| (x & 0xff) as u8).collect());

        b.iter(|| {
            edit.draw(&mut frontend);
        });
    }

}
