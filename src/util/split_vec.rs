//! Provides a Vec-like container for large sizes that is split to blocks.

use std::fmt;
use std::ops;
use std::ops::{Range, RangeFrom, RangeTo, RangeFull};
use std::cmp;

use itertools;
use odds::vec::VecExt;

/// A generic trait over Rust's built types.
pub trait FromRange {
    #[inline(always)]
    #[doc(hidden)]
    fn from_range(&self, seg: &SplitVec) -> (usize, usize);
}

impl FromRange for RangeFull {
    #[inline(always)]
    fn from_range(&self, seg: &SplitVec) -> (usize, usize) {
        return (0, seg.len());
    }
}

impl FromRange for Range<usize> {
    #[inline(always)]
    fn from_range(&self, _: &SplitVec) -> (usize, usize) {
        return (self.start, self.end);
    }
}

impl FromRange for RangeFrom<usize> {
    #[inline(always)]
    fn from_range(&self, seg: &SplitVec) -> (usize, usize) {
        return (self.start, seg.len());
    }
}

impl FromRange for RangeTo<usize> {
    #[inline(always)]
    fn from_range(&self, _: &SplitVec) -> (usize, usize) {
        return (0, self.end);
    }
}

/// A Vec for large sizes split into smaller blocks.
///
/// Splits a large continuos memory into block sizes to allow insertion/deletion to have an
/// upper bound, based on the block size. It is implemented as a Vec of Vec's, where each Vec
/// has a m inimum and maximum block size. As data is inserted and deleted, from any location,
/// SplitVec will split up and merge the blocks to try and stay between those minimum and maximum
/// block sizes.
pub struct SplitVec {
    vecs: Vec<Vec<u8>>,
    length: usize,
}

#[derive(Copy, Clone, Debug)]
struct Index {
    outer: usize,
    inner: usize,
}

/// An iterator over SplitVec contents.
pub struct Items<'a> {
    seg: &'a SplitVec,
    index: Index,
    num_elem: Option<usize>,
}

/// A mutable iterator over SplitVec contents.
pub struct MutItems<'a> {
    seg: &'a mut SplitVec,
    index: Index,
    num_elem: Option<usize>,
}

/// An iterator over the blocks in SplitVec.
pub struct Slices<'a> {
    seg: &'a SplitVec,
    outer: usize,
}

static MIN_BLOCK_SIZE: usize = 1024 * 1024;
static MAX_BLOCK_SIZE: usize = 4 * 1024 * 1024;

impl SplitVec {
    /// Create a new, empty SplitVec
    pub fn new() -> SplitVec {
        SplitVec {
            vecs: Vec::new(),
            length: 0,
        }
    }

    /// Create a SplitVec by consuming a vec as the initial data vector
    pub fn from_vec(values: Vec<u8>) -> SplitVec {
        let len = values.len();
        SplitVec {
            vecs: vec![values],
            length: len,
        }
    }

    pub fn from_vecs(vecs: Vec<Vec<u8>>) -> SplitVec {
        let mut sv = SplitVec {
            vecs: vecs,
            length: 0,
        };
        sv.calc_len();
        sv
    }

    /// Create a SplitVec by copying in values from a slice
    pub fn from_slice(values: &[u8]) -> SplitVec {
        SplitVec {
            vecs: vec![values.into()],
            length: values.len(),
        }
    }

    /// Return the length.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Update the saved length value so that the len func will be -O(1)
    fn calc_len(&mut self) {
        self.length = 0;
        for len in self.vecs.iter().map(|v| v.len()) {
            self.length += len
        }
    }

    /// Convert a global pos to a locall index
    fn pos_to_index(&self, pos: usize, for_insert: bool) -> Index {
        if pos == 0 {
            return Index { outer: 0, inner: 0 };
        }

        let mut cur_pos = pos;
        for (i, vec) in self.vecs.iter().enumerate() {
            if cur_pos < vec.len() || (for_insert && cur_pos == vec.len()) {
                return Index {
                    outer: i,
                    inner: cur_pos,
                }
            }
            cur_pos -= vec.len();
        }

        panic!("Position {} is out of bounds", pos);
    }

