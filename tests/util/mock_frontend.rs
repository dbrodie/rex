use rex::filesystem::Filesystem;
use rex::frontend::{Frontend, Event, Style, KeyPress};
use rex::ui::view::HexEdit;

pub struct MockFrontend {
    cursor: (usize, usize),
    size: (usize, usize),
}

impl MockFrontend {
    pub fn new() -> MockFrontend {
        MockFrontend {
            cursor: (0, 0),
            size: (100, 100),
        }
    }

    pub fn run_str<FS: Filesystem+'static>(&mut self, edit: &mut HexEdit<FS>, s: &str) {
        for c in s.chars() {
            edit.input(KeyPress::Key(c));
            edit.draw(self);
        }
    }

    pub fn run_keys<I, FS: Filesystem+'static>(&mut self, edit: &mut HexEdit<FS>, keys: I) where
            I: IntoIterator<Item=KeyPress> {
        for key in keys {
            edit.input(key);
            edit.draw(self);
        }
    }

    pub fn run_events<I, FS: Filesystem+'static>(&mut self, edit: &mut HexEdit<FS>, events: I) where
            I: IntoIterator<Item=Event> {
        for event in events {
            match event {
                Event::KeyPressEvent(key) => edit.input(key),
                Event::Resize(w, h) => {
                    self.size = (w, h);
                    edit.resize(w as i32, h as i32)
                }
            }
            edit.draw(self);
        }
    }
}

impl Frontend for MockFrontend {
    fn clear(&self) {
    }

    fn present(&self) {
    }

    fn print_style(&self, _x: usize, _y: usize, _style: Style, _s: &str) {
    }

    fn print_char_style(&self, _x: usize, _y: usize, _style: Style, _c: char) {
    }

    fn print_slice_style(&self, _x: usize, _y: usize, _style: Style, _chars: &[char]) {
    }

    fn set_cursor(&mut self, x: isize, y: isize) {
        self.cursor = (x as usize, y as usize);
    }

    fn height(&self) -> usize {
        self.size.1
    }

    fn width(&self) -> usize {
        self.size.0
    }

    fn poll_event(&mut self) -> Event {
        panic!("Unimplemented!");
    }
}
