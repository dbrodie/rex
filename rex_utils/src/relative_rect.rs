use std::ops::Add;
use std::ops::Sub;

use super::rect::Rect;

#[derive(Debug, Copy, Clone)]
pub enum RelativePos<T> {
    FromStart(T),
    FromEnd(T),
}

#[derive(Debug, Copy, Clone)]
pub enum RelativeSize<T> {
    Absolute(T),
    Relative(T),
}

#[derive(Debug, Copy, Clone)]
pub struct RelativeRect<T> {
    pub top: RelativePos<T>,
    pub left: RelativePos<T>,
    pub width: RelativeSize<T>,
    pub height: RelativeSize<T>,
}

impl<T: Copy + Add<T, Output=T> + Sub<T, Output=T>> RelativePos<T> {
    fn relative_to(&self, start: T, end: T) -> T {
        match *self {
            RelativePos::FromStart(n) => start + n,
            RelativePos::FromEnd(n) => end - n
        }
    }
}

impl<T: Copy + Add<T, Output=T> + Sub<T, Output=T>> RelativeSize<T> {
    fn relative_to(&self, size: T) -> T {
        match *self {
            RelativeSize::Absolute(n) => n,
            RelativeSize::Relative(n) => size - n
        }
    }
}

impl<T> RelativeRect<T>
        where T: Copy + Add<T, Output=T> + Sub<T, Output=T>
{
    pub fn get_absolute_to(&self, relative_to: Rect<T>) -> Rect<T> {
        Rect {
            top: self.top.relative_to(relative_to.top, relative_to.bottom()),
            left: self.left.relative_to(relative_to.left, relative_to.right()),
            width: self.width.relative_to(relative_to.width),
            height: self.height.relative_to(relative_to.height)
        }
    }
}
