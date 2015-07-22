use std::io;
use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::io::Write;

use super::segment::Segment;

pub trait Buffer {
    fn from_path(p: &Path) -> io::Result<Segment>;
    fn save(&self, to: &Path) -> io::Result<()>;
    fn write(&mut self, offset: usize, val: &[u8]);
    fn read(&self, offset: usize, len: usize) -> Vec<u8>;
    fn find_from(&self, offset: usize, needle: &[u8]) -> Option<usize>;
    fn remove(&mut self, start_offset: usize, end_offset: usize) -> Vec<u8>;
}

impl Buffer for Segment {
    fn from_path(p: &Path) -> io::Result<Segment> {
        let mut v = vec!();
        let mut f = try!(File::open(p));
        try!(f.read_to_end(&mut v));
        Ok(Segment::from_vec(v))
    }

    fn save(&self, to: &Path) -> io::Result<()> {
        let f_r = File::create(to);
        f_r.and_then(|mut f| self.iter_slices()
                                 .fold(Ok(()), |res, val| res.and_then(|_| f.write_all(val))))
    }

    fn write(&mut self, offset: usize, val: &[u8]) {
        for (s, d) in val.iter().zip(self.mut_iter_range(offset..(offset + val.len()))) {
            *d = s.clone();
        }
    }

    fn read(&self, offset: usize, len: usize) -> Vec<u8> {
        self.iter_range(offset..(offset + len)).map(|x| *x).collect::<Vec<u8>>()
    }

    fn find_from(&self, offset: usize, needle: &[u8]) -> Option<usize> {
        self.find_slice_from(offset, needle)
    }

    fn remove(&mut self, start_offset: usize, end_offset: usize) -> Vec<u8> {
        self.move_out_slice(start_offset, end_offset)
    }
}
