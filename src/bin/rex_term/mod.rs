use rustbox::{RustBox, InputMode, InitOptions, Color, RB_NORMAL, RB_BOLD, RB_UNDERLINE, RB_REVERSE};
use rustbox::keyboard::Key;
use rustbox::Event as RB_Event;
use rustbox::Style as RB_Style;

use rex::frontend::{Frontend, Style, Event, KeyPress};

pub struct RustBoxFrontend {
    rustbox: RustBox,
}

macro_rules! simple_enum_convert {
    ($from:ident : $from_type:ident into $to_type:ident for $($case:ident),*) => ({
        match $from {
            $(
                $from_type::$case => return $to_type::$case,
            )*
            _ => (),
        };
    })
}

impl RustBoxFrontend {
    pub fn new() -> RustBoxFrontend {
        RustBoxFrontend {
            rustbox: RustBox::init(InitOptions{
                buffer_stderr: false,
                input_mode: InputMode::Esc,
            }).unwrap()
        }
    }

    fn style_to_triple(style: Style) -> (RB_Style, Color, Color) {
        match style {
            Style::Default => (RB_NORMAL, Color::Default, Color::Default),
            Style::Selection => (RB_REVERSE, Color::Default, Color::Default),
            Style::Hint => (RB_UNDERLINE, Color::Default, Color::Default),
            Style::StatusBar => (RB_REVERSE, Color::Default, Color::Default),
            Style::InputLine => (RB_BOLD, Color::White, Color::Blue),
            Style::InputLineError => (RB_BOLD, Color::White, Color::Red),
            Style::MenuShortcut => (RB_BOLD, Color::Default, Color::Default),
            Style::MenuEntry => (RB_NORMAL, Color::Default, Color::Default),
            Style::MenuTitle => (RB_NORMAL, Color::Default, Color::Default),
        }
    }

    fn convert_key(key: Key) -> KeyPress {
        simple_enum_convert!(key : Key into KeyPress for
            Left,
            Right,
            Up,
            Down,
            PageUp,
            PageDown,
            Home,
            End,
            Backspace,
            Delete,
            Tab,
            Insert,
            Enter,
            Esc
        );
        match key {
            Key::Char('\u{0}') => KeyPress::Shortcut(' '),
            Key::Char(c) => KeyPress::Key(c),
            Key::Ctrl(c) => KeyPress::Shortcut(c),
            _ => panic!("Unhandled key found!"),
        }
    }
}

impl Frontend for RustBoxFrontend {
    fn clear(&self) {
        self.rustbox.clear();
    }

    fn present(&self) {
        self.rustbox.present();
    }

    fn poll_event(&mut self) -> Event {
        loop {
            match self.rustbox.poll_event(false).unwrap() {
                RB_Event::KeyEvent(Some(key)) => return Event::KeyPressEvent(RustBoxFrontend::convert_key(key)),
                RB_Event::ResizeEvent(w, h) => return Event::Resize(w as usize, h as usize),
                e @ _ => {
                    println!("Unhandled rustbox event: {:?}", e);
                    continue;
                }
            }
        }
    }

    fn print_style(&self, x: usize, y: usize, style: Style, s: &str) {
        let (st, fg, bg) = RustBoxFrontend::style_to_triple(style);
        self.rustbox.print(x, y, st, fg, bg, s);
    }

    fn print_char_style(&self, x: usize, y: usize, style: Style, c: char) {
        let (st, fg, bg) = RustBoxFrontend::style_to_triple(style);
        self.rustbox.print_char(x, y, st, fg, bg, c);
    }

    fn print_slice_style(&self, x: usize, y: usize, style: Style, chars: &[char]) {
        let (st, fg, bg) = RustBoxFrontend::style_to_triple(style);
        for (i, c) in chars.iter().enumerate() {
            self.rustbox.print_char(x + i, y, st, fg, bg, *c);
        }
    }

    fn set_cursor(&mut self, x: isize, y: isize) {
        self.rustbox.set_cursor(x, y);
    }

    fn height(&self) -> usize {
        self.rustbox.height()
    }

    fn width(&self) -> usize {
        self.rustbox.width()
    }
}
