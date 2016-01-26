use util::rect::Rect;

use super::super::frontend::{Frontend, KeyPress};
use super::input::Input;


pub trait Widget {
    fn input(&mut self, input: &Input, key: KeyPress) -> bool;
    fn draw(&mut self, rb: &mut Frontend, area: Rect<isize>, has_focus: bool);
}
