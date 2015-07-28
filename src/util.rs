use std::iter::{Iterator, repeat};

pub fn slice_set<T: Clone>(dest: &mut [T], src: &[T]) {
    if dest.len() != src.len() {
        panic!("destination and source slices should be the same length! ({} != {})", dest.len(),
               src.len());
    }

    for (i, v) in src.iter().enumerate() {
        dest[i] = v.clone();
    }
}

// pub fn iter_set<'a, T: Clone, A: Iterator<Item=&'a T>, B: Iterator<Item=&'a mut T>>(dest: B, src: A)
// {
//  // TODO: Fail (how?) when they are not the same length?

//  for (s, d) in src.zip(dest) {
//      *d = s.clone();
//  }
// }

pub fn iter_equals<A: Eq, T: Iterator<Item = A>, U: Iterator<Item = A>>(mut a: T,
                                                                        mut b: U)
                                                                        -> bool {
    loop {
        match (a.next(), b.next()) {
            (None, None) => return true,
            (None, _) | (_, None) => return false,
            (Some(x), Some(y)) => if x != y { return false },
        }
    }
}

pub fn string_with_repeat(c: char, n: usize) -> String {
    let v: Vec<_> = repeat(c as u8).take(n).collect();
    String::from_utf8(v).unwrap()
}

pub fn is_between<N: PartialOrd>(num: N, a: N, b: N) -> bool {
    let (smaller, larger) = if a < b { (a, b) } else { (b, a) };
    (smaller <= num) && (num <= larger)
}

pub enum OptionalIter<L, R> {
    TrueIter(L),
    FalseIter(R)
}

impl<L, R, A> Iterator for OptionalIter<L, R> where 
    L: Iterator<Item=A>,
    R: Iterator<Item=A>
{
    type Item = A;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            OptionalIter::TrueIter(ref mut it) => it.next(),
            OptionalIter::FalseIter(ref mut it) => it.next(),
        }
    }
}

pub trait IteratorOptionalExt : Iterator {
    fn optional<L, R, F, G>(self, conditional: bool, mut f_left: F, mut f_right: G) -> OptionalIter<L, R>
            where Self: Sized,
                  F: FnOnce(Self) -> L,
                  G: FnOnce(Self) -> R
    {
        if conditional {
            OptionalIter::TrueIter(f_left(self))
        } else {
            OptionalIter::FalseIter(f_right(self))
        }
    }
}

impl<T: ?Sized> IteratorOptionalExt for T where T: Iterator { }
