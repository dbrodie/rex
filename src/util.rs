use std::iter::Iterator;

pub fn slice_set<T:Clone>(dest: &mut [T], src: &[T]) {
	if dest.len() != src.len() {
		panic!("destination and source slices should be the same length! ({} != {})", dest.len(), src.len());
	}

	for (i, v) in src.iter().enumerate() {
		dest[i] = v.clone();
	}
}

pub fn iter_set<'a, 'b, T: Clone, A: Iterator<&'a T>, B: Iterator<&'b mut T>>(dest: B, src: A) {
	// TODO: Fail (how?) when they are not the same length?

	for (s, d) in src.zip(dest) {
		*d = s.clone();
	}
}

pub fn iter_equals<A: Eq, T: Iterator<A>, U: Iterator<A>>(mut a: T, mut b: U) -> bool {
    loop {
        match (a.next(), b.next()) {
            (None, None) => return true,
            (None, _) | (_, None) => return false,
            (Some(x), Some(y)) => if x != y { return false },
        }
    }
}