    /// Give an iterator over a given range
    pub fn iter_range<'a, R: FromRange>(&'a self, range: R) -> Items<'a> {
        let (from, to) = range.from_range(self);
        if to < from {
            panic!("to ({}) is smaller than from ({})!", to, from);
        }

        let idx = self.pos_to_index(from, false);
        Items {
            seg: self,
            index: idx,
            num_elem: Some(to - from),
        }
    }

    /// Provide an iterator over continuous memory slices
    ///
    /// This is useful for doing an efficient save to disk
    pub fn iter_slices<'a>(&'a self) -> Slices<'a> {
        Slices {
            seg: self,
            outer: 0,
        }
    }

    /// Prepare an index for future text insertion, splitting/merging big/small sections respectively
    fn prepare_insert(&mut self, index: Index) -> Index {
        if index.outer >= self.vecs.len() {
            self.vecs.push(Vec::new());
        }

        if self.vecs[index.outer].len() < MAX_BLOCK_SIZE {
            return index;
        }

        let page_start_idx = (index.inner / MIN_BLOCK_SIZE) * MIN_BLOCK_SIZE;
        if page_start_idx == 0 {
            if self.vecs[index.outer].len() > MAX_BLOCK_SIZE {
                let insert_vec: Vec < _ >= self.vecs[index.outer][MIN_BLOCK_SIZE..].into();
                self.vecs.insert(index.outer + 1, insert_vec);
                self.vecs[index.outer].truncate(MIN_BLOCK_SIZE);
            }

            return index;
        } else {
            let insert_vec: Vec<_> = self.vecs[index.outer][page_start_idx..].into();
            self.vecs.insert(index.outer + 1, insert_vec);
            self.vecs[index.outer].truncate(page_start_idx);
            return self.prepare_insert(Index {
                outer: index.outer + 1,
                inner: index.inner - page_start_idx
            })
        }
    }

    /// insert all values from a slice at an offset.
    fn insert(&mut self, offset: usize, values: &[u8]) {
        let mut index = self.pos_to_index(offset, true);
        index = self.prepare_insert(index);

        // This is needed for the mut borrow vec
        {
            self.vecs[index.outer].splice(index.inner..index.inner, values.into_iter().cloned());
        }

        self.calc_len();
    }

    /// Moves data out from the supplied range.
    fn move_out<R: FromRange>(&mut self, range: R) -> Vec<u8> {
        let (from, to) = range.from_range(self);
        assert!(from <= to);
        let mut res = Vec::new();

        let begin = self.pos_to_index(from, false);
        let end = self.pos_to_index(to, true);

        // Simple case, by staying on the same section
        if begin.outer == end.outer {
            return self.vecs[begin.outer].drain(begin.inner..end.inner).collect();
        }

        // First drain out the result
        for outer_index in begin.outer..end.outer {
            // Drain what we can
            let drain_begin = if outer_index != begin.outer {
                0
            } else {
                begin.inner
            };
            let drain_end = if outer_index != end.outer {
                self.vecs[outer_index].len()
            } else {
                end.inner
            };

            res.extend(self.vecs[outer_index].drain(drain_begin..drain_end));
        }

        // TODO: Make this also possible to merge small vecs
        self.vecs.retain(|vec| vec.len() != 0);

        self.calc_len();

        res
    }

    /// Produce of copy of the supplied range
    pub fn copy_out<R: FromRange>(&mut self, range: R) -> Vec<u8> {
        let (from, to) = range.from_range(self);

        // TODO: Make this use direct cocpy rather than iterators
        self.iter_range(from..to).map(|x| *x).collect::<Vec<u8>>()
    }

    /// Replace values in range with the supplied values
    pub fn splice<R: FromRange>(&mut self, range: R, values: &[u8]) -> Vec<u8> {
        let (from, to) = range.from_range(self);
        let res;

        // Make sure that when we pull data out for the splice, we don't go over the end
        // TODO: Replace with better splice implementation once an implementation for rfc-1432
        // lands.
        if from < self.len() {
            let move_end = cmp::min(self.len(), to);
            res = self.move_out(from..move_end);
        } else {
            res = vec![];
        }

        self.insert(from, values);

        res
    }

    /// Find a slice.
    pub fn find_slice(&self, needle: &[u8]) -> Option<usize> {
        self.find_slice_from(0, needle)
    }

    /// Find a slice from a certain index and onward
    pub fn find_slice_from(&self, from: usize, needle: &[u8]) -> Option<usize> {
        for i in from..self.len() {
            if itertools::equal(self.iter_range(i..i+needle.len()), needle.iter()) {
                return Some(i);
            }
        }
        None
    }

    #[cfg(test)]
    fn get_lengths(&self) -> Vec<usize> {
        self.vecs.iter().map(|v| v.len()).collect::<Vec<usize>>()
    }
}

impl ops::Index<usize> for SplitVec {
    type Output = u8;
    fn index<'a>(&'a self, _index: usize) -> &'a u8 {
        let idx = self.pos_to_index(_index, false);
        &self.vecs[idx.outer][idx.inner]
    }
}

impl ops::IndexMut<usize> for SplitVec {
    fn index_mut<'a>(&'a mut self, _index: usize) -> &'a mut u8 {
        let idx = self.pos_to_index(_index, false);
        &mut self.vecs[idx.outer][idx.inner]
    }
}

impl fmt::Debug for SplitVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.vecs.fmt(f)
    }
}

impl<'a> Iterator for Items<'a> {
    type Item = &'a u8;
    fn next(&mut self) -> Option<&'a u8> {
        if self.index.outer >= self.seg.vecs.len() {
            return None;
        }
        if let Some(ref mut num_elem) = self.num_elem {
            if *num_elem <= 0 {
                return None;
            }
            *num_elem -= 1;
        }

        let elem = {
            let vv = &self.seg.vecs[self.index.outer];
            &vv[self.index.inner]
        };

