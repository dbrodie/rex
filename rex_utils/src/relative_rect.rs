//! A rect that has relative values to be calculated from a concrete rect.
use std::ops::Add;
use std::ops::Sub;

use super::rect::Rect;

/// A position (top or left) relative to a parent's position (top or left).
#[derive(Debug, Copy, Clone)]
pub enum RelativePos<T> {
    /// The position is calculated from the start of the parent position.
    FromStart(T),
    /// the position is *substracted* from the parent struct.
    FromEnd(T),
}

/// A size (height or width) relative to a parent size (height or width).
#[derive(Debug, Copy, Clone)]
pub enum RelativeSize<T> {
    /// The size is absolute, it is up to the creator to make sure that it is not larger
    /// than the parent size.
    Absolute(T),
    /// A size that is relative to the parent and is smaller than the parent struct by the given units.
    /// The units are *substracted* from the parent.
    Relative(T),
}

/// A relative rect struct.
///
/// A rect that can be positioned relative to a regular rect. This is geared towards rects that
/// are contained in an other, absolute, rect. As such, the relative fields are geard towards
/// substraction.
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
    /// Returns an absolute rect resolving the relative values based on the supplied rect.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex_utils::relative_rect::{RelativeRect, RelativePos, RelativeSize};
    /// use rex_utils::rect::Rect;
    ///
    /// let sub_rect = RelativeRect {
    ///     // This is addition on .top
    ///     top: RelativePos::FromStart(5),
    ///     // This is substraction from .right()
    ///     left: RelativePos::FromEnd(10),
    ///     // This is substration from .width
    ///     width: RelativeSize::Absolute(4),
    ///     // This is an absolute values, not modified
    ///     height: RelativeSize::Relative(5)
    /// };
    ///
    /// let abs_rect = Rect {
    ///     top: 10,
    ///     left: 10,
    ///     height: 40,
    ///     width: 40,
    /// };
    ///
    /// assert_eq!(sub_rect.get_absolute_to(abs_rect), Rect {
    ///     top: 15,
    ///     left: 40,
    ///     width: 4,
    ///     height: 35});
    /// ```
    pub fn get_absolute_to(&self, relative_to: Rect<T>) -> Rect<T> {
        Rect {
            top: self.top.relative_to(relative_to.top, relative_to.bottom()),
            left: self.left.relative_to(relative_to.left, relative_to.right()),
            width: self.width.relative_to(relative_to.width),
            height: self.height.relative_to(relative_to.height)
        }
    }
}
