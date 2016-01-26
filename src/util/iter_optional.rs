//! A trait extension on iterators that allows optionally adding an iterator adaptor to an iterator.
use std::iter::Iterator;

/// An iterator adaptor that returns elements from one iterator or another based on a conditional
///
/// See [*.optional()*](trait.IterOptionalExt.html#method.optional) for more information.
pub enum IterOptional<T, F> {
    TrueIter(T),
    FalseIter(F)
}

impl<T, F, A> Iterator for IterOptional<T, F> where
    T: Iterator<Item=A>,
    F: Iterator<Item=A>
{
    type Item = A;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            IterOptional::TrueIter(ref mut it) => it.next(),
            IterOptional::FalseIter(ref mut it) => it.next(),
        }
    }
}

/// The trait IteratorOptional provides conditional iterator selection
pub trait IterOptionalExt : Iterator {

    /// Run and return elements from the iterator returned from the closure based on the
    /// conditional.
    ///
    /// # Examples
    ///
    /// ```
    /// use rex::util::iter_optional::IterOptionalExt;
    ///
    /// let src_iter = vec![1, 2, 3].into_iter();
    /// let optional_iterator = src_iter.optional(true, |iter| iter.map(|x| x*2), |iter| iter);
    /// assert_eq!(optional_iterator.collect::<Vec<_>>(), [2, 4, 6]);
    ///
    /// let src_iter = vec![1, 2, 3].into_iter();
    /// let optional_iterator = src_iter.optional(false, |iter| iter.map(|x| x*2), |iter| iter);
    /// assert_eq!(optional_iterator.collect::<Vec<_>>(), [1, 2, 3]);
    /// ```
    fn optional<T, F, G, H>(self, conditional: bool, true_func: G, false_func: H) -> IterOptional<T, F>
            where Self: Sized,
                  G: FnOnce(Self) -> T,
                  H: FnOnce(Self) -> F
    {
        if conditional {
            IterOptional::TrueIter(true_func(self))
        } else {
            IterOptional::FalseIter(false_func(self))
        }
    }
}

impl<T: ?Sized> IterOptionalExt for T where T: Iterator { }
