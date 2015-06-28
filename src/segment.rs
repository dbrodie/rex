use std::mem;
use std::iter;
use std::fmt;
use std::ops;

use super::util;

pub struct Segment {
    vecs: Vec<Vec<u8>>,
    length: usize,
}

#[derive(Copy, Clone)]
struct Index {
    outer: usize,
    inner: usize,
}

struct Indexes<'a> {
    seg: &'a Segment,
    index: Index,
}

pub struct Items<'a> {
    seg: &'a Segment,
    index: Index,
    num_elem: Option<usize>,
}

pub struct MutItems<'a> {
    seg: &'a mut Segment,
    index: Index,
    num_elem: Option<usize>,
}

pub struct Slices<'a> {
    seg: &'a Segment,
    outer: usize,
}

static min_block_size: usize = 1024 * 1024;
static max_block_size: usize = 4 * 1024 * 1024;

impl Segment {
    pub fn _internal_debug(&self) -> Vec<usize> {
        self.vecs.iter().map(|v| v.len()).collect::<Vec<usize>>()
    }
    pub fn new() -> Segment {
        Segment {
            vecs: Vec::new(),
            length: 0,
        }
    }
    pub fn from_vec(values: Vec<u8>) -> Segment {
        let len = values.len();
        Segment {
            vecs: vec!(values),
            length: len,
        }
    }

    pub fn from_slice(values: &[u8]) -> Segment {
        Segment {
            vecs: vec!(values.into()),
            length: values.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    fn calc_len(&mut self) {
        self.length = 0;
        for len in self.vecs.iter().map(|v| v.len()) {
            self.length += len
        }
    }

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

    pub fn iter_range<'a>(&'a self, from: usize, to: usize) -> Items<'a> {
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

    pub fn mut_iter_range<'a>(&'a mut self, from: usize, to: usize) -> MutItems<'a> {
        if to < from {
            panic!("to ({}) is smaller than from ({})!", to, from);
        }

        let idx = self.pos_to_index(from, false);
        MutItems {
            seg: self,
            index: idx,
            num_elem: Some(to - from),
        }
    }

    fn iter_index<'a>(&'a self, from: usize) -> Indexes<'a> {
        let index = self.pos_to_index(from, false);
        Indexes {
            seg: self,
            index: index,
        }
    }

    pub fn iter_slices<'a>(&'a self) -> Slices<'a> {
        Slices {
            seg: self,
            outer: 0,
        }
    }

    fn prepare_insert(&mut self, index: Index) -> Index {
        // TODO: Get self.vecs.get(index.outer) into a local variable without ruining lifetimes?
        if index.outer >= self.vecs.len() {
            self.vecs.push(Vec::new());
        }

        if self.vecs[index.outer].len() < max_block_size {
            return index;
        }

        let page_start_idx = (index.inner / min_block_size) * min_block_size;
        if page_start_idx == 0 {
            if self.vecs[index.outer].len() > max_block_size {
                let insert_vec: Vec < _ >= self.vecs[index.outer][min_block_size..].into(); self
                                           .vecs.insert(index.outer + 1, insert_vec); self
                                           .vecs[index.outer].truncate(min_block_size);
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

    pub fn insert(&mut self, offset: usize, values: &[u8]) {
        let mut index = self.pos_to_index(offset, true);
        index = self.prepare_insert(index);

        // This is needed for the mut borrow vec
        {
            let vec = &mut self.vecs[index.outer];
            // TODO: There has to be a better way for this range
            for val in values.into_iter().rev() {
                vec.insert(index.inner, *val);
            }
        }

        self.calc_len();
    }

    pub fn move_out_slice(&mut self, start_offset: usize, end_offset: usize) -> Vec<u8> {
        assert!(start_offset <= end_offset);
        let mut res = Vec::new();
        let mut index = self.pos_to_index(start_offset, false);
        let num_elem = end_offset - start_offset;

        for _ in 0..num_elem {
            let c = self.vecs[index.outer].remove(index.inner);
            res.push(c);

            if index.inner >= self.vecs[index.outer].len() {
                if self.vecs[index.outer].len() == 0 {
                    self.vecs.remove(index.outer);
                } else {
                    index.inner = 0;
                    index.outer += 1;
                }
            }
        }

        self.calc_len();

        res
    }

    pub fn find_slice(&self, needle: &[u8]) {
        self.find_slice_from(0, needle);
    }

    pub fn find_slice_from(&self, from: usize, needle: &[u8]) -> Option<usize> {
        let len = self.len();

        for i in from..self.len() {
            if util::iter_equals(self.iter_range(i, len), needle.iter()) {
                return Some(i);
            }
        }
        None
    }
}

impl ops::Index<usize> for Segment {
    type Output = u8;
    fn index<'a>(&'a self, _index: usize) -> &'a u8 {
        let idx = self.pos_to_index(_index, false);
        &self.vecs[idx.outer][idx.inner]
    }
}

impl ops::IndexMut<usize> for Segment {
    fn index_mut<'a>(&'a mut self, _index: usize) -> &'a mut u8 {
        let idx = self.pos_to_index(_index, false);
        &mut self.vecs[idx.outer][idx.inner]
    }
}

impl fmt::Debug for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.vecs.fmt(f)
    }
}

impl<'a> Iterator for Indexes<'a> {
    type Item = Index;
    fn next(&mut self) -> Option<Index> {
        if self.index.outer >= self.seg.vecs.len() {
            return None;
        }

        let res = self.index;

        self.index.inner += 1;
        if self.index.inner >= self.seg.vecs[self.index.outer].len() {
            self.index.inner = 0;
            self.index.outer += 1;
        }
        Some(res)
    }
}

impl<'a> Iterator for Items<'a> {
    type Item = &'a u8;
    fn next(&mut self) -> Option<&'a u8> {
        if self.index.outer >= self.seg.vecs.len() {
            return None;
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

#[test]
fn test_segment() {
    let mut s = Segment::from_slice(&[1, 2, 3, 4]);
    s.insert_slice(0, &[7, 7, 7, 7, 7]);
}
