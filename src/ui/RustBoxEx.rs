use rustbox::{RustBox, Color, RB_NORMAL, RB_BOLD};
use rustbox::Style as RB_Style;

#[derive(Copy, Clone)]
pub enum Style {
    Default,
    Selection,
    StatusBar,
    InputLine,
}

impl Style {
    fn to_triple(self) -> (RB_Style, Color, Color) {
        match self {
            Style::Default => (RB_NORMAL, Color::Default, Color::Default),
            Style::Selection => (RB_NORMAL, Color::Black, Color::White),
            Style::StatusBar => (RB_NORMAL, Color::Black, Color::White),
            Style::InputLine => (RB_BOLD, Color::White, Color::Blue),
        }
    }
}

pub trait RustBoxEx {
    fn print_style(&self, x: usize, y: usize, style: Style, s: &str);
}

impl RustBoxEx for RustBox {
    fn print_style(&self, x: usize, y: usize, style: Style, s: &str) {
        let (st, fg, bg) = style.to_triple();
        self.print(x, y, st, fg, bg, s);
    }
}
