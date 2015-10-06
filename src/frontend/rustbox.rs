use rustbox::{RustBox, Event, InputMode, InitOptions, Color, RB_NORMAL, RB_BOLD, RB_UNDERLINE};
use rustbox::Style as RB_Style;

use super::{Frontend, Style};

pub struct RustBoxFrontend {
    rustbox: RustBox,
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
            Style::Selection => (RB_NORMAL, Color::Black, Color::White),
            Style::Hint => (RB_UNDERLINE, Color::Default, Color::Default),
            Style::StatusBar => (RB_NORMAL, Color::Black, Color::White),
            Style::InputLine => (RB_BOLD, Color::White, Color::Blue),
            Style::MenuShortcut => (RB_BOLD, Color::Default, Color::Default),
            Style::MenuEntry => (RB_NORMAL, Color::Default, Color::Default),
            Style::MenuTitle => (RB_NORMAL, Color::Default, Color::Default),
        }
    }

    pub fn poll_event(&self) -> Event {
        self.rustbox.poll_event(false).unwrap()
    }
}

impl Frontend for RustBoxFrontend {
    fn clear(&self) {
        self.rustbox.clear();
    }

    fn present(&self) {
        self.rustbox.present();
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

    fn set_cursor(&self, x: isize, y: isize) {
        self.rustbox.set_cursor(x, y);
    }

    fn height(&self) -> usize {
        self.rustbox.height()
    }

    fn width(&self) -> usize {
        self.rustbox.width()
    }
}
