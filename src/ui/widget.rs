use rustbox::RustBox;
use rustbox::keyboard::Key;

use rex_utils::rect::Rect;

use super::input::Input;


pub trait Widget {
    fn input(&mut self, input: &Input, key: Key) -> bool;
    fn draw(&mut self, rb: &RustBox, area: Rect<isize>, has_focus: bool);
}