        self.index.inner += 1;
        if self.index.inner >= self.seg.vecs[self.index.outer].len() {
            self.index.inner = 0;
            self.index.outer += 1;
        }

        Some(elem)
    }
}

impl<'a> Iterator for MutItems<'a> {
    type Item = &'a mut u8;
    fn next(&mut self) -> Option<&'a mut u8> {
        if self.index.outer >= self.seg.vecs.len() {
            return None;
        }
        if let Some(ref mut num_elem) = self.num_elem {
            if *num_elem <= 0 {
                return None;
            }
            *num_elem -= 1;
        }

        let elem_raw: *mut u8 = {
            let vv = &mut self.seg.vecs[self.index.outer];
            &mut vv[self.index.inner]
        };

        self.index.inner += 1;
        if self.index.inner >= self.seg.vecs[self.index.outer].len() {
            self.index.inner = 0;
            self.index.outer += 1;
        }

        Some(unsafe { &mut *elem_raw })
    }
}

impl<'a> Iterator for Slices<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        if self.outer >= self.seg.vecs.len() {
            None
        } else {
            let i = self.outer;
            self.outer += 1;
            Some(&self.seg.vecs[i])
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SIZE: usize = 30;

    fn create_test_split_vec() -> SplitVec {
        SplitVec::from_vecs(vec![vec![0; SIZE], vec![1; SIZE]])//, vec![2; SIZE], vec![3; SIZE]])
    }

    #[test]
    fn test_replace_in_split() {
        let mut sv = create_test_split_vec();

        sv.splice(SIZE-10..SIZE+10, &vec![5, 5]);

        assert_eq!(sv[SIZE-11], 0);
        assert_eq!(sv[SIZE-10], 5);
        assert_eq!(sv[SIZE-9], 5);
        assert_eq!(sv[SIZE-8], 1);
    }

    #[test]
    fn test_insert_in_split() {
        let mut sv = create_test_split_vec();

        sv.splice(SIZE..SIZE, &vec![5, 5]);

        assert_eq!(sv[SIZE-1], 0);
        assert_eq!(sv[SIZE], 5);
        assert_eq!(sv[SIZE+1], 5);
        assert_eq!(sv[SIZE+2], 1);
    }

    #[test]
    fn test_delete_in_split() {
        let mut sv = create_test_split_vec();

        sv.splice(SIZE-10..SIZE+10, &vec![]);

        assert_eq!(sv[SIZE-11], 0);
        assert_eq!(sv[SIZE-10], 1);
    }

    const MIDDLE: usize = SIZE/2;

    #[test]
    fn test_replace_in_middle() {
        let mut sv = create_test_split_vec();

        sv.splice(MIDDLE-10..MIDDLE+10, &vec![5, 5]);

        assert_eq!(sv[MIDDLE-11], 0);
        assert_eq!(sv[MIDDLE-10], 5);
        assert_eq!(sv[MIDDLE-9], 5);
        assert_eq!(sv[MIDDLE-8], 0);

        assert_eq!(sv[SIZE-19], 0);
        assert_eq!(sv[SIZE-18], 1);
    }

    #[test]
    fn test_insert_in_middle() {
        let mut sv = create_test_split_vec();

        sv.splice(MIDDLE..MIDDLE, &vec![5, 5]);

        assert_eq!(sv[MIDDLE-1], 0);
        assert_eq!(sv[MIDDLE], 5);
        assert_eq!(sv[MIDDLE+1], 5);
        assert_eq!(sv[MIDDLE+2], 0);
    }

    #[test]
    fn test_delete_in_middle() {
        let mut sv = create_test_split_vec();

        sv.splice(MIDDLE-2..MIDDLE+2, &vec![]);

        assert_eq!(sv[SIZE-5], 0);
        assert_eq!(sv[SIZE-4], 1);
    }

    #[test]
    fn test_small_splitvec() {
        let size = 1024;
        let mut seg = SplitVec::from_vec(vec![1, 2, 3, 4, 5]);
        assert_eq!(Some(4), seg.find_slice(&[5]));

        let seg_len = seg.len();
        seg.splice((seg_len/2)..(seg_len/2), &vec![1 as u8; size]);

        assert_eq!(Some(size + 4), seg.find_slice(&[5]));
    }

    #[test]
    fn test_large_splitvec() {
        let big_size = 4*1024*1024;
        let small_size = 1024;
        let mut seg = SplitVec::from_vec(vec![0; big_size]);

        seg.splice((big_size/2)..(big_size/2), &vec![1 as u8; small_size]);

        assert_eq!(Some(big_size/2 -1), seg.find_slice(&[0, 1]));

        // Make sure we actually tested a "split" version
        let seg_lengths = seg.get_lengths();
        assert_eq!(2, seg_lengths.len());
        let index = seg_lengths[0];
        let sentinal = 100;
        seg[index] = sentinal;
        seg[index+1] = sentinal +1;
        assert_eq!(Some(index), seg.find_slice(&[sentinal, sentinal+1]));
    }
}
