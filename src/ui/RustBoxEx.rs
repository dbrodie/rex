use rustbox::{RustBox, Color, RB_NORMAL, RB_BOLD};
use rustbox::Style as RB_Style;

#[derive(Copy, Clone)]
pub enum Style {
    Default,
    Selection,
    StatusBar,
    InputLine,
    MenuShortcut,
    MenuEntry,
    MenuTitle
}

impl Style {
    fn to_triple(self) -> (RB_Style, Color, Color) {
        match self {
            Style::Default => (RB_NORMAL, Color::Default, Color::Default),
            Style::Selection => (RB_NORMAL, Color::Black, Color::White),
            Style::StatusBar => (RB_NORMAL, Color::Black, Color::White),
            Style::InputLine => (RB_BOLD, Color::White, Color::Blue),
            Style::MenuShortcut => (RB_BOLD, Color::Default, Color::Default),
            Style::MenuEntry => (RB_NORMAL, Color::Default, Color::Default),
            Style::MenuTitle => (RB_NORMAL, Color::Default, Color::Default),
        }
    }
}

pub trait RustBoxEx {
    fn print_style(&self, x: usize, y: usize, style: Style, s: &str);
    fn print_char_style(&self, x: usize, y: usize, style: Style, c: char);
    fn print_slice_style(&self, x: usize, y: usize, style: Style, chars: &[char]);
}

impl RustBoxEx for RustBox {
    fn print_style(&self, x: usize, y: usize, style: Style, s: &str) {
        let (st, fg, bg) = style.to_triple();
        self.print(x, y, st, fg, bg, s);
    }

    fn print_char_style(&self, x: usize, y: usize, style: Style, c: char) {
        let (st, fg, bg) = style.to_triple();
        self.print_char(x, y, st, fg, bg, c);
    }

    fn print_slice_style(&self, x: usize, y: usize, style: Style, chars: &[char]) {
        let (st, fg, bg) = style.to_triple();
        for (i, c) in chars.iter().enumerate() {
            self.print_char(x + i, y, st, fg, bg, *c);
        }
    }
}
