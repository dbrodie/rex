use std::default::Default;
use std::ops::Add;

pub struct Rect<T> {
    pub top: T,
    pub left: T,
    pub height: T,
    pub width: T
}

impl<T> Rect<T> {
    pub fn bottom(&self) -> <T as Add>::Output
            where T: Add+Copy {
        self.top + self.height
    }

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
