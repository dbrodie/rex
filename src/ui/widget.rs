use rustbox::keyboard::Key;

use rex_utils::rect::Rect;

use super::super::frontend::Frontend;
use super::input::Input;


pub trait Widget {
    fn input(&mut self, input: &Input, key: Key) -> bool;
    fn draw(&mut self, rb: &Frontend, area: Rect<isize>, has_focus: bool);
}
