//! Provides a simple Rect struct.

use std::default::Default;
use std::ops::Add;

/// A simple rect struct.
#[derive(Debug)]
pub struct Rect<T> {
    pub top: T,
    pub left: T,
    pub height: T,
    pub width: T
}

impl<T> Rect<T> {
    /// Returns the value of the bottom of the struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::default::Default;
    /// use rex_utils::rect::Rect;
    ///
    /// let rect = Rect { top: 2, height: 5, ..Default::default()};
    /// assert_eq!(rect.bottom(), 7);
    /// ```
    pub fn bottom(&self) -> <T as Add>::Output
            where T: Add+Copy {
        self.top + self.height
    }

    /// Returns the value of the right of the struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::default::Default;
    /// use rex_utils::rect::Rect;
    ///
    /// let rect = Rect { left: 2, width: 5, ..Default::default()};
    /// assert_eq!(rect.right(), 7);
    /// ```
    pub fn right(&self) -> <T as Add>::Output
            where T: Add+Copy {
        self.left + self.width
    }
}

impl<T> Default for Rect<T>
        where T: Default {
    fn default() -> Rect<T> {
        Rect {
            top: Default::default(),
            left: Default::default(),
            height: Default::default(),
            width: Default::default(),
        }
    }
}
